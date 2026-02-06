fn default_true() -> bool { true }

#[derive(sonic_rs::Deserialize, Clone)]
pub struct FuncLinkOpt {
    #[serde(default = "default_true")]
    pub link_clocks: bool,

    #[serde(default = "default_true")]
    pub link_filesystem: bool,

    #[serde(default = "default_true")]
    pub link_random: bool,

    #[serde(default = "default_true")]
    pub link_cli: bool,

    #[serde(default = "default_true")]
    pub link_io: bool,

    #[serde(default = "default_true")]
    pub link_socket: bool,
}

