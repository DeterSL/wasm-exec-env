use serde_json::{Value, Map};

/// Implements the JSON processing function.
pub struct Jsonprocess;

impl Jsonprocess {
    /// Processes the input JSON string by adding a key "test" with value "value".
    /// If the input is not valid JSON, an empty object is used.
    pub fn process(input: String) -> String {
        // Attempt to parse the input as JSON
        let mut data: Value = serde_json::from_str(&input).unwrap_or(Value::Object(Map::new()));

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
        serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())
    }
}
