use super::{config_input::FuncInputEvent, config_execution_policy::FuncExecutionPolicy, config_link_opt::FuncLinkOpt, config_binary_location::FuncBinarySource, config_output::FuncOutputEvent, config_inital_values::FuncInitValue};

#[derive(serde::Deserialize, Clone)]
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

impl FuncBinaryConfig {
    pub fn new(
        func_name: String, 
        func_invocation_id: String,
        func_binary_hash: String,
        func_binary_source: FuncBinarySource, 
        func_input_event: FuncInputEvent,
        func_output_event: FuncOutputEvent,
        func_link_opt: FuncLinkOpt,
        func_execution_policy: FuncExecutionPolicy,
        func_initial_values: FuncInitValue
    ) -> Self {
        FuncBinaryConfig { 
            func_name,
            func_invocation_id,
            func_binary_hash,
            func_binary_source,
            func_input_event,
            func_output_event,
            func_link_opt,
            func_execution_policy, 
            func_initial_values
        }
    }
}


