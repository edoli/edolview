use std::{
    num::NonZeroUsize,
    sync::{mpsc, Arc, OnceLock},
    thread,
};

use color_eyre::eyre::{eyre, Result};
use lru::LruCache;

use super::{gpu_compute, Image, ImageData, Recti};

pub const HISTOGRAM_BINS: usize = 256;
pub const HISTOGRAM_STRIDE: usize = HISTOGRAM_BINS + 3;
const CUSTOM_HISTOGRAM_CACHE_CAPACITY: usize = 8;

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

/// Fixed ranges live with their image, independently of custom-range eviction.
#[derive(Default)]
pub(super) struct ImageHistogramCache {
    unit: OnceLock<Arc<Histogram>>,
    data: OnceLock<Arc<Histogram>>,
}

impl ImageHistogramCache {
    fn slot(&self, range: HistogramRange) -> Option<&OnceLock<Arc<Histogram>>> {
        match range {
            HistogramRange::Unit => Some(&self.unit),
            HistogramRange::Data => Some(&self.data),
            HistogramRange::Custom(_) => None,
        }
    }

    fn get(&self, range: HistogramRange) -> Option<Arc<Histogram>> {
        self.slot(range)?.get().map(Arc::clone)
    }

    fn insert(&self, range: HistogramRange, histogram: Arc<Histogram>) -> Arc<Histogram> {
        match self.slot(range) {
            Some(slot) => Arc::clone(slot.get_or_init(|| histogram)),
            None => histogram,
        }
    }
}

/// One GPU readback at a time. New requests replace stale work after it completes;
/// cached results contain only small histograms, never copies of image pixels.
pub struct HistogramWorker {
    pending: Option<(RequestKey, HistogramRange, mpsc::Receiver<Result<Arc<Histogram>>>)>,
    custom_cache: LruCache<RequestKey, Arc<Histogram>>,
    failed: Option<RequestKey>,
}

impl Default for HistogramWorker {
    fn default() -> Self {
        Self {
            pending: None,
            custom_cache: LruCache::new(NonZeroUsize::new(CUSTOM_HISTOGRAM_CACHE_CAPACITY).unwrap()),
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
        if let Some((pending_key, pending_range, rx)) = &self.pending {
            let completed = match rx.try_recv() {
                Ok(result) => Some(result),
                Err(mpsc::TryRecvError::Empty) => None,
                Err(mpsc::TryRecvError::Disconnected) => Some(Err(eyre!("Histogram worker stopped unexpectedly"))),
            };
            if let Some(result) = completed {
                let completed_key = *pending_key;
                let completed_range = *pending_range;
                self.pending = None;
                match result {
                    Ok(histogram) if matches!(completed_range, HistogramRange::Custom(_)) => {
                        self.custom_cache.put(completed_key, histogram);
                    }
                    Ok(_) => {} // The worker stored fixed ranges on their image before sending.
                    Err(error) if completed_key == key => {
                        self.failed = Some(key);
                        return Err(error);
                    }
                    Err(error) => eprintln!("Stale histogram request failed: {error}"),
                }
            }
        }
        let cached = match range {
            HistogramRange::Custom(_) => self.custom_cache.get(&key).map(Arc::clone),
            _ => image.histogram_cache().get(range),
        };
        if cached.is_some() {
            return Ok(cached);
        }
        if self.failed.is_some_and(|failed| failed != key) {
            self.failed = None;
        }
        if self.pending.is_none() && self.failed != Some(key) {
            let image = image.clone();
            let ctx = ctx.clone();
            let (tx, rx) = mpsc::channel();
            self.pending = Some((key, range, rx));
            thread::spawn(move || {
                let result = (|| {
                    if let Some(histogram) = image.histogram_cache().get(range) {
                        return Ok(histogram);
                    }
                    let bounds = range.resolve(&image)?;
                    let texture = image.gpu_texture()?;
                    let histogram = Arc::new(gpu_compute()?.histogram_range(&texture, bounds)?);
                    Ok(image.histogram_cache().insert(range, histogram))
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
    use crate::model::PixelType;

    fn sample_histogram() -> Arc<Histogram> {
        Arc::new(Histogram {
            bins: vec![vec![1; HISTOGRAM_BINS]],
            excluded: vec![[0; 3]],
            range: [0.0, 1.0],
            pixels: HISTOGRAM_BINS as u64,
        })
    }

    #[test]
    fn fixed_histograms_survive_navigation_and_worker_recreation_until_image_is_dropped() {
        let images: Vec<_> = (0..12).map(|_| ImageData::empty(PixelType::F32)).collect();
        let mut expected = Vec::new();
        for image in &images {
            let unit = image.histogram_cache().insert(HistogramRange::Unit, sample_histogram());
            let data = image.histogram_cache().insert(HistogramRange::Data, sample_histogram());
            expected.push([Arc::downgrade(&unit), Arc::downgrade(&data)]);
        }
        let ctx = egui::Context::default();
        for _ in 0..2 {
            let mut worker = HistogramWorker::default();
            for (image, cached) in images.iter().zip(&expected).rev() {
                for (range, expected) in [HistogramRange::Unit, HistogramRange::Data].into_iter().zip(cached) {
                    let histogram = worker.request(image, range, &ctx).unwrap().expect("fixed cache hit");
                    assert!(Arc::ptr_eq(&histogram, &expected.upgrade().unwrap()));
                    assert!(worker.pending.is_none(), "a cache hit must not start GPU work");
                }
            }
            assert!(worker.custom_cache.is_empty());
        }
        drop(images);
        assert!(expected.iter().flatten().all(|histogram| histogram.upgrade().is_none()));
    }

    #[test]
    fn custom_histograms_keep_eight_entries_across_images_and_evict_the_least_recently_used() {
        let images: Vec<_> = (0..9).map(|_| ImageData::empty(PixelType::F32)).collect();
        let mut worker = HistogramWorker::default();
        let ctx = egui::Context::default();
        let range = HistogramRange::Custom([0.0, 1.0]);
        let complete = |worker: &mut HistogramWorker, image: &ImageData| {
            // Deliver a completed GPU job through the actual result channel, without a GPU fixture.
            let histogram = sample_histogram();
            let weak = Arc::downgrade(&histogram);
            let (tx, rx) = mpsc::channel();
            assert!(tx.send(Ok(histogram)).is_ok());
            worker.pending = Some(((image.id(), range.key()), range, rx));
            assert!(worker.request(image, range, &ctx).unwrap().is_some());
            assert!(image.histogram_cache().get(range).is_none());
            weak
        };
        let mut cached = Vec::new();
        for image in images.iter().take(8) {
            cached.push(complete(&mut worker, image));
        }
        // A hit on the oldest entry protects it; the next-oldest entry should be evicted.
        assert!(worker.request(&images[0], range, &ctx).unwrap().is_some());
        cached.push(complete(&mut worker, &images[8]));
        assert_eq!(worker.custom_cache.len(), 8);
        assert!(cached[0].upgrade().is_some());
        assert!(cached[1].upgrade().is_none());
        assert!(cached[2..].iter().all(|histogram| histogram.upgrade().is_some()));
    }

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
