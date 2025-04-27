//! Represents a JSON-RPC Notification object.

use crate::RpcRequest;
use crate::rpc_message::rpc_request_parsing_error::RpcRequestParsingError;
use crate::rpc_message::support::{extract_value, parse_method, parse_params, validate_version};
use crate::support::get_json_type;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

/// Represents a JSON-RPC Notification object, which is a request without an `id`.
/// Notifications are inherently unreliable as no response is expected.
///
/// Note: Derives `Deserialize` for convenience but custom parsing logic is in `from_value`.
/// Does not derive `Serialize` as a custom implementation is needed to add `jsonrpc: "2.0"`.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct RpcNotification {
	pub method: String,
	pub params: Option<Value>,
}

impl RpcNotification {
	/// Parses a `serde_json::Value` into an `RpcNotification`.
	///
	/// This method performs checks according to the JSON-RPC 2.0 specification:
	/// 1. Checks if the input is a JSON object.
	/// 2. Validates the presence and value of the `"jsonrpc": "2.0"` field.
	/// 3. Validates the presence and type of the `"method"` field (must be a string).
	/// 4. Validates the type of the `"params"` field (must be array or object if present).
	/// 5. Ensures no `"id"` field is present (as it's a notification).
	///
	/// # Errors
	/// Returns `RpcRequestParsingError` if any validation fails.
	pub fn from_value(value: Value) -> Result<RpcNotification, RpcRequestParsingError> {
		let value_type = get_json_type(&value);

		let Value::Object(mut obj) = value else {
			return Err(RpcRequestParsingError::RequestInvalidType {
				actual_type: value_type.to_string(),
			});
		};

		// -- Check Version
		let version_val = extract_value(&mut obj, "jsonrpc");
		if let Err(version_result) = validate_version(version_val) {
			// Attempt to get method for better error context (id is None for notifications)
			let method_val = extract_value(&mut obj, "method");
			let method = method_val.and_then(|v| v.as_str().map(|s| s.to_string()));
			return match version_result {
				Some(v) => Err(RpcRequestParsingError::VersionInvalid {
					id: None, // Notifications have no ID
					method,
					version: v,
				}),
				None => Err(RpcRequestParsingError::VersionMissing {
					id: None, // Notifications have no ID
					method,
				}),
			};
		}

		// -- Check Method
		let method_val = extract_value(&mut obj, "method");
		let method = match parse_method(method_val) {
			Ok(m) => m,
			Err(method_result) => {
				return match method_result {
					Some(m) => Err(RpcRequestParsingError::MethodInvalidType {
						id: None, // Notifications have no ID
						method: m,
					}),
					None => Err(RpcRequestParsingError::MethodMissing { id: None }), // Notifications have no ID
				};
			}
		};

		// -- Check Params
		let params_val = extract_value(&mut obj, "params");
		let params = parse_params(params_val)?; // Propagates ParamsInvalidType error

		// -- Check for disallowed ID field
		if let Some(id_val) = extract_value(&mut obj, "id") {
			return Err(RpcRequestParsingError::NotificationHasId {
				method: Some(method),
				id: id_val,
			});
		}

		Ok(RpcNotification { method, params })
	}
}

// region:    --- Serialize Custom

impl Serialize for RpcNotification {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// Determine the number of fields: jsonrpc, method are always present. params is optional.
		let mut field_count = 2;
		if self.params.is_some() {
			field_count += 1;
		}

		let mut state = serializer.serialize_struct("RpcNotification", field_count)?;

		// Always add "jsonrpc": "2.0"
		state.serialize_field("jsonrpc", "2.0")?;

		state.serialize_field("method", &self.method)?;

		// Serialize params only if it's Some
		if let Some(params) = &self.params {
			state.serialize_field("params", params)?;
		}

		state.end()
	}
}

// endregion: --- Serialize Custom

// region:    --- Froms

impl From<RpcRequest> for RpcNotification {
	fn from(request: RpcRequest) -> Self {
		RpcNotification {
			method: request.method,
			params: request.params,
		}
	}
}

// endregion: --- Froms

// region:    --- TryFrom

/// Convenient TryFrom, performs strict JSON-RPC 2.0 validation via `RpcNotification::from_value`.
impl TryFrom<Value> for RpcNotification {
	type Error = RpcRequestParsingError;
	fn try_from(value: Value) -> Result<RpcNotification, RpcRequestParsingError> {
		RpcNotification::from_value(value)
	}
}

