use std::time::Duration;


pub struct DeterSLWallClock {
    initial: Duration,
}

impl DeterSLWallClock {

    #[allow(dead_code)]
    pub fn new(initial: Duration) -> Self {
        Self { initial }
    }

    pub fn from_nanos(initial_nanos: u64) -> Self {
        Self { initial: Duration::from_nanos(initial_nanos) }
    }

    #[allow(dead_code)]
    pub fn from_millis(initial_millis: u64) -> Self {
        Self { initial: Duration::from_millis(initial_millis) }
    }

    #[allow(dead_code)]
    pub fn from_secs(initial_secs: u64) -> Self {
        Self { initial: Duration::from_secs(initial_secs) }
    }
}

impl wasmtime_wasi::HostWallClock for DeterSLWallClock {
    fn now(&self) -> Duration {
        self.initial
    }

    fn resolution(&self) -> Duration {
        self.initial
    }
}

pub struct DeterSLMonotonicWallClock {
    initial_value: u64
}

impl DeterSLMonotonicWallClock {
    pub fn new(initial_value: u64) -> Self {
        Self { initial_value }
    }
}

impl wasmtime_wasi::HostMonotonicClock for DeterSLMonotonicWallClock {
    fn resolution(&self) -> u64 {
        self.initial_value
    }

    fn now(&self) -> u64 {
        self.initial_value
    }
}
