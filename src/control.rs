use std::{
    cmp::Reverse,
    fs,
    io::{Read, Write},
    net::{Shutdown, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    thread::JoinHandle,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::util::concurrency::NotifierSender;

const CONTROL_HOST: &str = "127.0.0.1";
const CONTROL_PORT_START: u16 = 21740;
const CONTROL_PORT_END: u16 = 21790;
const CONTROL_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);
const CONTROL_CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const CONTROL_IO_TIMEOUT: Duration = Duration::from_secs(2);
const CONTROL_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);
const CONTROL_FORWARD_TIMEOUT: Duration = Duration::from_secs(4);
const CONTROL_PROTOCOL_VERSION: u32 = 2;
const CONTROL_MAX_MESSAGE_BYTES: usize = 1024 * 1024;

#[derive(Serialize, Deserialize)]
struct ControlOpenRequest {
    protocol_version: u32,
    instance_id: String,
    expires_at_ms: u128,
    open_paths: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ControlOpenResponse {
    protocol_version: u32,
    instance_id: String,
}

#[derive(PartialEq)]
enum OpenStatus {
    Pending,
    Accepted,
    Cancelled,
}

pub struct ControlOpenEvent {
    paths: Vec<PathBuf>,
    deadline: Instant,
    status: Arc<Mutex<OpenStatus>>,
    accepted_tx: mpsc::Sender<()>,
}

impl ControlOpenEvent {
    /// Only the UI may accept ownership. Cancelled or late events must not
    /// open files after the sender has already fallen back to a new window.
    pub fn accept(self) -> Option<Vec<PathBuf>> {
        let mut status = self.status.lock().unwrap();
        if *status != OpenStatus::Pending || Instant::now() >= self.deadline {
            return None;
        }
        *status = OpenStatus::Accepted;
        if self.accepted_tx.send(()).is_err() {
            *status = OpenStatus::Cancelled;
            return None;
        }
        Some(self.paths)
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct WindowRecord {
    instance_id: String,
    pid: u32,
    control_addr: String,
    last_active_ms: u128,
}

#[derive(Default, Serialize, Deserialize)]
struct WindowRegistry {
    windows: Vec<WindowRecord>,
}

pub struct ControlInstance {
    instance_id: String,
    pid: u32,
    control_addr: String,
    stop: Arc<AtomicBool>,
    active_stream: Arc<Mutex<Option<TcpStream>>>,
    handle: Option<JoinHandle<()>>,
}

impl ControlInstance {
    pub fn address(&self) -> &str {
        &self.control_addr
    }

    pub fn touch_active(&self) -> Result<(), String> {
        let mut registry = load_registry()?;
        let now = now_millis()?;
        registry.windows.retain(|record| record.instance_id != self.instance_id);
        registry.windows.push(WindowRecord {
            instance_id: self.instance_id.clone(),
            pid: self.pid,
            control_addr: self.control_addr.clone(),
            last_active_ms: now,
        });
        save_registry(&registry)
    }

    pub fn remove(&self) -> Result<(), String> {
        let mut registry = load_registry()?;
        registry.windows.retain(|record| record.instance_id != self.instance_id);
        save_registry(&registry)
    }

    pub fn shutdown(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(stream) = self.active_stream.lock().unwrap().take() {
            let _ = stream.shutdown(Shutdown::Both);
        }
        if let Some(handle) = self.handle.take() {
            let (tx, rx) = std::sync::mpsc::channel();
            thread::spawn(move || {
                let _ = tx.send(handle.join());
            });
            match rx.recv_timeout(CONTROL_SHUTDOWN_TIMEOUT) {
                Ok(Ok(())) => {}
                Ok(Err(_)) => eprintln!("Control listener thread panicked during shutdown"),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    eprintln!("Control listener shutdown timed out");
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("Control listener shutdown result was disconnected");
                }
            }
        }
    }
}

impl Drop for ControlInstance {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn start_control_listener(tx: NotifierSender<ControlOpenEvent>) -> Result<ControlInstance, String> {
    for port in CONTROL_PORT_START..=CONTROL_PORT_END {
        let addr = format!("{CONTROL_HOST}:{port}");
        match TcpListener::bind(addr.as_str()) {
            Ok(listener) => {
                listener
                    .set_nonblocking(true)
                    .map_err(|e| format!("Failed to configure control listener: {e}"))?;

                let stop = Arc::new(AtomicBool::new(false));
                let stop_thread = stop.clone();
                let active_stream = Arc::new(Mutex::new(None::<TcpStream>));
                let active_stream_thread = active_stream.clone();
                let instance_id = format!("{}-{}", std::process::id(), now_millis()?);
                let listener_instance_id = instance_id.clone();

                let handle = thread::spawn(move || loop {
                    if stop_thread.load(Ordering::Relaxed) {
                        break;
                    }

                    match listener.accept() {
                        Ok((mut stream, _peer)) => {
                            match stream.try_clone() {
                                Ok(cloned) => {
                                    *active_stream_thread.lock().unwrap() = Some(cloned);
                                }
                                Err(err) => {
                                    eprintln!("Failed to track control stream: {err}");
                                    continue;
                                }
                            }
                            if stop_thread.load(Ordering::Relaxed) {
                                active_stream_thread.lock().unwrap().take();
                                let _ = stream.shutdown(Shutdown::Both);
                                break;
                            }
                            if let Err(err) = handle_request(&mut stream, &listener_instance_id, &tx, &stop_thread) {
                                eprintln!("File-open control request failed: {err}");
                            }
                            active_stream_thread.lock().unwrap().take();
                        }
                        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(50));
                        }
                        Err(err) => {
                            eprintln!("Control listener error: {err}");
                            break;
                        }
                    }
                });

                let instance = ControlInstance {
                    instance_id,
                    pid: std::process::id(),
                    control_addr: addr,
                    stop,
                    active_stream,
                    handle: Some(handle),
                };
                instance.touch_active()?;
                return Ok(instance);
            }
            Err(err) if err.kind() == std::io::ErrorKind::AddrInUse => continue,
            Err(err) => return Err(format!("Failed to bind control listener: {err}")),
        }
    }

