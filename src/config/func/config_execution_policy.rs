use serde::Deserialize;
use wasmtime_wasi::p2::{ClockEvent, FSEvent, RndEvent, UDPEvent, TCPEvent, EnvEvent, ExitEvent};
use std::any::{Any, TypeId};

use wasmtime::FilterFn;

#[derive(Debug, Deserialize)]
pub struct FuncExecutionPolicy {
    #[serde(default)]
    pub allow_clocks: bool,

    // If allow_clocks == false, an optional `clocks` subpolicy can enable
    // specific clock variants.
    #[serde(default)]
    pub clocks: Option<ClockPolicy>,

    #[serde(default)]
    pub allow_filesystem: bool,

    // If allow_filesystem == false, an optional `filesystem` subpolicy can
    // enable specific FS variants.
    #[serde(default)]
    pub filesystem: Option<FsPolicy>,

    #[serde(default)]
    pub allow_random: bool,

    #[serde(default)]
    pub allow_cli: bool,

    #[serde(default)]
    pub allow_socket: bool,

    // If allow_socket == false, an optional `socket` subpolicy can enable
    // specific TCP/UDP variants.
    #[serde(default)]
    pub socket: Option<SocketPolicy>,
}

#[derive(Debug, Deserialize)]
pub struct ClockPolicy {
    #[serde(default)]
    pub wall_clock: bool,
    #[serde(default)]
    pub monotonic_clock: bool,
}

