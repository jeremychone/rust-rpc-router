//! Support module for parsing shared elements of RpcRequest and RpcNotification.

use crate::rpc_message::rpc_request_parsing_error::RpcRequestParsingError;
use crate::support::get_json_type;
use serde_json::{Map, Value};

/// Extracts a value from the map, removing it.
pub(super) fn extract_value(obj: &mut Map<String, Value>, key: &str) -> Option<Value> {
	obj.remove(key)
}

/// Validates the "jsonrpc" property value.
/// Returns `Ok(())` if valid ("2.0"), otherwise returns the invalid value for error reporting.
pub(super) fn validate_version(version_val: Option<Value>) -> Result<(), Option<Value>> {
	match version_val {
		Some(version) => {
			if version.as_str().unwrap_or_default() == "2.0" {
				Ok(())
			} else {
				Err(Some(version)) // Invalid version value
			}
		}
		None => Err(None), // Version missing
	}
}

/// Parses the "method" property value.
/// Returns `Ok(String)` if valid, otherwise returns the invalid value for error reporting.
pub(super) fn parse_method(method_val: Option<Value>) -> Result<String, Option<Value>> {
	match method_val {
		Some(Value::String(method_name)) => Ok(method_name),
		Some(other) => Err(Some(other)), // Invalid type
		None => Err(None),               // Method missing
	}
}

/// Parses the "params" property value.
/// Params can be absent, an array, or an object.
pub(super) fn parse_params(params_val: Option<Value>) -> Result<Option<Value>, RpcRequestParsingError> {
	match params_val {
		None => Ok(None),
		Some(Value::Array(_)) | Some(Value::Object(_)) => Ok(Some(params_val.unwrap())), // Take ownership
		Some(other) => Err(RpcRequestParsingError::ParamsInvalidType {
			actual_type: get_json_type(&other).to_string(),
		}),
	}
}
