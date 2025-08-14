#[allow(warnings)]
mod bindings;

use crate::bindings::exports::detersl::func_api::handler::*;
use serde_json::{Value, Map};

struct Component;

impl Guest for Component {
    fn handle(event: Event) -> Output {
        let mut data: Value = serde_json::from_str(&event.data).unwrap_or(Value::Object(Map::new()));

        // Only operate on objects
        if let Value::Object(ref mut obj) = data {
            obj.insert("test".to_string(), Value::String("value".to_string()));
        } else {
            // If not an object, use an empty object
            let mut obj = Map::new();
            obj.insert("test".to_string(), Value::String("value".to_string()));
            data = Value::Object(obj);
        }

        // Serialize back to a string
        Output {
            data: serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

bindings::export!(Component with_types_in bindings);