    Err("Failed to find a free control port".to_string())
}

pub fn try_forward_paths_to_last_active(paths: &[PathBuf]) -> Result<bool, String> {
    // Resolve relative paths in the launching process, not the receiving one.
    let cwd = std::env::current_dir().map_err(|e| format!("Failed to resolve file-open paths: {e}"))?;
    let paths: Vec<_> = paths.iter().map(|path| cwd.join(path)).collect();
    let mut registry = load_registry()?;
    registry.windows.sort_by_key(|window| Reverse(window.last_active_ms));

    let (forwarded, failed_instances) = forward_to_windows(&registry.windows, &paths);

    if !failed_instances.is_empty() {
        // Reload so a successful forward doesn't overwrite a newly registered
        // window with the snapshot used for choosing the destination.
        let cleanup = load_registry().and_then(|mut registry| {
            registry
                .windows
                .retain(|window| !failed_instances.contains(&window.instance_id));
            save_registry(&registry)
        });
        if let Err(err) = cleanup {
            eprintln!("Failed to clean stale Edolview window registrations: {err}");
        }
    }

    Ok(forwarded)
}

fn forward_to_windows(windows: &[WindowRecord], paths: &[PathBuf]) -> (bool, Vec<String>) {
    let mut failed_instances = Vec::new();
    let deadline = Instant::now() + CONTROL_FORWARD_TIMEOUT;
    let mut forwarded = false;
    for record in windows {
        if Instant::now() >= deadline {
            break;
        }
        if record.pid == std::process::id() {
            continue;
        }

        match send_request(record, paths, deadline) {
            Ok(()) => {
                forwarded = true;
                break;
            }
            Err(err) => {
                eprintln!("Failed to forward files to Edolview instance {}: {err}", record.instance_id);
                failed_instances.push(record.instance_id.clone());
            }
        }
    }

    (forwarded, failed_instances)
}

fn read_message<T: serde::de::DeserializeOwned>(stream: &mut TcpStream, deadline: Instant) -> Result<T, String> {
    let mut body = Vec::new();
    let mut chunk = [0; 8192];
    loop {
        stream
            .set_read_timeout(Some(remaining_timeout(deadline)?))
            .map_err(|e| format!("Failed to set read timeout: {e}"))?;
        let count = stream
            .read(&mut chunk)
            .map_err(|e| format!("Failed to read control message: {e}"))?;
        if count == 0 {
            break;
        }
        if body.len() + count > CONTROL_MAX_MESSAGE_BYTES {
            return Err("File-open control message is too large".to_string());
        }
        body.extend_from_slice(&chunk[..count]);
    }

    serde_json::from_slice(&body).map_err(|e| format!("Failed to parse control message: {e}"))
}

fn handle_request(
    stream: &mut TcpStream,
    instance_id: &str,
    tx: &NotifierSender<ControlOpenEvent>,
    stop: &AtomicBool,
) -> Result<(), String> {
    // Windows accepted sockets inherit the listener's nonblocking mode.
    // Use bounded blocking reads on this background thread to wait for EOF.
    stream
        .set_nonblocking(false)
        .map_err(|e| format!("Failed to configure control stream: {e}"))?;
    stream
        .set_read_timeout(Some(CONTROL_IO_TIMEOUT))
        .map_err(|e| format!("Failed to set read timeout: {e}"))?;
    stream
        .set_write_timeout(Some(CONTROL_IO_TIMEOUT))
        .map_err(|e| format!("Failed to set write timeout: {e}"))?;
    let request: ControlOpenRequest = read_message(stream, Instant::now() + CONTROL_IO_TIMEOUT)?;
    if request.protocol_version != CONTROL_PROTOCOL_VERSION || request.instance_id != instance_id {
        return Err("File-open control protocol or instance does not match".to_string());
    }
    let remaining_ms = request.expires_at_ms.saturating_sub(now_millis()?);
    if remaining_ms == 0 || request.open_paths.is_empty() {
        return Err("File-open request is expired or empty".to_string());
    }
    let deadline = Instant::now() + Duration::from_millis(remaining_ms.min(CONTROL_REQUEST_TIMEOUT.as_millis()) as u64);
    let (accepted_tx, accepted_rx) = mpsc::channel();
    let status = Arc::new(Mutex::new(OpenStatus::Pending));
    tx.send(ControlOpenEvent {
        paths: request.open_paths.into_iter().map(PathBuf::from).collect(),
        deadline,
        status: Arc::clone(&status),
        accepted_tx,
    })
    .map_err(|_| "Edolview UI is no longer receiving file-open requests".to_string())?;

    let accepted = loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if stop.load(Ordering::Relaxed) || remaining.is_zero() {
            break false;
        }
        match accepted_rx.recv_timeout(remaining.min(Duration::from_millis(50))) {
            Ok(()) => break true,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break false,
        }
    };
    if !accepted {
        *status.lock().unwrap() = OpenStatus::Cancelled;
        return Err("Edolview UI did not accept the file-open request before its deadline".to_string());
    }

