use crate::Error;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Represents the JSON-RPC 2.0 Error Object.
/// <https://www.jsonrpc.org/specification#error_object>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RpcError {
	/// A Number that indicates the error type that occurred.
	pub code: i64,

	/// A String providing a short description of the error.
	pub message: String,

	/// A Primitive or Structured value that contains additional information about the error.
	/// This may be omitted.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub data: Option<Value>,
}

// region:    --- Predefined Errors
// https://www.jsonrpc.org/specification#error_object
impl RpcError {
	pub const CODE_PARSE_ERROR: i64 = -32700;
	pub const CODE_INVALID_REQUEST: i64 = -32600;
	pub const CODE_METHOD_NOT_FOUND: i64 = -32601;
	pub const CODE_INVALID_PARAMS: i64 = -32602;
	pub const CODE_INTERNAL_ERROR: i64 = -32603;
	// -32000 to -32099: Server error. Reserved for implementation-defined server-errors.

	pub fn from_parse_error(data: Option<Value>) -> Self {
		Self {
			code: Self::CODE_PARSE_ERROR,
			message: "Parse error".to_string(),
			data,
		}
	}

	pub fn from_invalid_request(data: Option<Value>) -> Self {
		Self {
			code: Self::CODE_INVALID_REQUEST,
			message: "Invalid Request".to_string(),
			data,
		}
	}

	pub fn from_method_not_found(data: Option<Value>) -> Self {
		Self {
			code: Self::CODE_METHOD_NOT_FOUND,
			message: "Method not found".to_string(),
			data,
		}
	}

	pub fn from_invalid_params(data: Option<Value>) -> Self {
		Self {
			code: Self::CODE_INVALID_PARAMS,
			message: "Invalid params".to_string(),
			data,
		}
	}

	pub fn from_internal_error(data: Option<Value>) -> Self {
		Self {
			code: Self::CODE_INTERNAL_ERROR,
			message: "Internal error".to_string(),
			data,
		}
	}

	/// Helper to create an RpcError with optional data representing the original error string.
	fn new(code: i64, message: impl Into<String>, error: Option<&dyn std::error::Error>) -> Self {
		let data = error.map(|e| json!(e.to_string()));
		Self {
			code,
			message: message.into(),
			data,
		}
	}
}
// endregion: --- Predefined Errors

// region:    --- From RouterError

impl From<&Error> for RpcError {
	/// Converts a router `Error` into a JSON-RPC `RpcError`.
	fn from(err: &Error) -> Self {
		match err {
			Error::ParamsParsing(p) => Self::new(Self::CODE_INVALID_PARAMS, "Invalid params", Some(p)),
			Error::ParamsMissingButRequested => Self::new(Self::CODE_INVALID_PARAMS, "Invalid params", Some(err)),
			Error::MethodUnknown => Self::new(Self::CODE_METHOD_NOT_FOUND, "Method not found", Some(err)),
			Error::FromResources(fr_err) => Self::new(Self::CODE_INTERNAL_ERROR, "Internal error", Some(fr_err)),
			Error::HandlerResultSerialize(s_err) => Self::new(Self::CODE_INTERNAL_ERROR, "Internal error", Some(s_err)),
			// NOTE: For HandlerError, we use a generic Internal Error.
			//       A future enhancement could involve a trait on the error
			//       wrapped by HandlerError to provide specific RpcError details.
			Error::Handler(h_err) => Self::new(Self::CODE_INTERNAL_ERROR, "Internal error", Some(h_err)),
		}
	}
}

// endregion: --- From RouterError

// region:    --- From CallError

// We also implement From<CallError> for RpcError for convenience, although
// the direct conversion from CallError to RpcResponse is often more useful.
impl From<crate::CallError> for RpcError {
	fn from(call_error: crate::CallError) -> Self {
		// Reuse the logic from From<&RouterError>
		RpcError::from(&call_error.error)
	}
}

impl From<&crate::CallError> for RpcError {
	fn from(call_error: &crate::CallError) -> Self {
		// Reuse the logic from From<&RouterError>
		RpcError::from(&call_error.error)
	}
}

// endregion: --- From CallError
