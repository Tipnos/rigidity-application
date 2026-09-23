use std::collections::HashMap;

fn make_path_and_query(path: &str, params: &HashMap<&str, String>) -> String {
    let mut result = path.to_string();
    
    if !params.is_empty() {
        result.push_str("?");
    }

    for (key, value) in params {
        result.push_str(key);
        result.push_str("=");
        result.push_str(value.as_str());
        result.push_str("&");
    }

    result.pop();

    result
}

pub mod websocket;
pub mod custom_room;
pub mod aws;
pub mod steam;

// Serialize and deserialize logic for dealing with nested values reprsented as
// JSON strings.
pub mod as_json_string {
    use serde_json;
    use serde::ser::{Serialize, Serializer};
    use serde::de::{Deserialize, DeserializeOwned, Deserializer};

    // Serialize to a JSON string, then serialize the string to the output
    // format.
    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: Serialize,
        S: Serializer,
    {
        use serde::ser::Error;
        let j = serde_json::to_string(value).map_err(Error::custom)?;
        j.serialize(serializer)
    }

    // Deserialize a string from the input format, then deserialize the content
    // of that string as JSON.
    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: DeserializeOwned,
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let j = String::deserialize(deserializer)?;
        serde_json::from_str(&j).map_err(Error::custom)
    }
}