use crate::support::get_json_type;
use crate::{RpcId, RpcRequestParsingError};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serializer};
use serde_json::Value;

/// The raw JSON-RPC request object, serving as the foundation for RPC routing.
#[derive(Deserialize, Clone, Debug)]
pub struct RpcRequest {
	pub id: RpcId,
	pub method: String,
	pub params: Option<Value>,
}

impl RpcRequest {
	pub fn new(id: impl Into<RpcId>, method: impl Into<String>, params: Option<Value>) -> Self {
		RpcRequest {
			id: id.into(),
			method: method.into(),
			params,
		}
	}
}

/// Custom parser (probably need to be a deserializer)
impl RpcRequest {
	pub fn from_value(value: Value) -> Result<RpcRequest, RpcRequestParsingError> {
		RpcRequest::from_value_with_checks(value, RpcRequestCheckFlags::ALL)
	}

	/// Will perform the `jsonrpc: "2.0"` validation and parse the request.
	/// If this is not desired, using the standard `serde_json::from_value` would do the parsing
	/// and ignore `jsonrpc` property.
	pub fn from_value_with_checks(
		value: Value,
		checks: RpcRequestCheckFlags,
	) -> Result<RpcRequest, RpcRequestParsingError> {
		// TODO: When capturing the Value, we might implement a safeguard to prevent capturing Value Object or arrays
		//       as they can be indefinitely large. One technical solution would be to replace the value with a String,
		//       using something like `"[object/array redacted, 'id' should be of type number, string or null]"` as the string.
		let value_type = get_json_type(&value);

		let Value::Object(mut obj) = value else {
			return Err(RpcRequestParsingError::RequestInvalidType {
				actual_type: value_type.to_string(),
			});
		};

		// -- Check and remove `jsonrpc` property
		if checks.contains(RpcRequestCheckFlags::VERSION) {
			// -- Check and remove `jsonrpc` property
			match obj.remove("jsonrpc") {
				Some(version) => {
					if version.as_str().unwrap_or_default() != "2.0" {
						let (id_val, method) = extract_id_value_and_method(obj);
						return Err(RpcRequestParsingError::VersionInvalid {
							id: id_val,
							method,
							version,
						});
					}
				}
				None => {
					let (id_val, method) = extract_id_value_and_method(obj);
					return Err(RpcRequestParsingError::VersionMissing { id: id_val, method });
				}
			}
		}

		// -- Extract Raw Value Id for now
		let rpc_id_value: Option<Value> = obj.remove("id");

		// -- Check method presence and type
		let method = match obj.remove("method") {
			None => {
				return Err(RpcRequestParsingError::MethodMissing { id: rpc_id_value });
			}
			Some(method_val) => match method_val {
				Value::String(method_name) => method_name,
				other => {
					return Err(RpcRequestParsingError::MethodInvalidType {
						id: rpc_id_value,
						method: other,
					});
				}
			},
		};

		// -- Process RpcId
		// Note: here if we do not have the check_id flag, we are permissive on the rpc_id, and
		let check_id = checks.contains(RpcRequestCheckFlags::ID);
		let id = match rpc_id_value {
			None => {
				if check_id {
					return Err(RpcRequestParsingError::IdMissing { method: Some(method) });
				} else {
					RpcId::Null
				}
			}
			Some(id_value) => match RpcId::from_value(id_value) {
				Ok(rpc_id) => rpc_id,
				Err(err) => {
					if check_id {
						return Err(err);
					} else {
						RpcId::Null
					}
				}
			},
		};

		// -- Extract params (can be absent, which is valid)
		let params = obj.get_mut("params").map(Value::take);

		Ok(RpcRequest { id, method, params })
	}
}

// region:    --- Serialize Custom

impl serde::Serialize for RpcRequest {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// Determine the number of fields: jsonrpc, id, method are always present. params is optional.
		let mut field_count = 3;
		if self.params.is_some() {
			field_count += 1;
		}

		let mut state = serializer.serialize_struct("RpcRequest", field_count)?;

		// Always add "jsonrpc": "2.0"
		state.serialize_field("jsonrpc", "2.0")?;

		state.serialize_field("id", &self.id)?;
		state.serialize_field("method", &self.method)?;

		// Serialize params only if it's Some
		if let Some(params) = &self.params {
			state.serialize_field("params", params)?;
		}

		state.end()
	}
}

// endregion: --- Serialize Custom

bitflags::bitflags! {
	/// Represents a set of flags.
	#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
	pub struct RpcRequestCheckFlags: u32 {
		/// Check the `jsonrpc = "2.0"` property
		const VERSION = 0b00000001;
		/// Check that `id` property is a valid rpc id (string, number, or null)
		/// NOTE: Does not support floating number (although the spec does)
		const ID = 0b00000010;

		/// Check all, te ID and VERSION
		const ALL = Self::VERSION.bits() | Self::ID.bits();
	}
}

// region:    --- Support

// Extract (remove) the id and method.
fn extract_id_value_and_method(mut obj: serde_json::Map<String, Value>) -> (Option<Value>, Option<String>) {
	let id = obj.remove("id");
	// for now be permisive with the method name, so as_str
	let method = obj.remove("method").and_then(|v| v.as_str().map(|s| s.to_string()));
	(id, method)
}

/// Convenient TryFrom, and will execute the Request::from_value,
/// which will perform the version validation.
impl TryFrom<Value> for RpcRequest {
	type Error = RpcRequestParsingError;
	fn try_from(value: Value) -> Result<RpcRequest, RpcRequestParsingError> {
		RpcRequest::from_value(value)
	}
}

// endregion: --- Support
