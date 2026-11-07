use std::{
    num::NonZeroUsize,
    sync::{mpsc, Arc},
    thread,
};

use color_eyre::eyre::{eyre, Result};
use lru::LruCache;

use super::{gpu_compute, Image, ImageData, Recti};

pub const HISTOGRAM_BINS: usize = 256;
pub const HISTOGRAM_STRIDE: usize = HISTOGRAM_BINS + 3;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum HistogramRange {
    #[default]
    Unit,
    Data,
    Custom([f32; 2]),
}

impl HistogramRange {
    fn key(self) -> [u32; 3] {
        match self {
            Self::Unit => [0, 0, 0],
            Self::Data => [1, 0, 0],
            Self::Custom([low, high]) => [2, low.to_bits(), high.to_bits()],
        }
    }

    pub fn resolve(self, image: &ImageData) -> Result<[f32; 2]> {
        match self {
            Self::Unit => Ok([0.0, 1.0]),
            Self::Custom([low, high]) if low.is_finite() && high.is_finite() && low < high => Ok([low, high]),
            Self::Custom(_) => Err(eyre!("Histogram minimum must be finite and smaller than maximum")),
            Self::Data => {
                let texture = image.gpu_texture()?;
                let (mins, maxs) = gpu_compute()?.minmax(&texture, Recti::ZERO)?;
                let low = mins.into_iter().filter(|v| v.is_finite()).fold(f32::INFINITY, f32::min);
                let high = maxs.into_iter().filter(|v| v.is_finite()).fold(f32::NEG_INFINITY, f32::max);
                // No finite samples: retain a useful axis while the invalid counts explain the empty plot.
                Ok(if low <= high { [low, high] } else { [0.0, 1.0] })
            }
        }
    }
}

pub struct Histogram {
    pub bins: Vec<Vec<u32>>,
    /// Counts below the range, above the range, and non-finite, per channel.
    pub excluded: Vec<[u32; 3]>,
    pub range: [f32; 2],
    pub pixels: u64,
}

impl Histogram {
    pub fn visible_max(&self, mask: &[bool]) -> u32 {
        self.bins
            .iter()
            .enumerate()
            .filter(|(i, _)| mask.get(*i).copied().unwrap_or(false))
            .flat_map(|(_, bins)| bins)
            .copied()
            .max()
            .unwrap_or(0)
    }

    pub fn bin_bounds(&self, bin: usize) -> [f64; 2] {
        let [low, high] = self.range.map(f64::from);
        let width = (high - low) / HISTOGRAM_BINS as f64;
        [
            low + bin as f64 * width,
            if bin + 1 == HISTOGRAM_BINS {
                high
            } else {
                low + (bin + 1) as f64 * width
            },
        ]
    }

    pub fn percent(&self, count: u32) -> f64 {
        if self.pixels == 0 {
            0.0
        } else {
            count as f64 * 100.0 / self.pixels as f64
        }
    }
}

type RequestKey = (u64, [u32; 3]);

/// One GPU readback at a time. New requests replace stale work after it completes;
/// cached results contain only small histograms, never copies of image pixels.
pub struct HistogramWorker {
    pending: Option<(RequestKey, mpsc::Receiver<Result<Arc<Histogram>>>)>,
    cache: LruCache<RequestKey, Arc<Histogram>>,
    failed: Option<RequestKey>,
}

impl Default for HistogramWorker {
    fn default() -> Self {
        Self {
            pending: None,
            cache: LruCache::new(NonZeroUsize::new(8).unwrap()),
            failed: None,
        }
    }
}

impl HistogramWorker {
    pub fn request(
        &mut self,
        image: &ImageData,
        range: HistogramRange,
        ctx: &egui::Context,
    ) -> Result<Option<Arc<Histogram>>> {
        let key = (image.id(), range.key());
        if let Some((pending_key, rx)) = &self.pending {
            let completed = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(eyre!("Histogram worker stopped unexpectedly"))),
            };
            if let Some(result) = completed {
                let completed_key = *pending_key;
                self.pending = None;
                match result {
                    Ok(histogram) => {
                        self.cache.put(completed_key, histogram);
                    }
                    Err(error) if completed_key == key => {
                        self.failed = Some(key);
                        return Err(error);
                    }
                    Err(error) => eprintln!("Stale histogram request failed: {error}"),
                }
            }
        }
        if let Some(histogram) = self.cache.get(&key) {
            return Ok(Some(Arc::clone(histogram)));
        }
        if self.failed.is_some_and(|failed| failed != key) {
            self.failed = None;
        }
        if self.pending.is_none() && self.failed != Some(key) {
            let image = image.clone();
            let ctx = ctx.clone();
            let (tx, rx) = mpsc::channel();
            self.pending = Some((key, rx));
            thread::spawn(move || {
                let result = (|| {
                    let range = range.resolve(&image)?;
                    let texture = image.gpu_texture()?;
                    gpu_compute()?.histogram_range(&texture, range).map(Arc::new)
                })();
                let _ = tx.send(result);
                ctx.request_repaint_of(egui::ViewportId::ROOT);
            });
        }
        Ok(None)
    }

    pub fn failed(&self, image: &ImageData, range: HistogramRange) -> bool {
        self.failed == Some((image.id(), range.key()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_alpha_does_not_flatten_rgb_and_empty_masks_have_no_peak() {
        let hist = Histogram {
            bins: vec![vec![4, 8], vec![2, 3], vec![1, 6], vec![0, 1024]],
            excluded: vec![[0; 3]; 4],
            range: [0.0, 1.0],
            pixels: 1024,
        };
        assert_eq!(hist.visible_max(&[true, true, true, false]), 8);
        assert_eq!(hist.visible_max(&[false, true, false, false]), 3);
        assert_eq!(hist.visible_max(&[false; 4]), 0);
        assert_eq!(hist.visible_max(&[true; 4]), 1024);
        assert_eq!(hist.percent(512), 50.0);
    }
}