    let response = ControlOpenResponse {
        protocol_version: CONTROL_PROTOCOL_VERSION,
        instance_id: instance_id.to_string(),
    };
    let body = serde_json::to_vec(&response).map_err(|e| format!("Failed to serialize control response: {e}"))?;
    stream
        .write_all(&body)
        .map_err(|e| format!("Failed to acknowledge file-open request: {e}"))
}

fn send_request(record: &WindowRecord, paths: &[PathBuf], forward_deadline: Instant) -> Result<(), String> {
    let timeout = CONTROL_REQUEST_TIMEOUT.min(forward_deadline.saturating_duration_since(Instant::now()));
    if timeout.is_zero() {
        return Err("File-open forwarding deadline expired".to_string());
    }
    let deadline = Instant::now() + timeout;
    let request = ControlOpenRequest {
        protocol_version: CONTROL_PROTOCOL_VERSION,
        instance_id: record.instance_id.clone(),
        expires_at_ms: now_millis()? + timeout.as_millis(),
        open_paths: paths.iter().map(|path| path.to_string_lossy().to_string()).collect(),
    };
    let body = serde_json::to_vec(&request).map_err(|e| format!("Failed to serialize control request: {e}"))?;

    let addr: SocketAddr = record
        .control_addr
        .parse()
        .map_err(|e| format!("Invalid control address: {e}"))?;
    if !addr.ip().is_loopback() {
        return Err("File-open control address must be local".to_string());
    }
    let mut stream = TcpStream::connect_timeout(&addr, CONTROL_CONNECT_TIMEOUT.min(timeout))
        .map_err(|e| format!("Failed to connect to {addr}: {e}"))?;
    stream
        .set_write_timeout(Some(remaining_timeout(deadline)?.min(CONTROL_IO_TIMEOUT)))
        .map_err(|e| format!("Failed to set control stream timeout: {e}"))?;
    stream
        .write_all(&body)
        .map_err(|e| format!("Failed to send control request: {e}"))?;
    // EOF frames the request while leaving the read half open for the reply.
    stream
        .shutdown(Shutdown::Write)
        .map_err(|e| format!("Failed to finish control request: {e}"))?;
    stream
        .set_read_timeout(Some(remaining_timeout(deadline)?))
        .map_err(|e| format!("Failed to set response timeout: {e}"))?;
    let response: ControlOpenResponse = read_message(&mut stream, deadline)?;
    if response.protocol_version != CONTROL_PROTOCOL_VERSION || response.instance_id != record.instance_id {
        return Err("File-open acknowledgement came from a different protocol or instance".to_string());
    }
    Ok(())
}

