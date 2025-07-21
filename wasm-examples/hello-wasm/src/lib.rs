#[unsafe(no_mangle)]
pub extern "C" fn hello() {
    // In wasm, you can't print to stdout easily.
    // But you can export this and call from host.
}
