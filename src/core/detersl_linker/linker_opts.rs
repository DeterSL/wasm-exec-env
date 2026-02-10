use wasmtime::component::{Linker, HasData};
use wasmtime_wasi::p2::{add_clock_to_linker, WasiView, add_random_to_linker, add_cli_to_linker, add_io_to_linker, add_filesystem_to_linker, add_sockets_to_linker};

use crate::{config::FuncLinkOpt, core::{bindings, detersl_wasi::{http, kv::{KVType, KvView}}}};

pub trait LinkerOption<T>
where T: WasiView {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()>;
}

pub struct AddClockToLinker{}
impl AddClockToLinker {
    #[allow(dead_code)]
    fn new() -> Box<Self> {
        Box::new(Self {  })
    }
}

impl<T> LinkerOption<T> for AddClockToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let _ = add_clock_to_linker(linker);
        Ok(())
    }
}

pub struct AddRandomToLinker{}
impl AddRandomToLinker {
    #[allow(dead_code)]
    fn new() -> Box<Self> {
        Box::new(Self {  })
    }
}

impl<T> LinkerOption<T> for AddRandomToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let _ = add_random_to_linker(linker);
        Ok(())
    }
}

pub struct AddCliToLinker{}
impl AddCliToLinker {
    #[allow(dead_code)]
    fn new() -> Box<Self> {
        Box::new(Self {  })
    }
}

impl<T> LinkerOption<T> for AddCliToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let _ = add_cli_to_linker(linker);
        Ok(())
    }
}

pub struct AddIOToLinker{}
impl AddIOToLinker {
    #[allow(dead_code)]
    fn new() -> Box<Self> {
        Box::new(Self {  })
    } 
}

impl<T> LinkerOption<T> for AddIOToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let _ = add_io_to_linker(linker);
        Ok(())
    }
}

pub struct AddFSToLinker{}
impl AddFSToLinker {
    #[allow(dead_code)]
    fn new() -> Box<Self> {
        Box::new(Self {  })
    } 
}

impl<T> LinkerOption<T> for AddFSToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let _ = add_filesystem_to_linker(linker);
        Ok(())
    }
}

pub struct AddSocketsToLinker{}
impl AddSocketsToLinker {
    #[allow(dead_code)]
    fn new() -> Box<Self> {
        Box::new(Self {  })
    } 
}

impl<T> LinkerOption<T> for AddSocketsToLinker 
where T: WasiView  + 'static {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let _ = add_sockets_to_linker(linker);
        Ok(())
    }
}

pub struct AddKVToLinker {
}

impl AddKVToLinker {
    pub fn new() -> Box<Self> {
        Box::new(Self { })
    } 
}

struct HasKV();

impl HasData for HasKV {
    type Data<'a> = &'a mut dyn KVType;
}

impl<T> LinkerOption<T> for AddKVToLinker
where T: WasiView  + KvView  + 'static {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let f: fn(&mut T) -> &mut dyn KVType = |t| t.kv();
        bindings::detersl::kv_api::kv::add_to_linker::<T, HasKV>(linker, f).unwrap();
        Ok(())
    }
}

pub struct AddHTTPToLinker {
}

impl AddHTTPToLinker {
    #[allow(dead_code)]
   fn new() -> Box<Self> {
       Box::new(AddHTTPToLinker {  })
   }
}

impl<T> LinkerOption<T> for AddHTTPToLinker
where T: http::DeterSLHttpView +  WasiView  + KvView  + 'static {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        http::add_only_http_to_linker_async(linker)
    }
}


#[allow(dead_code)]
pub fn get_linker_opts_from_link_opt<T>(linker_opt: &FuncLinkOpt) -> Vec<Box<dyn LinkerOption<T>>>
where T: WasiView + 'static {
    let mut opts = Vec::<Box<dyn LinkerOption<T>>>::new();

    if linker_opt.link_clocks {
        opts.push(AddClockToLinker::new());
    }

    if linker_opt.link_random {
        opts.push(AddRandomToLinker::new());
    }

    if linker_opt.link_filesystem {
        opts.push(AddFSToLinker::new());
    }

    if linker_opt.link_cli {
        opts.push(AddCliToLinker::new());
    }

    if linker_opt.link_io {
        opts.push(AddIOToLinker::new());
    }

    if linker_opt.link_socket {
        opts.push(AddSocketsToLinker::new());
    }

    opts
}

#[allow(dead_code)]
pub fn get_kv_as_opt<T>() -> Vec<Box<dyn LinkerOption<T>>>
where T: KvView + WasiView + 'static{
    let mut opts = Vec::<Box<dyn LinkerOption<T>>>::new();
    opts.push(AddKVToLinker::new());
    opts
}

#[allow(dead_code)]
pub fn get_http_as_opt<T>() -> Vec<Box<dyn LinkerOption<T>>>
where T: http::DeterSLHttpView + KvView + WasiView + 'static {
    let mut opts = Vec::<Box<dyn LinkerOption<T>>>::new();
    opts.push(AddHTTPToLinker::new());
    opts
}

