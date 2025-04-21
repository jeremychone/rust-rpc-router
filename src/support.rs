use derive_more::Display;
use serde_json::Value;

// region:    --- Serde Value Util

#[derive(Clone, Debug, Display)]
pub enum JsonType {
	Null,
	Bool,
	Integer,
	Unsigned,
	Float,
	String,
	Array,
	Object,
}

pub fn get_json_type(value: &Value) -> JsonType {
	match value {
		Value::Null => JsonType::Null,
		Value::Bool(_) => JsonType::Bool,
		Value::Number(n) => {
			if n.is_i64() {
				JsonType::Integer
			} else if n.is_u64() {
				JsonType::Unsigned
			} else if n.is_f64() {
				JsonType::Float
			} else {
				unreachable!("serde_json::Number should be i64, u64, or f64");
			}
		}
		Value::String(_) => JsonType::String,
		Value::Array(_) => JsonType::Array,
		Value::Object(_) => JsonType::Object,
	}
}

// endregion: --- Serde Value Util
