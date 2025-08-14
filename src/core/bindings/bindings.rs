mod genrated {
    wasmtime::component::bindgen!({
        path: "src/wit",
        world: "detersl-api",
    });
}

pub use self::genrated::exports::*;
pub use self::genrated::DeterslApi;
