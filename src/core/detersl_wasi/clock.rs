pub struct DeterSLWallClock {

}

impl DeterSLWallClock {
    pub fn new() -> Self {
        Self {  }
    }
}

impl wasmtime_wasi::HostWallClock for DeterSLWallClock {
    fn now(&self) -> std::time::Duration {
        return std::time::Duration::new(0, 0);
    }

    fn resolution(&self) -> std::time::Duration {
        return std::time::Duration::new(0, 0);
    }
}

pub struct DeterSLMonotonicWallClock {

}

impl DeterSLMonotonicWallClock {
    pub fn new() -> Self {
        Self {  }
    }
}

impl wasmtime_wasi::HostMonotonicClock for DeterSLMonotonicWallClock {
    fn resolution(&self) -> u64 {
       0 
    }

    fn now(&self) -> u64 {
        0
    }
}
