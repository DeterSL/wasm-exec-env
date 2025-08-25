
use std::vec;

use wasmtime::component::{Linker, HasData};
use wasmtime_wasi::p2::{add_clock_to_linker, WasiView, add_random_to_linker, add_cli_to_linker, add_io_to_linker, add_filesystem_to_linker, add_sockets_to_linker};

use crate::{config::func_config::FuncExecutionPolicy, core::{bindings, detersl_wasi::kv::{KVRcMut, KvView, KVRefMut}}};

pub trait LinkerOption<T>
where T: WasiView {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()>;
}

pub struct AddClockToLinker{}
impl AddClockToLinker {
    fn new() -> Box<Self> {
        Box::new(Self {  })
    }
}

impl<T> LinkerOption<T> for AddClockToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        add_clock_to_linker(linker);
        Ok(())
    }
}

pub struct AddRandomToLinker{}
impl AddRandomToLinker {
    fn new() -> Box<Self> {
        Box::new(Self {  })
    }
}

impl<T> LinkerOption<T> for AddRandomToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        add_random_to_linker(linker);
        Ok(())
    }
}

pub struct AddCliToLinker{}
impl AddCliToLinker {
    fn new() -> Box<Self> {
        Box::new(Self {  })
    }
}

impl<T> LinkerOption<T> for AddCliToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        add_cli_to_linker(linker);
        Ok(())
    }
}

pub struct AddIOToLinker{}
impl AddIOToLinker {
    fn new() -> Box<Self> {
        Box::new(Self {  })
    } 
}

impl<T> LinkerOption<T> for AddIOToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        add_io_to_linker(linker);
        Ok(())
    }
}

pub struct AddFSToLinker{}
impl AddFSToLinker {
    fn new() -> Box<Self> {
        Box::new(Self {  })
    } 
}

impl<T> LinkerOption<T> for AddFSToLinker 
where T: WasiView  + 'static{
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        add_filesystem_to_linker(linker);
        Ok(())
    }
}

pub struct AddSocketsToLinker{}
impl AddSocketsToLinker {
    fn new() -> Box<Self> {
        Box::new(Self {  })
    } 
}

impl<T> LinkerOption<T> for AddSocketsToLinker 
where T: WasiView  + 'static {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        add_sockets_to_linker(linker);
        Ok(())
    }
}

pub struct AddKVToLinker {
    kv: KVRcMut 
}

impl AddKVToLinker {
    fn new(kv: KVRcMut) -> Box<Self> {
        Box::new(Self { kv })
    } 
}

struct HasKV();

impl HasData for HasKV {
    type Data<'a> = KVRefMut<'a>;
}

impl<T> LinkerOption<T> for AddKVToLinker
where T: WasiView  + KvView  + 'static {
    fn apply_to_linker(&mut self, linker: &mut Linker<T>) -> anyhow::Result<()> {
        let f: fn(&mut T) -> KVRefMut = |t| t.kv().borrow_mut();
        bindings::detersl::api::kv::add_to_linker::<T, HasKV>(linker, f).unwrap();
        Ok(())
    }
}

pub fn get_linker_opts_from_execution_policy<T>(execution_policy: &FuncExecutionPolicy) -> Vec<Box<dyn LinkerOption<T>>>
where T: WasiView + 'static {
    let mut opts = Vec::<Box<dyn LinkerOption<T>>>::new();

    if execution_policy.allow_clocks {
        opts.push(AddClockToLinker::new());
    }

    if execution_policy.allow_random {
        opts.push(AddRandomToLinker::new());
    }

    if execution_policy.allow_filesystem {
        opts.push(AddFSToLinker::new());
    }

    if execution_policy.allow_cli {
        opts.push(AddCliToLinker::new());
    }

    if execution_policy.allow_io {
        opts.push(AddIOToLinker::new());
    }

    if execution_policy.allow_socket {
        opts.push(AddSocketsToLinker::new());
    }

    opts
}

pub fn get_kv_as_opt<T>(kv: KVRcMut) -> Vec<Box<dyn LinkerOption<T>>>
where T: KvView + WasiView + 'static{
    let mut opts = Vec::<Box<dyn LinkerOption<T>>>::new();
    opts.push(AddKVToLinker::new(kv));
    opts
}

