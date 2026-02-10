#[derive(Clone)]
pub struct DeterSLEngineConfig {
    pub lrucache_capacity: usize
}

impl DeterSLEngineConfig {
    pub fn default() -> Self {
        Self { lrucache_capacity: 10 }
    }

    #[allow(dead_code)]
    pub fn with_cache_capacity(mut self, cache_capacity: usize) -> Self {
        self.lrucache_capacity = cache_capacity;
        self
    }
}
