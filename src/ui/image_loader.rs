use std::{collections::HashSet, path::PathBuf, sync::mpsc, thread};

use clipboard_rs::{Clipboard, ClipboardContext, ContentFormat};
use color_eyre::eyre::{eyre, Report, Result, WrapErr};

use crate::{
    model::{FileAsset, ImageData},
    util::path_ext::clipboard_text_to_path,
};

pub(super) enum ImageLoadRequest {
    Files(Vec<PathBuf>),
    Clipboard,
}

enum PendingRequest {
    Files(Vec<PathBuf>),
    Clipboard(mpsc::Receiver<Result<ClipboardInput>>),
}

pub(super) enum ImageLoadResult {
    FileLoaded {
        path: PathBuf,
        hash: String,
        image: ImageData,
    },
    FileReused {
        path: PathBuf,
        hash: String,
    },
    ClipboardLoaded(ImageData),
    Failed {
        path: Option<PathBuf>,
        error: Report,
    },
}

/// All image I/O runs on a sequential worker; only completed results reach the UI.
#[derive(Default)]
pub(super) struct ImageLoader {
    pending: Vec<PendingRequest>,
    rx: Option<mpsc::Receiver<ImageLoadResult>>,
}

impl ImageLoader {
    pub fn enqueue(&mut self, request: ImageLoadRequest) {
        let request = match request {
            ImageLoadRequest::Files(paths) => PendingRequest::Files(paths),
            ImageLoadRequest::Clipboard => {
                // Capture now, even if an earlier file or URL is still loading.
                // Otherwise a later Copy could change the contents of this paste.
                let (tx, rx) = mpsc::channel();
                thread::spawn(move || {
                    let _ = tx.send(read_clipboard());
                });
                PendingRequest::Clipboard(rx)
            }
        };
        self.pending.push(request);
    }

    pub fn is_loading(&self) -> bool {
        self.rx.is_some() || !self.pending.is_empty()
    }

    pub fn start(&mut self, ctx: &egui::Context, cached_files: impl Iterator<Item = String>) {
        if self.rx.is_some() || self.pending.is_empty() {
            return;
        }
        let mut cached_files: HashSet<_> = cached_files.collect();
        let requests = std::mem::take(&mut self.pending);
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let mut emit = |result| {
                let sent = tx.send(result).is_ok();
                ctx.request_repaint_of(egui::ViewportId::ROOT);
                sent
            };
            for request in requests {
                let keep_loading = match request {
                    PendingRequest::Files(paths) => load_files(paths, &mut cached_files, &mut emit),
                    PendingRequest::Clipboard(snapshot) => match snapshot
                        .recv()
                        .unwrap_or_else(|_| Err(eyre!("Clipboard reader stopped before returning an image")))
                    {
                        Ok(ClipboardInput::Files(paths)) => load_files(paths, &mut cached_files, &mut emit),
                        Ok(ClipboardInput::Image(image)) => emit(ImageLoadResult::ClipboardLoaded(image)),
                        Ok(ClipboardInput::Url(url)) => emit(
                            ImageData::load_from_url(&url)
                                .map(ImageLoadResult::ClipboardLoaded)
                                .unwrap_or_else(|error| ImageLoadResult::Failed { path: None, error }),
                        ),
                        Err(error) => emit(ImageLoadResult::Failed { path: None, error }),
                    },
                };
                if !keep_loading {
                    break;
                }
            }
            // Wake the UI when the receiver disconnects, even after an empty batch.
            drop(tx);
            ctx.request_repaint_of(egui::ViewportId::ROOT);
        });
    }

    pub fn poll(&mut self) -> Option<ImageLoadResult> {
        match self.rx.as_ref()?.try_recv() {
            Ok(result) => Some(result),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.rx = None;
                None
            }
        }
    }
}

fn load_files(
    paths: Vec<PathBuf>,
    cached_files: &mut HashSet<String>,
    emit: &mut impl FnMut(ImageLoadResult) -> bool,
) -> bool {
    for path in paths {
        let result = (|| {
            let hash = FileAsset::hash_from_path(&path)?;
            if cached_files.contains(&hash) {
                return Ok(ImageLoadResult::FileReused {
                    path: path.clone(),
                    hash,
                });
            }
            let image = ImageData::load_from_path(&path)?;
            cached_files.insert(hash.clone());
            Ok(ImageLoadResult::FileLoaded {
                path: path.clone(),
                hash,
                image,
            })
        })()
        .unwrap_or_else(|error| ImageLoadResult::Failed {
            path: Some(path),
            error,
        });
        if !emit(result) {
            return false;
        }
    }
    true
}

