use wasmtime::component::{HasData, Linker};
use wasmtime_wasi::p2::IoImpl;

pub use self::types::{DeterSLHttpView, DeterSLHttpImpl, DeterSLHttpCtx};

mod types;
mod types_impl;
mod http_impl;
mod body;
mod errors;
mod bindings;
mod io;

pub fn add_only_http_to_linker_async<T>(
    l: &mut wasmtime::component::Linker<T>,
) -> anyhow::Result<()>
where
    T: DeterSLHttpView + 'static,
{
    bindings::detersl::http_api::outgoing_handler::add_to_linker::<_, DeterSLHttp<T>>(l, |x| {
        DeterSLHttpImpl (IoImpl(x))
    })?;
    bindings::detersl::http_api::types::add_to_linker::<_, DeterSLHttp<T>>(l, |x| {
        DeterSLHttpImpl (IoImpl(x))
    })?;

    Ok(())
}

struct DeterSLHttp<T>(T);

impl<T: 'static> HasData for DeterSLHttp<T> {
    type Data<'a> = DeterSLHttpImpl<&'a mut T>;
}
