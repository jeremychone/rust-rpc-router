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
	// Will perform the `jsonrpc: "2.0"` validation.
	// If this is not desired, using the standard `serde_json::from_value` would do the parsing
	// and ignore `jsonrpc` property.
	pub fn from_value(value: Value) -> Result<Request, RequestParsingError> {
		// -- validate the version
		if let Some(jsonrpc) = value.get("jsonrpc").and_then(|v| v.as_str()) {
			if jsonrpc != "2.0" {
				// let value.get("id");
				let (id, method) = take_rpc_ref(value);
				return Err(RequestParsingError::VersionInvalid { id, method });
			}
		} else {
			let (id, method) = take_rpc_ref(value);
			return Err(RequestParsingError::VersionMissing { id, method });
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

/// Convenient TryFrom, and will execute the RpcRequest::from_value,
/// which will perform the version validation.
impl TryFrom<Value> for Request {
	type Error = RequestParsingError;
	fn try_from(value: Value) -> Result<Request, RequestParsingError> {
		Request::from_value(value)
	}
}
