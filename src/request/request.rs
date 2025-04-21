use crate::support::get_json_type;
use crate::{RequestParsingError, RpcId};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Request {
	pub id: RpcId,
	pub method: String,
	pub params: Option<Value>,
}

impl Request {
	/// Will perform the `jsonrpc: "2.0"` validation and parse the request.
	/// If this is not desired, using the standard `serde_json::from_value` would do the parsing
	/// and ignore `jsonrpc` property.
	pub fn from_value(value: Value) -> Result<Request, RequestParsingError> {
		// TODO: When capturing the Value, we might implement a safeguard to prevent capturing Value Object or arrays
		//       as they can be indefinitely large. One technical solution would be to replace the value with a String,
		//       using something like `"[object/array redacted, 'id' should be of type number, string or null]"` as the string.
		let value_type = get_json_type(&value);

		let Value::Object(mut obj) = value else {
			return Err(RequestParsingError::RequestInvalidType {
				actual_type: value_type.to_string(),
			});
		};

		// -- Check and remove `jsonrpc` property
		let version_validation = match obj.remove("jsonrpc") {
			Some(version) => {
				if version.as_str().unwrap_or_default() == "2.0" {
					Ok(())
				} else {
					let (id_val, method) = extract_id_value_and_method(&obj);
					Err(RequestParsingError::VersionInvalid {
						id: id_val,
						method,
						version,
					})
				}
			}
			None => {
				let (id_val, method) = extract_id_value_and_method(&obj);
				Err(RequestParsingError::VersionMissing { id: id_val, method })
			}
		};
		version_validation?;

		// -- Check method presence and type
		let method = match obj.remove("method") {
			None => {
				let id_val = obj.get_mut("id").map(Value::take);
				return Err(RequestParsingError::MethodMissing { id: id_val });
			}
			Some(method_val) => match method_val {
				Value::String(method_name) => method_name,
				other => {
					let id = obj.get("id").cloned();
					return Err(RequestParsingError::MethodInvalidType { id, method: other });
				}
			},
		};

		// -- Check id presence and parse it
		let id = match obj.get_mut("id").map(Value::take) {
			Some(id_value) => RpcId::from_value(id_value)?,
			None => {
				// Note: Technically, JSON-RPC 2.0 allows requests without IDs (Notifications).
				// However, this router is primarily designed for Request/Response cycles.
				// If Notification support is added later, this check needs revision.
				// For now, we mandate an ID.
				return Err(RequestParsingError::IdMissing {
					method: get_method(&obj),
				});
			}
		};

		// -- Extract params (can be absent, which is valid)
		let params = obj.get_mut("params").map(Value::take);

		Ok(Request { id, method, params })
	}
}

// Returns the eventual (id_value, method) tuple from a reference
fn extract_id_value_and_method(obj: &serde_json::Map<String, Value>) -> (Option<Value>, Option<String>) {
	let id = obj.get("id").cloned();
	let method = obj.get("method").and_then(|v| v.as_str().map(|s| s.to_string()));
	(id, method)
}

fn get_method(obj: &serde_json::Map<String, Value>) -> Option<String> {
	obj.get("method").and_then(|v| v.as_str().map(|s| s.to_string()))
}

/// Convenient TryFrom, and will execute the Request::from_value,
/// which will perform the version validation.
impl TryFrom<Value> for Request {
	type Error = RequestParsingError;
	fn try_from(value: Value) -> Result<Request, RequestParsingError> {
		Request::from_value(value)
	}
}