// endregion: --- TryFrom

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::rpc_message::RpcRequestParsingError;
	use serde_json::{json, to_value};

	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	// -- Test Data
	fn notif_value_ok_params_some() -> Value {
		json!({
			"jsonrpc": "2.0",
			"method": "updateState",
			"params": {"value": 123}
		})
	}

	fn notif_value_ok_params_none() -> Value {
		json!({
			"jsonrpc": "2.0",
			"method": "ping"
		})
	}

	fn notif_value_ok_params_arr() -> Value {
		json!({
			"jsonrpc": "2.0",
			"method": "notifyUsers",
			"params": ["user1", "user2"]
		})
	}

	fn notif_value_fail_id_present() -> Value {
		json!({
			"jsonrpc": "2.0",
			"id": 888, // ID is not allowed in notifications
			"method": "updateState",
			"params": {"value": 123}
		})
	}

	fn notif_value_fail_version_missing() -> Value {
		json!({
			// "jsonrpc": "2.0", // Missing
			"method": "updateState"
		})
	}

	fn notif_value_fail_version_invalid() -> Value {
		json!({
			"jsonrpc": "1.0", // Invalid
			"method": "updateState"
		})
	}

	fn notif_value_fail_method_missing() -> Value {
		json!({
			"jsonrpc": "2.0"
			// "method": "updateState" // Missing
		})
	}

	fn notif_value_fail_method_invalid() -> Value {
		json!({
			"jsonrpc": "2.0",
			"method": 123 // Invalid type
		})
	}

	fn notif_value_fail_params_invalid() -> Value {
		json!({
			"jsonrpc": "2.0",
			"method": "update",
			"params": "not-array-or-object" // Invalid type
		})
	}

	// region:    --- Serialize Tests
	#[test]
	fn test_rpc_notification_serialize_ok_params_some() -> Result<()> {
		// -- Setup & Fixtures
		let notif = RpcNotification {
			method: "updateState".to_string(),
			params: Some(json!({"value": 123})),
		};

		// -- Exec
		let value = to_value(notif)?;

		// -- Check
		assert_eq!(value, notif_value_ok_params_some());
		Ok(())
	}

	#[test]
	fn test_rpc_notification_serialize_ok_params_none() -> Result<()> {
		// -- Setup & Fixtures
		let notif = RpcNotification {
			method: "ping".to_string(),
			params: None,
		};

		// -- Exec
		let value = to_value(notif)?;

		// -- Check
		assert_eq!(value, notif_value_ok_params_none());
		Ok(())
	}

	#[test]
	fn test_rpc_notification_serialize_ok_params_arr() -> Result<()> {
		// -- Setup & Fixtures
		let notif = RpcNotification {
			method: "notifyUsers".to_string(),
			params: Some(json!(["user1", "user2"])),
		};

		// -- Exec
		let value = to_value(notif)?;

		// -- Check
		assert_eq!(value, notif_value_ok_params_arr());
		Ok(())
	}
	// endregion: --- Serialize Tests

	// region:    --- Deserialize (from_value) Tests
	#[test]
	fn test_rpc_notification_from_value_ok_params_some() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_ok_params_some();
		let expected = RpcNotification {
			method: "updateState".to_string(),
			params: Some(json!({"value": 123})),
		};

		// -- Exec
		let notification = RpcNotification::from_value(value)?;

		// -- Check
		assert_eq!(notification, expected);
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_ok_params_none() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_ok_params_none();
		let expected = RpcNotification {
			method: "ping".to_string(),
			params: None,
		};

		// -- Exec
		let notification = RpcNotification::from_value(value)?;

		// -- Check
		assert_eq!(notification, expected);
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_ok_params_arr() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_ok_params_arr();
		let expected = RpcNotification {
			method: "notifyUsers".to_string(),
			params: Some(json!(["user1", "user2"])),
		};

		// -- Exec
		let notification = RpcNotification::from_value(value)?;

		// -- Check
		assert_eq!(notification, expected);
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_id_present() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_fail_id_present();

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::NotificationHasId { method: Some(_), id: _ })
		));
		if let Err(RpcRequestParsingError::NotificationHasId { method, id }) = result {
			assert_eq!(method.unwrap(), "updateState");
			assert_eq!(id, json!(888));
		} else {
			panic!("Expected NotificationHasId error");
		}
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_version_missing() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_fail_version_missing();

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::VersionMissing {
				id: None,
				method: Some(_)
			})
		));
		if let Err(RpcRequestParsingError::VersionMissing { id, method }) = result {
			assert!(id.is_none());
			assert_eq!(method.unwrap(), "updateState");
		} else {
			panic!("Expected VersionMissing error");
		}
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_version_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_fail_version_invalid();

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::VersionInvalid {
				id: None,
				method: Some(_),
				version: _
			})
		));
		if let Err(RpcRequestParsingError::VersionInvalid { id, method, version }) = result {
			assert!(id.is_none());
			assert_eq!(method.unwrap(), "updateState");
			assert_eq!(version, json!("1.0"));
		} else {
			panic!("Expected VersionInvalid error");
		}
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_method_missing() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_fail_method_missing();

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::MethodMissing { id: None })
		));
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_method_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_fail_method_invalid();

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::MethodInvalidType { id: None, method: _ })
		));
		if let Err(RpcRequestParsingError::MethodInvalidType { id, method }) = result {
			assert!(id.is_none());
			assert_eq!(method, json!(123));
		} else {
			panic!("Expected MethodInvalidType error");
		}
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_params_invalid() -> Result<()> {
		// -- Setup & Fixtures
		let value = notif_value_fail_params_invalid();

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::ParamsInvalidType { actual_type: _ })
		));
		if let Err(RpcRequestParsingError::ParamsInvalidType { actual_type }) = result {
			assert_eq!(actual_type, "String");
		} else {
			panic!("Expected ParamsInvalidType error");
		}
		Ok(())
	}

	#[test]
	fn test_rpc_notification_from_value_fail_not_object() -> Result<()> {
		// -- Setup & Fixtures
		let value = json!("not an object");

		// -- Exec
		let result = RpcNotification::from_value(value);

		// -- Check
		assert!(matches!(
			result,
			Err(RpcRequestParsingError::RequestInvalidType { actual_type: _ })
		));
		if let Err(RpcRequestParsingError::RequestInvalidType { actual_type }) = result {
			assert_eq!(actual_type, "String");
		} else {
			panic!("Expected RequestInvalidType error");
		}
		Ok(())
	}
	// endregion: --- Deserialize (from_value) Tests
}
// endregion: --- Tests