fn remaining_timeout(deadline: Instant) -> Result<Duration, String> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        Err("File-open request deadline expired".to_string())
    } else {
        Ok(remaining)
    }
}

fn registry_path() -> PathBuf {
    crate::util::path_ext::app_config_dir().join("window-registry.json")
}

fn load_registry() -> Result<WindowRegistry, String> {
    let path = registry_path();
    if !path.exists() {
        return Ok(WindowRegistry::default());
    }

    let body =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read window registry '{}': {e}", path.display()))?;
    serde_json::from_str(&body).map_err(|e| format!("Failed to parse window registry '{}': {e}", path.display()))
}

fn save_registry(registry: &WindowRegistry) -> Result<(), String> {
    let path = registry_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create control directory '{}': {e}", parent.display()))?;
    }

    let temp_path = path.with_extension("json.tmp");
    let body =
        serde_json::to_string_pretty(registry).map_err(|e| format!("Failed to serialize window registry: {e}"))?;
    fs::write(&temp_path, body)
        .map_err(|e| format!("Failed to write temporary window registry '{}': {e}", temp_path.display()))?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove previous window registry '{}': {e}", path.display()))?;
    }
    fs::rename(&temp_path, &path).map_err(|e| {
        format!(
            "Failed to replace window registry '{}' with '{}': {e}",
            temp_path.display(),
            path.display()
        )
    })
}

