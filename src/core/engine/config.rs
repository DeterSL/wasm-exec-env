#[derive(Clone)]
pub struct DeterSLEngineConfig {
    pub LRUCacheCapacity: usize
}

impl DeterSLEngineConfig {
    pub fn default() -> Self {
        Self { LRUCacheCapacity: 10 }
    }

    pub fn with_cache_capacity(mut self, cache_capacity: usize) -> Self {
        self.LRUCacheCapacity = cache_capacity;
        self
    }
}