enum ClipboardInput {
    Files(Vec<PathBuf>),
    Image(ImageData),
    Url(String),
}

fn read_clipboard() -> Result<ClipboardInput> {
    // Prefer original files over thumbnails, and local paths over URL downloads.
    // Create clipboard handles on this worker instead of moving platform handles
    // across threads or opening the clipboard on the egui thread.
    let text = if let Ok(clipboard) = ClipboardContext::new() {
        if clipboard.has(ContentFormat::Files) {
            let files = clipboard
                .get_files()
                .map_err(|error| eyre!("Failed to read clipboard files: {error}"))?;
            if !files.is_empty() {
                return Ok(ClipboardInput::Files(files.into_iter().map(PathBuf::from).collect()));
            }
        }
        clipboard.get_text().ok()
    } else {
        None
    };
    let text = text.or_else(|| arboard::Clipboard::new().ok()?.get_text().ok());
    if let Some(path) = text.as_deref().and_then(clipboard_text_to_path) {
        return Ok(ClipboardInput::Files(vec![path]));
    }
    if let Ok(image) = ImageData::load_from_clipboard() {
        return Ok(ClipboardInput::Image(image));
    }
    let text = text.ok_or_else(|| eyre!("Clipboard does not contain an image, files, or an image URL"))?;
    let url = url::Url::parse(text.trim()).wrap_err("Clipboard text is not an image path or URL")?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(eyre!("Clipboard image URL must use HTTP or HTTPS"));
    }
    Ok(ClipboardInput::Url(url.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Image;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        time::{Duration, Instant},
    };

    #[test]
    fn worker_preserves_order_recovers_from_errors_and_reuses_cached_files() {
        static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/image-loader-tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!(
            "{}-{}.ppm",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, b"P6\n2 1\n255\n\xff\x00\x00\x00\xff\x00").unwrap();
        let missing = path.with_extension("missing.png");
        let ctx = egui::Context::default();
        let mut loader = ImageLoader::default();
        let (snapshot_tx, snapshot_rx) = mpsc::channel();
        loader.pending.push(PendingRequest::Clipboard(snapshot_rx));
        loader.start(&ctx, std::iter::empty());
        // Starting the loader must return while clipboard input is still pending.
        snapshot_tx
            .send(Ok(ClipboardInput::Files(vec![missing.clone(), path.clone(), path.clone()])))
            .unwrap();
        // An additional request must wait for this batch, without replacing it.
        loader.enqueue(ImageLoadRequest::Files(vec![path.clone()]));
        let results = drain_batch(&mut loader);
        assert_eq!(results.len(), 3);
        assert!(matches!(&results[0], ImageLoadResult::Failed { path: Some(failed), .. } if failed == &missing));
        let hash = match &results[1] {
            ImageLoadResult::FileLoaded {
                path: loaded,
                hash,
                image,
            } => {
                assert_eq!(loaded, &path);
                assert_eq!((image.spec().width, image.spec().height), (2, 1));
                hash.clone()
            }
            _ => panic!("Expected a decoded file after the missing file error"),
        };
        assert!(
            matches!(&results[2], ImageLoadResult::FileReused { path: reused, hash: cached } if reused == &path && cached == &hash)
        );
        assert!(loader.is_loading());
        loader.start(&ctx, std::iter::once(hash.clone()));
        let results = drain_batch(&mut loader);
        assert_eq!(results.len(), 1);
        assert!(matches!(&results[0], ImageLoadResult::FileReused { hash: cached, .. } if cached == &hash));
        assert!(!loader.is_loading());
        std::fs::remove_file(path).unwrap();
    }

    fn drain_batch(loader: &mut ImageLoader) -> Vec<ImageLoadResult> {
        let deadline = Instant::now() + Duration::from_secs(10);
        let mut results = Vec::new();
        while loader.rx.is_some() {
            assert!(Instant::now() < deadline, "Image worker did not complete");
            if let Some(result) = loader.poll() {
                results.push(result);
            } else {
                thread::sleep(Duration::from_millis(1));
            }
        }
        results
    }
}