#[derive(Debug, Deserialize)]
pub struct FsPolicy {
    #[serde(default)]
    pub read_fs: bool,
    #[serde(default)]
    pub write_fs: bool,
    #[serde(default)]
    pub open_fs: bool,
    #[serde(default)]
    pub list_dir: bool,
    #[serde(default)]
    pub make_dir: bool,
    #[serde(default)]
    pub rm_dir: bool,
    #[serde(default)]
    pub read_dir: bool,
    #[serde(default)]
    pub rename_dir: bool,
    #[serde(default)]
    pub other_allowed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UdpPolicy {
    #[serde(default)]
    pub receive: bool,
    #[serde(default)]
    pub send: bool,
    #[serde(default)]
    pub creation: bool,
    #[serde(default)]
    pub options_allowed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TcpPolicy {
    #[serde(default)]
    pub read: bool,
    #[serde(default)]
    pub write: bool,
    #[serde(default)]
    pub connect: bool,
    #[serde(default)]
    pub listen: bool,
    #[serde(default)]
    pub accept: bool,
    #[serde(default)]
    pub creation: bool,
    #[serde(default)]
    pub options_allowed: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SocketPolicy {
    #[serde(default)]
    pub udp: Option<UdpPolicy>,

    #[serde(default)]
    pub tcp: Option<TcpPolicy>,
}

pub fn make_clock_filter(policy: &FuncExecutionPolicy) -> Option<FilterFn> {
    let mask: u8 = if policy.allow_clocks {
        0b11
    } else if let Some(c) = &policy.clocks {
        ((c.wall_clock as u8) << 0) | ((c.monotonic_clock as u8) << 1)
    } else {
        0
    };

    if mask == 0 {
        return Some(Box::new(|_any: &dyn Any| { false }));
    }

    let m = mask;
    Some(Box::new(move |any: &dyn Any| {
        any.downcast_ref::<ClockEvent>().map_or(false, |ev| match ev {
            ClockEvent::WallClock => (m & 0b01) != 0,
            ClockEvent::MonotonicClock => (m & 0b10) != 0,
        })
    }))
}

pub fn make_fs_filter(policy: &FuncExecutionPolicy) -> Option<FilterFn> {
    let mask: u16 = if policy.allow_filesystem {
        0xFF
    } else if let Some(fs) = &policy.filesystem {
        ((fs.read_fs as u16) << 0)
            | ((fs.write_fs as u16) << 1)
            | ((fs.open_fs as u16) << 2)
            | ((fs.list_dir as u16) << 3)
            | ((fs.make_dir as u16) << 4)
            | ((fs.rm_dir as u16) << 5)
            | ((fs.read_dir as u16) << 6)
            | ((fs.rename_dir as u16) << 7)
    } else {
        0
    };

    let other_allowed = policy
        .filesystem
        .as_ref()
        .map(|fs| fs.other_allowed.clone())
        .unwrap_or_default();

    if mask == 0 && other_allowed.is_empty() {
        return Some(Box::new(|_any: &dyn Any| { false }));
    }

    Some(Box::new(move |any: &dyn Any| {
        any.downcast_ref::<FSEvent>().map_or(false, |ev| match ev {
            FSEvent::ReadFS => (mask & (1 << 0)) != 0,
            FSEvent::WriteFS => (mask & (1 << 1)) != 0,
            FSEvent::OpenFS => (mask & (1 << 2)) != 0,
            FSEvent::ListDir => (mask & (1 << 3)) != 0,
            FSEvent::MakeDir => (mask & (1 << 4)) != 0,
            FSEvent::RmDir => (mask & (1 << 5)) != 0,
            FSEvent::ReadDir => (mask & (1 << 6)) != 0,
            FSEvent::RenameDir => (mask & (1 << 7)) != 0,
            FSEvent::Other(s) => other_allowed.iter().any(|pat| pat == s),
        })
    }))
}

pub fn make_random_filter(policy: &FuncExecutionPolicy) -> Option<FilterFn> {
    let allow = policy.allow_random;
    if !allow {
        return Some(Box::new(|_any: &dyn Any| { false }));
    }
    
    Some(Box::new(move |any: &dyn Any| {
        any.downcast_ref::<RndEvent>().is_some()
    }))
}

pub fn make_cli_filter(policy: &FuncExecutionPolicy) -> Option<FilterFn> {
    let allow = policy.allow_cli;
    if !allow {
        return Some(Box::new(|_any: &dyn Any| { false }));
    }
    
    Some(Box::new(move |any: &dyn Any| {
        any.downcast_ref::<EnvEvent>().is_some() || any.downcast_ref::<ExitEvent>().is_some()
    }))
}

pub fn make_socket_filters(policy: &FuncExecutionPolicy) -> Vec<(TypeId, FilterFn)> {
    if policy.allow_socket {
        let udp_filter: FilterFn = Box::new(move |any: &dyn Any| any.downcast_ref::<UDPEvent>().is_some());
        let tcp_filter: FilterFn = Box::new(move |any: &dyn Any| any.downcast_ref::<TCPEvent>().is_some());
        return vec![
            (TypeId::of::<UDPEvent>(), udp_filter),
            (TypeId::of::<TCPEvent>(), tcp_filter),
        ];
    }

    let (udp_mask, udp_opts, tcp_mask, tcp_opts) = match &policy.socket {
        Some(sp) => {
            let (umask, uopts) = match &sp.udp {
                Some(up) => {
                    let mask = ((up.receive as u8) << 0)
                        | ((up.send as u8) << 1)
                        | ((up.creation as u8) << 2);
                    (mask, up.options_allowed.clone())
                }
                None => (0u8, Vec::new()),
            };
            let (tmask, topts) = match &sp.tcp {
                Some(tp) => {
                    let mask = ((tp.read as u8) << 0)
                        | ((tp.write as u8) << 1)
                        | ((tp.connect as u8) << 2)
                        | ((tp.listen as u8) << 3)
                        | ((tp.accept as u8) << 4)
                        | ((tp.creation as u8) << 5);
                    (mask, tp.options_allowed.clone())
                }
                None => (0u8, Vec::new()),
            };
            (umask, uopts, tmask, topts)
        }
        None => (0u8, Vec::new(), 0u8, Vec::new()),
    };

    let mut out: Vec<(TypeId, FilterFn)> = Vec::with_capacity(2);

    if udp_mask != 0 || !udp_opts.is_empty() {
        let um = udp_mask;
        let uopts = udp_opts;
        let f: FilterFn = Box::new(move |any: &dyn Any| {
            any.downcast_ref::<UDPEvent>().map_or(false, |ev| match ev {
                UDPEvent::Receieve => (um & (1 << 0)) != 0,
                UDPEvent::Send => (um & (1 << 1)) != 0,
                UDPEvent::Creation => (um & (1 << 2)) != 0,
                UDPEvent::Option(s) => uopts.iter().any(|pat| pat == s),
            })
        });
        out.push((TypeId::of::<UDPEvent>(), f));
    }

    if tcp_mask != 0 || !tcp_opts.is_empty() {
        let tm = tcp_mask;
        let topts = tcp_opts;
        let f: FilterFn = Box::new(move |any: &dyn Any| {
            any.downcast_ref::<TCPEvent>().map_or(false, |ev| match ev {
                TCPEvent::Read => (tm & (1 << 0)) != 0,
                TCPEvent::Write => (tm & (1 << 1)) != 0,
                TCPEvent::Connect => (tm & (1 << 2)) != 0,
                TCPEvent::Listen => (tm & (1 << 3)) != 0,
                TCPEvent::Accept => (tm & (1 << 4)) != 0,
                TCPEvent::Creation => (tm & (1 << 5)) != 0,
                TCPEvent::Option(s) => topts.iter().any(|pat| pat == s),
            })
        });
        out.push((TypeId::of::<TCPEvent>(), f));
    }

    out
}

pub fn make_filters(policy: &FuncExecutionPolicy) -> Vec<(TypeId, FilterFn)> {
    let mut out = Vec::new();

    if let Some(f) = make_clock_filter(policy) {
        out.push((TypeId::of::<ClockEvent>(), f));
    }

    if let Some(f) = make_fs_filter(policy) {
        out.push((TypeId::of::<FSEvent>(), f));
    }

    if let Some(f) = make_random_filter(policy) {
        out.push((TypeId::of::<RndEvent>(), f));
    }

    if let Some(f) = make_cli_filter(policy) {
        let f_env = {
            let allow = policy.allow_cli;
            Box::new(move |any: &dyn Any| any.downcast_ref::<EnvEvent>().map_or(false, |_| allow))
                as FilterFn
        };
        let f_exit = {
            let allow = policy.allow_cli;
            Box::new(move |any: &dyn Any| any.downcast_ref::<ExitEvent>().map_or(false, |_| allow))
                as FilterFn
        };
        out.push((TypeId::of::<EnvEvent>(), f_env));
        out.push((TypeId::of::<ExitEvent>(), f_exit));
    }

    out.extend(make_socket_filters(policy));

    out
}
