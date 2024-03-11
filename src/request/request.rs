use crate::RequestParsingError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize, Serialize, Clone)]
pub struct Request {
	pub id: Value,
	pub method: String,
	pub params: Option<Value>,
}

impl Request {
	/// Will perform the `jsonrpc: "2.0"` validation.
	/// If this is not desired, using the standard `serde_json::from_value` would do the parsing
	/// and ignore `jsonrpc` property.
	pub fn from_value(mut value: Value) -> Result<Request, RequestParsingError> {
		// TODO: When capturing the Value, we might implement a safeguard to prevent capturing Value Object or arrays
		//       as they can be indefinitely large. One technical solution would be to replace the value with a String,
		//       using something like `"[object/array redacted, 'id' should be of type number, string or null]"` as the string.

		// -- validate the version
		// assert if present
		let Some(version) = value.get_mut("jsonrpc").map(Value::take) else {
			let (id, method) = take_rpc_ref(value);
			return Err(RequestParsingError::VersionMissing { id, method });
		};
		// assert if equal "2.0"
		if version.as_str().unwrap_or_default() != "2.0" {
			let (id, method) = take_rpc_ref(value);
			return Err(RequestParsingError::VersionInvalid { id, method, version });
		}

		// assert if id is present
		if value.get("id").is_none() {
			return Err(RequestParsingError::IdMissing {
				method: get_method(value),
			});
		}

		// assert that method is present
		let Some(method) = value.get("method") else {
			let id = value.get_mut("id").map(Value::take);
			return Err(RequestParsingError::MethodMissing { id });
		};

		// assert the method is the correct type
		if method.as_str().is_none() {
			let id = value.get_mut("id").map(Value::take);
			// Note: here the unwrap_or_default() should never run as we know method is present.
			//       however, we never want to call `unwrap()`, so, acceptable fall back.
			let method = value.get_mut("method").map(Value::take).unwrap_or_default();
			return Err(RequestParsingError::MethodInvalidType { id, method });
		}

		// -- serde json parse
		let res = serde_json::from_value(value).map_err(RequestParsingError::Parse)?;

		Ok(res)
	}
}

// Returns the eventual (id, method) tuple
fn take_rpc_ref(mut value: Value) -> (Option<Value>, Option<String>) {
	let id = value.get_mut("id").map(Value::take);
	let method = value.get_mut("method").and_then(|v| v.as_str().map(|s| s.to_string()));
	(id, method)
}

fn get_method(value: Value) -> Option<String> {
	value.get("method").and_then(|v| v.as_str().map(|s| s.to_string()))
}

/// Convenient TryFrom, and will execute the RpcRequest::from_value,
/// which will perform the version validation.
impl TryFrom<Value> for Request {
	type Error = RequestParsingError;
	fn try_from(value: Value) -> Result<Request, RequestParsingError> {
		Request::from_value(value)
	}
}
