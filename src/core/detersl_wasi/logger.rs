use wasmtime_wasi::p2::Logger;

pub struct SimpleLogger {

}

impl SimpleLogger {
    pub fn new() -> Self {
        Self {  }
    }
}

impl Logger for SimpleLogger {
    fn log(&mut self, _log_level: wasmtime_wasi::p2::LogLevel, log: String) {
        println!("Got syscall: {}", log);
    }
}
