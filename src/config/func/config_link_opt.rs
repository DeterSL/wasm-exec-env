#[derive(sonic_rs::Deserialize, Clone)]
pub struct FuncLinkOpt {
    #[serde(default)]
    pub link_clocks: bool,

    #[serde(default)]
    pub link_filesystem: bool,

    #[serde(default)]
    pub link_random: bool,

    #[serde(default)]
    pub link_cli: bool,

    #[serde(default)]
    pub link_io: bool,

    #[serde(default)]
    pub link_socket: bool
}
