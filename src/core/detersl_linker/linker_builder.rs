use wasmtime::component::Linker;
use wasmtime_wasi::p2::WasiView;

use crate::config::FuncLinkOpt;

use super::linker_opts::LinkerOption;

#[allow(dead_code)] // TODO: fix later
pub struct LinkerBuilder<T>
where T: WasiView + 'static {
    linker: Linker<T>,
    opts: Vec<Box<dyn LinkerOption<T>>>
}

#[allow(dead_code)] // TODO: fix later
impl<T> LinkerBuilder<T>
where T: WasiView + 'static {
    pub fn new(linker: Linker<T>) -> Self {
        Self { linker, opts: Vec::new() }
    }

    pub fn add_opts(&mut self, opts: &mut Vec<Box<dyn LinkerOption<T>>>) {
        self.opts.append(opts);
    }

    pub fn build(mut self) -> Linker<T> {
        for mut opt in self.opts {
            let _ = opt.apply_to_linker(&mut self.linker);
        }

        self.linker
    }
}

#[allow(dead_code)] // TODO: fix later
pub fn encode_linker_opt(linker_opt: &FuncLinkOpt) -> String {
    let mut encode_policy = String::new();

    fn encode_on_and_off_field(switch: bool) -> char {
        if switch {
            '1'
        } else {
            '0'
        }
    }

    encode_policy.push(encode_on_and_off_field(linker_opt.link_clocks));
    encode_policy.push(encode_on_and_off_field(linker_opt.link_filesystem));
    encode_policy.push(encode_on_and_off_field(linker_opt.link_random));
    encode_policy.push(encode_on_and_off_field(linker_opt.link_cli));
    encode_policy.push(encode_on_and_off_field(linker_opt.link_io));
    encode_policy.push(encode_on_and_off_field(linker_opt.link_socket));
    encode_policy
}