fn now_millis() -> Result<u128, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .map_err(|e| format!("Failed to compute current time: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::concurrency::mpsc_with_notify;

    fn test_record(listener: &TcpListener) -> WindowRecord {
        WindowRecord {
            instance_id: "test-window".to_string(),
            pid: u32::MAX,
            control_addr: listener.local_addr().unwrap().to_string(),
            last_active_ms: 1,
        }
    }

    fn request(instance_id: &str, timeout: Duration) -> ControlOpenRequest {
        ControlOpenRequest {
            protocol_version: CONTROL_PROTOCOL_VERSION,
            instance_id: instance_id.to_string(),
            expires_at_ms: now_millis().unwrap() + timeout.as_millis(),
            open_paths: vec!["test.png".to_string()],
        }
    }

    fn send(record: &WindowRecord) -> Result<(), String> {
        send_request(record, &[PathBuf::from("test.png")], Instant::now() + CONTROL_FORWARD_TIMEOUT)
    }

    #[test]
    fn forwarding_waits_for_ui_acceptance() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let instance_id = record.instance_id.clone();
        let (tx, rx, _nx) = mpsc_with_notify();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle_request(&mut stream, &instance_id, &tx, &AtomicBool::new(false))
        });
        let (result_tx, result_rx) = mpsc::channel();
        let sender = thread::spawn(move || result_tx.send(send(&record)).unwrap());
        let event = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(result_rx.try_recv().is_err(), "a socket write alone must not count as success");
        assert_eq!(event.accept().unwrap(), vec![PathBuf::from("test.png")]);
        assert!(result_rx.recv_timeout(Duration::from_secs(1)).unwrap().is_ok());
        assert!(server.join().unwrap().is_ok());
        sender.join().unwrap();
    }

    #[test]
    fn windows_nonblocking_listener_reads_fragmented_requests_until_eof() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let mut client = TcpStream::connect(&record.control_addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        let (mut stream, _) = listener.accept().unwrap();
        let (tx, rx, _nx) = mpsc_with_notify();
        let server = thread::spawn(move || handle_request(&mut stream, "test-window", &tx, &AtomicBool::new(false)));
        let body = serde_json::to_vec(&request("test-window", CONTROL_REQUEST_TIMEOUT)).unwrap();
        let midpoint = body.len() / 2;
        client.write_all(&body[..midpoint]).unwrap();
        assert!(rx.recv_timeout(Duration::from_millis(30)).is_err());
        client.write_all(&body[midpoint..]).unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        assert!(rx.recv_timeout(Duration::from_secs(1)).unwrap().accept().is_some());
        let reply: ControlOpenResponse = read_message(&mut client, Instant::now() + CONTROL_REQUEST_TIMEOUT).unwrap();
        assert_eq!(reply.instance_id, "test-window");
        assert!(server.join().unwrap().is_ok());
    }

    #[test]
    fn requests_for_a_reused_port_do_not_reach_the_wrong_ui() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let (tx, rx, _nx) = mpsc_with_notify();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle_request(&mut stream, "different-window", &tx, &AtomicBool::new(false))
        });
        assert!(send(&record).is_err());
        assert!(server.join().unwrap().is_err());
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn disconnected_ui_does_not_acknowledge_requests() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let (tx, rx, _nx) = mpsc_with_notify();
        drop(rx);
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle_request(&mut stream, "test-window", &tx, &AtomicBool::new(false))
        });
        assert!(send(&record).is_err());
        assert!(server.join().unwrap().is_err());
    }

    #[test]
    fn timed_out_requests_cannot_be_opened_later_by_the_ui() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let (tx, rx, _nx) = mpsc_with_notify();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle_request(&mut stream, "test-window", &tx, &AtomicBool::new(false))
        });
        client
            .write_all(&serde_json::to_vec(&request("test-window", Duration::from_millis(200))).unwrap())
            .unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        let event = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(server.join().unwrap().is_err());
        assert!(event.accept().is_none());
        let mut response = Vec::new();
        client.read_to_end(&mut response).unwrap();
        assert!(response.is_empty());
    }

    #[test]
    fn shutdown_cancels_requests_waiting_for_the_ui() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let (tx, rx, _nx) = mpsc_with_notify();
        let stop = Arc::new(AtomicBool::new(false));
        let server_stop = Arc::clone(&stop);
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle_request(&mut stream, "test-window", &tx, &server_stop)
        });
        let sender = thread::spawn(move || send(&record));
        let event = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        stop.store(true, Ordering::Relaxed);
        assert!(server.join().unwrap().is_err());
        assert!(event.accept().is_none());
        assert!(sender.join().unwrap().is_err());
    }

    #[test]
    fn unresponsive_server_cannot_prevent_a_new_window_indefinitely() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let (release_tx, release_rx) = mpsc::channel();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _: ControlOpenRequest = read_message(&mut stream, Instant::now() + CONTROL_REQUEST_TIMEOUT).unwrap();
            release_rx.recv().unwrap();
        });
        let started = Instant::now();
        assert!(send(&record).is_err());
        assert!(started.elapsed() < CONTROL_FORWARD_TIMEOUT);
        release_tx.send(()).unwrap();
        server.join().unwrap();
    }

    #[test]
    fn failed_target_is_reported_even_when_another_window_accepts() {
        let wrong_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut wrong_record = test_record(&wrong_listener);
        wrong_record.instance_id = "stale-window".to_string();
        let wrong_server = thread::spawn(move || {
            let (mut stream, _) = wrong_listener.accept().unwrap();
            let _: ControlOpenRequest = read_message(&mut stream, Instant::now() + CONTROL_REQUEST_TIMEOUT).unwrap();
            // Accept the TCP write without being an Edolview UI.
        });
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let record = test_record(&listener);
        let (tx, rx, _nx) = mpsc_with_notify();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            handle_request(&mut stream, "test-window", &tx, &AtomicBool::new(false))
        });
        let ui = thread::spawn(move || assert!(rx.recv_timeout(Duration::from_secs(1)).unwrap().accept().is_some()));
        let (forwarded, failed) = forward_to_windows(&[wrong_record, record], &[PathBuf::from("test.png")]);
        assert!(forwarded);
        assert_eq!(failed, vec!["stale-window".to_string()]);
        assert!(server.join().unwrap().is_ok());
        ui.join().unwrap();
        wrong_server.join().unwrap();
    }
}
