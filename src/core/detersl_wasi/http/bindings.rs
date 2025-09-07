mod genrated {
    use crate::detersl_wasi::http::types;
    use crate::detersl_wasi::http::body;
    use crate::detersl_wasi::http::errors;
    
    wasmtime::component::bindgen!({
        path: "src/wit",
        world: "detersl:http-api/http-api",
        require_store_data_send: true,
        trappable_imports: true,
        with: {
            // Upstream package dependencies
            "wasi:io": wasmtime_wasi::p2::bindings::io,
            
            "detersl:http-api/types/outgoing-request": types::HostOutgoingRequest,
            "detersl:http-api/types/incoming-response": types::HostIncomingResponse,
            "detersl:http-api/types/future-incoming-response": types::HostFutureIncomingResponse,
            "detersl:http-api/types/incoming-body": body::HostIncomingBody,
            "detersl:http-api/types/outgoing-body": body::HostOutgoingBody,
            "detersl:http-api/types/future-trailers": body::HostFutureTrailers,
            "detersl:http-api/types/fields": types::HostFields,
        },
        trappable_error_type: {
            "detersl:http-api/types/error-code" => errors::HttpError,
        }
    });
}

pub use self::genrated::*;
//pub use self::genrated::DeterslApi;
