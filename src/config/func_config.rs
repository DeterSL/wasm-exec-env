
#[derive(serde::Deserialize)]
pub struct FuncInputEvent {
    pub data: String
}

#[derive(serde::Deserialize)]
pub struct FuncExecutionPolicy {
    #[serde(default)]
    allow_clocks: bool,

    #[serde(default)]
    allow_filesystem: bool,

    #[serde(default)]
    allow_random: bool,

    #[serde(default)]
    allow_io: bool,

    #[serde(default)]
    allow_socket: bool
}

#[derive(serde::Deserialize)]
pub struct FuncBinaryConfig {
    pub func_name: String,
    pub func_binary_path: String,
    pub func_input_event: FuncInputEvent,
    pub func_execution_policy: FuncExecutionPolicy
}

impl FuncBinaryConfig {
    pub fn new(
        func_name: String, 
        func_binary_path: String, 
        func_input_event: FuncInputEvent,
        func_execution_policy: FuncExecutionPolicy
    ) -> Self {
        FuncBinaryConfig { 
            func_name,
            func_binary_path,
            func_input_event,
            func_execution_policy 
        }
    }
}

pub trait FuncBinaryConfigParser {
    fn parse(&self) -> anyhow::Result<FuncBinaryConfig>;
}
