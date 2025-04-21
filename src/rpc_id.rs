use crate::RequestParsingError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::sync::Arc;

/// Represents a JSON-RPC 2.0 Request ID, which can be a String, Number, or Null.
/// Uses `Arc<str>` for strings to allow for efficient cloning, especially when the
/// ID is part of request/response structures that might be cloned (e.g., in tracing/logging).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RpcId {
	String(Arc<str>),
	Number(i64),
	Null,
}

// region:    --- std Display

impl core::fmt::Display for RpcId {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			RpcId::String(s) => write!(f, "{}", s),
			RpcId::Number(n) => write!(f, "{}", n),
			RpcId::Null => write!(f, "null"),
		}
	}
}

// endregion: --- std Display

// -- Conversions

impl RpcId {
	/// Converts the `RpcId` into a `serde_json::Value`. Infallible.
	pub fn to_value(&self) -> Value {
		match self {
			RpcId::String(s) => Value::String(s.to_string()),
			RpcId::Number(n) => Value::Number((*n).into()),
			RpcId::Null => Value::Null,
		}
	}

	/// Attempts to convert a `serde_json::Value` into an `RpcId`.
	/// Returns `Error::RpcIdInvalid` if the `value` is not a String, Number, or Null.
	pub fn from_value(value: Value) -> core::result::Result<Self, RequestParsingError> {
		match value {
			Value::String(s) => Ok(RpcId::String(s.into())),
			Value::Number(n) => n.as_i64().map(RpcId::Number).ok_or_else(|| RequestParsingError::IdInvalid {
				actual: format!("{n}"),
				cause: "Number is not a valid i64".into(),
			}),
			Value::Null => Ok(RpcId::Null),
			_ => Err(RequestParsingError::IdInvalid {
				actual: format!("{value:?}"),
				cause: "ID must be a String, Number, or Null".into(),
			}),
		}
	}
}

// -- Default

impl Default for RpcId {
	/// Defaults to `RpcId::Null`.
	fn default() -> Self {
		RpcId::Null
	}
}

// -- Serde

impl Serialize for RpcId {
	fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		match self {
			RpcId::String(s) => serializer.serialize_str(s),
			RpcId::Number(n) => serializer.serialize_i64(*n),
			RpcId::Null => serializer.serialize_none(),
		}
	}
}

impl<'de> Deserialize<'de> for RpcId {
	fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let value = Value::deserialize(deserializer)?;
		RpcId::from_value(value).map_err(serde::de::Error::custom)
	}
}

// -- From Implementations

impl From<String> for RpcId {
	fn from(s: String) -> Self {
		RpcId::String(s.into())
	}
}

impl From<&str> for RpcId {
	fn from(s: &str) -> Self {
		RpcId::String(s.into())
	}
}

impl From<i64> for RpcId {
	fn from(n: i64) -> Self {
		RpcId::Number(n)
	}
}

impl From<i32> for RpcId {
	fn from(n: i32) -> Self {
		RpcId::Number(n as i64)
	}
}

impl From<u32> for RpcId {
	fn from(n: u32) -> Self {
		RpcId::Number(n as i64)
	}
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use serde_json::{from_value, json, to_value};

	type TestResult<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	#[test]
	fn test_rpc_id_ser_de() -> TestResult<()> {
		// -- Setup & Fixtures
		let ids = [
			RpcId::String("id-1".into()),
			RpcId::Number(123),
			RpcId::Null,
			RpcId::String("".into()), // Empty string
		];
		let expected_values = [
			json!("id-1"),
			json!(123),
			json!(null),
			json!(""), // Empty string JSON
		];

		// -- Exec & Check
		for (i, id) in ids.iter().enumerate() {
			let value = to_value(id)?;
			assert_eq!(value, expected_values[i], "Serialization check for id[{i}]");

			let deserialized_id: RpcId = from_value(value.clone())?;
			assert_eq!(&deserialized_id, id, "Deserialization check for id[{i}]");

			let from_value_id = RpcId::from_value(value)?;
			assert_eq!(from_value_id, *id, "from_value check for id[{i}]");
		}

		Ok(())
	}

	#[test]
	fn test_rpc_id_from_value_invalid() -> TestResult<()> {
		// -- Setup & Fixtures
		let invalid_values = vec![
			json!(true),
			json!(123.45), // Float number
			json!([1, 2]),
			json!({"a": 1}),
		];

		// -- Exec & Check
		for value in invalid_values {
			let res = RpcId::from_value(value.clone());
			assert!(
				matches!(res, Err(RequestParsingError::IdInvalid { .. })),
				"Expected RpcIdInvalid for value: {:?}",
				value
			);
		}

		Ok(())
	}

	#[test]
	fn test_rpc_id_to_value() -> TestResult<()> {
		// -- Setup & Fixtures
		let id_str = RpcId::String("hello".into());
		let id_num = RpcId::Number(42);
		let id_null = RpcId::Null;

		// -- Exec
		let val_str = id_str.to_value();
		let val_num = id_num.to_value();
		let val_null = id_null.to_value();

		// -- Check
		assert_eq!(val_str, json!("hello"));
		assert_eq!(val_num, json!(42));
		assert_eq!(val_null, json!(null));

		Ok(())
	}

	#[test]
	fn test_rpc_id_from_impls() -> TestResult<()> {
		// -- Check String/&str
		assert_eq!(RpcId::from("test_str"), RpcId::String("test_str".into()));
		assert_eq!(
			RpcId::from(String::from("test_string")),
			RpcId::String("test_string".into())
		);

		// -- Check numbers
		assert_eq!(RpcId::from(100i64), RpcId::Number(100));
		assert_eq!(RpcId::from(200i32), RpcId::Number(200));
		assert_eq!(RpcId::from(300u32), RpcId::Number(300));

		Ok(())
	}

	#[test]
	fn test_rpc_id_default() -> TestResult<()> {
		// -- Check
		assert_eq!(RpcId::default(), RpcId::Null);

		Ok(())
	}
}

// endregion: --- Tests
