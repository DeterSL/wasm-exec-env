use super::{config_input::FuncInputEvent, config_execution_policy::FuncExecutionPolicy, config_link_opt::FuncLinkOpt, config_binary_location::FuncBinarySource, config_output::FuncOutputEvent, config_inital_values::FuncInitValue};

#[derive(sonic_rs::Deserialize, Clone)]
#[allow(dead_code)]
pub struct FuncBinaryConfig {
    pub func_name: String,
    pub func_invocation_id: String,
    pub func_binary_hash: String,
    pub func_binary_source: FuncBinarySource,
    pub func_input_event: FuncInputEvent,
    pub func_output_event: FuncOutputEvent,
    pub func_link_opt: FuncLinkOpt,
    pub func_execution_policy: FuncExecutionPolicy,
    pub func_initial_values: FuncInitValue
}
