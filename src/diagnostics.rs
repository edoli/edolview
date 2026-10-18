//! Local-build diagnostics. Keep application stderr visible while saving it to
//! a per-process file; write panic diagnostics synchronously before aborting.
//! The entire module, including the stderr macro, is excluded from release-dist.

use std::fmt;

// Defined before the other modules so their existing eprintln! calls also reach
// the log. Explicit std::eprintln! below avoids recursive macro expansion.
macro_rules! eprintln {
    () => {
        $crate::diagnostics::stderr(format_args!(""))
    };
    ($($arg:tt)*) => {
        $crate::diagnostics::stderr(format_args!($($arg)*))
    };
}

pub fn stderr(message: fmt::Arguments<'_>) {
    local::write(message);
    std::eprintln!("{message}");
}

pub fn main() -> color_eyre::eyre::Result<()> {
    if let Err(error) = local::init() {
        eprintln!("Failed to initialize file logging: {error}");
    }
    let result = crate::run();
    if let Err(error) = &result {
        // Returning the error already prints it through Result's Termination
        // implementation. Save it to the file without duplicating stderr.
        local::write(format_args!("Application failed: {error:?}"));
    }
    result
}

pub fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Capture independently of RUST_BACKTRACE and color-eyre's console
        // formatting. File writes finish before panic=abort terminates us.
        let thread = std::thread::current();
        local::write(format_args!("PANIC in thread {:?}: {info}", thread.name()));
        let backtrace = std::backtrace::Backtrace::force_capture();
        local::write(format_args!("{backtrace}"));
        previous(info);
    }));
}

mod local {
    use std::{
        fmt,
        fs::{self, File, OpenOptions},
        io::{self, Write},
        path::Path,
        sync::{
            atomic::{AtomicBool, Ordering},
            Mutex, OnceLock,
        },
        time::{SystemTime, UNIX_EPOCH},
    };

    static FILE: OnceLock<Mutex<File>> = OnceLock::new();
    static WRITE_ERROR_REPORTED: AtomicBool = AtomicBool::new(false);
    static LOGGER: Logger = Logger;

    fn timestamp() -> std::time::Duration {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()
    }

    fn create_log(directory: &Path) -> io::Result<(std::path::PathBuf, File)> {
        fs::create_dir_all(directory)?;
        let path = directory.join(format!("edolview-{}-{}.log", timestamp().as_nanos(), std::process::id()));
        let file = OpenOptions::new().write(true).create_new(true).open(&path)?;
        Ok((path, file))
    }

    pub(super) fn init() -> io::Result<()> {
        let executable = std::env::current_exe()?;
        let directory = executable
            .parent()
            .ok_or_else(|| io::Error::other("Executable has no parent directory"))?
            .join("logs");
        let (path, file) = create_log(&directory)?;
        FILE.set(Mutex::new(file))
            .map_err(|_| io::Error::other("File logging is already initialized"))?;
        if let Err(error) = log::set_logger(&LOGGER) {
            super::stderr(format_args!("Failed to capture library logs: {error}"));
        } else {
            log::set_max_level(log::LevelFilter::Warn);
        }
        super::stderr(format_args!("Log file: {}", path.display()));
        write(format_args!(
            "edolview {} profile={} pid={} os={} arch={}",
            env!("EDOLVIEW_BUILD_VERSION"),
            env!("EDOLVIEW_BUILD_PROFILE"),
            std::process::id(),
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
        write(format_args!("Executable: {}", executable.display()));
        Ok(())
    }

    // File is unbuffered: each record is written before returning, including
    // messages from background workers. Do not hold this lock while invoking
    // a panic hook, symbolizing a backtrace, or writing to stderr.
    pub(super) fn write(message: fmt::Arguments<'_>) {
        let Some(file) = FILE.get() else { return };
        let time = timestamp();
        let result = {
            let mut file = file.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            writeln!(file, "[{}.{:03} UTC epoch] {message}", time.as_secs(), time.subsec_millis())
        };
        if let Err(error) = result {
            if !WRITE_ERROR_REPORTED.swap(true, Ordering::Relaxed) {
                std::eprintln!("Failed to write diagnostic log: {error}");
            }
        }
    }

    struct Logger;

    impl log::Log for Logger {
        fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
            metadata.level() <= log::Level::Warn
        }

        fn log(&self, record: &log::Record<'_>) {
            if self.enabled(record.metadata()) {
                super::stderr(format_args!("{} {}: {}", record.level(), record.target(), record.args()));
            }
        }

        fn flush(&self) {}
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn panic_diagnostics_survive_process_exit() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "diagnostics::local::tests::diagnostic_child", "--nocapture"])
                .env("EDOLVIEW_DIAGNOSTIC_TEST_CHILD", "1")
                .env("RUST_BACKTRACE", "0")
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(23));
            let stderr = String::from_utf8(output.stderr).unwrap();
            assert!(stderr.contains("diagnostic stderr"));
            assert!(stderr.contains("diagnostic warning"));
            assert!(stderr.contains("diagnostic panic"));
            let path = stderr.lines().find_map(|line| line.strip_prefix("Log file: ")).unwrap();
            let contents = fs::read_to_string(path).unwrap();
            assert!(contents.contains("diagnostic stderr"));
            assert!(contents.contains("diagnostic warning"));
            assert!(contents.contains("PANIC in thread Some(\"diagnostic-worker\")"));
            assert!(contents.contains("diagnostic panic"));
            assert!(contents.contains("diagnostics::install_panic_hook"));
            fs::remove_file(path).unwrap();
        }

        #[test]
        fn diagnostic_child() {
            if std::env::var_os("EDOLVIEW_DIAGNOSTIC_TEST_CHILD").is_none() {
                return;
            }
            super::init().unwrap();
            color_eyre::install().unwrap();
            super::super::install_panic_hook();
            eprintln!("diagnostic stderr");
            log::warn!("diagnostic warning");
            let worker = std::thread::Builder::new()
                .name("diagnostic-worker".into())
                .spawn(|| panic!("diagnostic panic"))
                .unwrap();
            assert!(worker.join().is_err());
            // Bypass normal cleanup to check the records were already written.
            std::process::exit(23);
        }

        #[test]
        fn concurrent_instances_keep_separate_logs() {
            let directory = std::env::current_exe().unwrap().parent().unwrap().join("logs");
            let (first_path, mut first) = create_log(&directory).unwrap();
            let (second_path, mut second) = create_log(&directory).unwrap();
            writeln!(first, "first instance").unwrap();
            writeln!(second, "second instance").unwrap();
            assert_ne!(first_path, second_path);
            assert_eq!(fs::read_to_string(&first_path).unwrap(), "first instance\n");
            assert_eq!(fs::read_to_string(&second_path).unwrap(), "second instance\n");
            drop((first, second));
            fs::remove_file(first_path).unwrap();
            fs::remove_file(second_path).unwrap();
        }
    }
}
