//! Byte-budgeted GPU retention. Display/compute owners can outlive an eviction;
//! this budget counts the texture payloads retained by the cache itself.

use lru::LruCache;

pub(super) const GPU_CACHE_BUDGET: u64 = 512 * 1024 * 1024;

struct Entry<T> {
    value: T,
    bytes: u64,
}

pub(super) struct GpuCache<T> {
    entries: LruCache<u64, Entry<T>>,
    bytes: u64,
    budget: u64,
}

impl<T> GpuCache<T> {
    pub fn new(budget: u64) -> Self {
        Self {
            entries: LruCache::unbounded(),
            bytes: 0,
            budget,
        }
    }

    pub fn retain(&mut self, id: u64, bytes: u64, value: T) {
        if bytes == 0 || bytes > self.budget {
            return;
        }
        self.remove(id);
        while self.bytes > self.budget - bytes {
            let (_, oldest) = self.entries.pop_lru().expect("nonempty cache exceeds budget");
            self.bytes -= oldest.bytes;
        }
        self.entries.put(id, Entry { value, bytes });
        self.bytes += bytes;
    }

    pub fn remove(&mut self, id: u64) {
        if let Some(entry) = self.entries.pop(&id) {
            self.bytes -= entry.bytes;
        }
    }
}

pub(super) fn payload_bytes(format: wgpu::TextureFormat, width: u32, height: u32, layers: u32) -> Option<u64> {
    let bytes = u64::from(format.block_copy_size(None)?);
    let (block_width, block_height) = format.block_dimensions();
    Some(u64::from(width.div_ceil(block_width)) * u64::from(height.div_ceil(block_height)) * u64::from(layers) * bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn exact_512_mib_boundary_and_lru_eviction() {
        let mut cache = GpuCache::new(GPU_CACHE_BUDGET);
        let image_bytes = payload_bytes(wgpu::TextureFormat::Rgba8Unorm, 8192, 8192, 1).unwrap();
        assert_eq!(image_bytes, 256 * 1024 * 1024);
        let a = Arc::new(());
        let b = Arc::new(());
        let b_weak = Arc::downgrade(&b);
        cache.retain(1, image_bytes, Arc::clone(&a));
        cache.retain(2, image_bytes, b);
        assert_eq!(cache.bytes, GPU_CACHE_BUDGET);
        cache.retain(1, image_bytes, a);
        cache.retain(3, image_bytes, Arc::new(()));
        assert!(cache.entries.contains(&1) && cache.entries.contains(&3));
        assert!(b_weak.upgrade().is_none(), "least recently used texture is released");
        assert_eq!(cache.bytes, GPU_CACHE_BUDGET);
        cache.remove(1);
        assert_eq!(cache.bytes, image_bytes);
    }

    #[test]
    fn mixed_formats_replacement_and_oversize_bypass() {
        let mut cache = GpuCache::new(GPU_CACHE_BUDGET);
        let u8_bytes = payload_bytes(wgpu::TextureFormat::Rgba8Unorm, 4096, 4096, 1).unwrap();
        let f32_bytes = payload_bytes(wgpu::TextureFormat::Rgba32Float, 4096, 4096, 1).unwrap();
        cache.retain(1, u8_bytes, ());
        cache.retain(2, f32_bytes, ());
        cache.retain(1, f32_bytes, ());
        assert_eq!(cache.bytes, GPU_CACHE_BUDGET, "replacement is not double counted");
        let oversized = payload_bytes(wgpu::TextureFormat::Rgba32Float, 8192, 8192, 1).unwrap();
        assert_eq!(oversized, 1024 * 1024 * 1024);
        cache.retain(3, oversized, ());
        assert!(!cache.entries.contains(&3));
        assert_eq!(
            cache.bytes, GPU_CACHE_BUDGET,
            "an oversized display does not flush useful cached images"
        );
        cache.retain(4, u8_bytes, ());
        assert!(!cache.entries.contains(&2), "replacement updates recency");
        assert_eq!(cache.bytes, f32_bytes + u8_bytes);
    }

    #[test]
    fn byte_accounting_uses_storage_format_not_original_dtype() {
        use wgpu::TextureFormat::*;
        assert_eq!(payload_bytes(R8Unorm, 1024, 1024, 1), Some(1024 * 1024));
        assert_eq!(payload_bytes(Rg8Unorm, 1024, 1024, 1), Some(2 * 1024 * 1024));
        assert_eq!(payload_bytes(R32Float, 1024, 1024, 1), Some(4 * 1024 * 1024));
        assert_eq!(payload_bytes(Rg32Float, 1024, 1024, 1), Some(8 * 1024 * 1024));
        assert_eq!(payload_bytes(Rgba32Float, 7680, 4320, 1), Some(530_841_600));
    }

    #[test]
    fn eviction_does_not_invalidate_a_display_owner() {
        let mut cache = GpuCache::new(1);
        let display = Arc::new(());
        let weak = Arc::downgrade(&display);
        cache.retain(1, 1, Arc::clone(&display));
        cache.retain(2, 1, Arc::new(()));
        assert!(weak.upgrade().is_some());
        drop(display);
        assert!(weak.upgrade().is_none());
        cache.remove(2);
        assert_eq!(cache.bytes, 0);
    }
}
