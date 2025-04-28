use crate::RpcId;
use crate::router::{CallError, CallResult, CallSuccess};
use crate::rpc_response::{RpcError, RpcResponseParsingError};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

/// Represents a JSON-RPC 2.0 Response object.
/// It can be either a success or an error response.
/// <https://www.jsonrpc.org/specification#response_object>
#[derive(Debug, Clone, PartialEq)]
pub enum RpcResponse {
	/// Represents a successful JSON-RPC response.
	Success(RpcSuccessResponse),
	/// Represents a JSON-RPC error response.
	Error(RpcErrorResponse),
}

/// Holds the components of a successful JSON-RPC response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcSuccessResponse {
	/// The request ID this response corresponds to.
	pub id: RpcId,

	/// The result payload of the successful RPC call.
	pub result: Value,
}

/// Holds the components of a JSON-RPC error response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcErrorResponse {
	/// The request ID this response corresponds to. Can be `Null` if the request ID couldn't be determined.
	pub id: RpcId,

	/// The error object containing details about the failure.
	pub error: RpcError,
}

// region:    --- Constructors

impl RpcResponse {
	pub fn from_success(id: RpcId, result: Value) -> Self {
		Self::Success(RpcSuccessResponse { id, result })
	}

	pub fn from_error(id: RpcId, error: RpcError) -> Self {
		Self::Error(RpcErrorResponse { id, error })
	}
}

// endregion: --- Constructors

// region:    --- Accessors

impl RpcResponse {
	pub fn is_success(&self) -> bool {
		matches!(self, RpcResponse::Success(_))
	}

	pub fn is_error(&self) -> bool {
		matches!(self, RpcResponse::Error(_))
	}

	pub fn id(&self) -> &RpcId {
		match self {
			RpcResponse::Success(r) => &r.id,
			RpcResponse::Error(r) => &r.id,
		}
	}

	/// Consumes the response and returns its parts: the ID and a `Result` containing
	/// either the success value or the error object.
	pub fn into_parts(self) -> (RpcId, core::result::Result<Value, RpcError>) {
		match self {
			RpcResponse::Success(r) => (r.id, Ok(r.result)),
			RpcResponse::Error(r) => (r.id, Err(r.error)),
		}
	}
}
// endregion: --- Accessors

// region:    --- From Router CallResult/CallSuccess/CallError

impl From<CallSuccess> for RpcResponse {
	/// Converts a successful router `CallSuccess` into a JSON-RPC `RpcResponse::Success`.
	fn from(call_success: CallSuccess) -> Self {
		RpcResponse::from_success(call_success.id, call_success.value)
	}
}

impl From<CallError> for RpcResponse {
	/// Converts a router `CallError` into a JSON-RPC `RpcResponse::Error`.
	fn from(call_error: CallError) -> Self {
		let id = call_error.id.clone(); // Clone id before moving call_error
		let error = RpcError::from(call_error); // Reuse From<CallError> for RpcError
		RpcResponse::from_error(id, error)
	}
}

impl From<CallResult> for RpcResponse {
	/// Converts a router `CallResult` (which is Result<CallSuccess, CallError>)
	/// into the appropriate JSON-RPC `RpcResponse`.
	fn from(call_result: CallResult) -> Self {
		match call_result {
			Ok(call_success) => RpcResponse::from(call_success),
			Err(call_error) => RpcResponse::from(call_error),
		}
	}
}

// endregion: --- From Router CallResult/CallSuccess/CallError

// region:    --- Serde Impls

impl Serialize for RpcResponse {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let mut map = serializer.serialize_map(Some(3))?;
		map.serialize_entry("jsonrpc", "2.0")?;

		match self {
			RpcResponse::Success(RpcSuccessResponse { id, result }) => {
				map.serialize_entry("id", id)?;
				map.serialize_entry("result", result)?;
			}
			RpcResponse::Error(RpcErrorResponse { id, error }) => {
				map.serialize_entry("id", id)?;
				map.serialize_entry("error", error)?;
			}
		}

		map.end()
	}
}

impl<'de> Deserialize<'de> for RpcResponse {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		struct RpcResponseVisitor;

		impl<'de> Visitor<'de> for RpcResponseVisitor {
			type Value = RpcResponse;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("a JSON-RPC 2.0 response object")
			}

			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: MapAccess<'de>,
			{
				let mut version: Option<String> = None;
				let mut id_val: Option<Value> = None;
				let mut result_val: Option<Value> = None;
				let mut error_val: Option<Value> = None;

				while let Some(key) = map.next_key::<String>()? {
					match key.as_str() {
						"jsonrpc" => {
							if version.is_some() {
								return Err(serde::de::Error::duplicate_field("jsonrpc"));
							}
							version = Some(map.next_value()?);
						}
						"id" => {
							if id_val.is_some() {
								return Err(serde::de::Error::duplicate_field("id"));
							}
							// Deserialize as Value first to handle String/Number/Null correctly
							id_val = Some(map.next_value()?);
						}
						"result" => {
							if result_val.is_some() {
								return Err(serde::de::Error::duplicate_field("result"));
							}
							result_val = Some(map.next_value()?);
						}
						"error" => {
							if error_val.is_some() {
								return Err(serde::de::Error::duplicate_field("error"));
							}
							// Deserialize error as Value for now, parse fully later
							error_val = Some(map.next_value()?);
						}
						_ => {
							// Ignore unknown fields
							let _: Value = map.next_value()?;
						}
					}
				}

				// Validate jsonrpc version
				let id_for_error = id_val.as_ref().and_then(|v| RpcId::from_value(v.clone()).ok());
				match version.as_deref() {
					Some("2.0") => {} // OK
					Some(v) => {
						return Err(serde::de::Error::custom(
							RpcResponseParsingError::InvalidJsonRpcVersion {
								id: id_for_error,
								expected: "2.0",
								actual: Some(Value::String(v.to_string())),
							},
						));
					}
					None => {
						return Err(serde::de::Error::custom(
							RpcResponseParsingError::MissingJsonRpcVersion { id: id_for_error },
						));
					}
				};

				// Parse id
				let id = match id_val {
					Some(v) => RpcId::from_value(v)
						.map_err(|e| serde::de::Error::custom(RpcResponseParsingError::InvalidId(e)))?,
					None => return Err(serde::de::Error::custom(RpcResponseParsingError::MissingId)),
				};

				// Determine if Success or Error
				match (result_val, error_val) {
					(Some(result), None) => Ok(RpcResponse::Success(RpcSuccessResponse { id, result })),
					(None, Some(error_value)) => {
						// Now parse the error object from the Value
						let error: RpcError = serde_json::from_value(error_value)
							.map_err(|e| serde::de::Error::custom(RpcResponseParsingError::InvalidErrorObject(e)))?;
						Ok(RpcResponse::Error(RpcErrorResponse { id, error }))
					}
					(Some(_), Some(_)) => Err(serde::de::Error::custom(RpcResponseParsingError::BothResultAndError {
						id: id.clone(),
					})),
					(None, None) => Err(serde::de::Error::custom(
						RpcResponseParsingError::MissingResultAndError { id: id.clone() },
					)),
				}
			}
		}

		deserializer.deserialize_map(RpcResponseVisitor)
	}
}

// endregion: --- Serde Impls

// region:    --- Tests

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Error as RouterError; // Import router Error
	use serde_json::{from_value, json, to_value};

	type TestResult<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	// Helper to create CallError easily
	fn create_call_error(id: impl Into<RpcId>, method: &str, error: RouterError) -> CallError {
		CallError {
			id: id.into(),
			method: method.to_string(),
			error,
		}
	}

	#[test]
	fn test_rpc_response_success_ser_de() -> TestResult<()> {
		// -- Setup & Fixtures
		let id = RpcId::Number(1);
		let result_val = json!({"data": "ok"});
		let response = RpcResponse::from_success(id.clone(), result_val.clone());
		let expected_json = json!({
			"jsonrpc": "2.0",
			"id": 1,
			"result": {"data": "ok"}
		});

		// -- Exec: Serialize
		let serialized_value = to_value(&response)?;

		// -- Check: Serialize
		assert_eq!(serialized_value, expected_json);

		// -- Exec: Deserialize
		let deserialized_response: RpcResponse = from_value(serialized_value)?;

		// -- Check: Deserialize
		assert_eq!(deserialized_response, response);
		assert_eq!(deserialized_response.id(), &id);
		assert!(deserialized_response.is_success());
		assert!(!deserialized_response.is_error());
		let (resp_id, resp_result) = deserialized_response.into_parts();
		assert_eq!(resp_id, id);
		assert_eq!(resp_result.unwrap(), result_val);

		Ok(())
	}

	#[test]
	fn test_rpc_response_error_ser_de() -> TestResult<()> {
		// -- Setup & Fixtures
		let id = RpcId::String("req-abc".into());
		let rpc_error = RpcError {
			code: -32601,
			message: "Method not found".to_string(),
			data: Some(json!("method_name")),
		};
		let response = RpcResponse::from_error(id.clone(), rpc_error.clone());
		let expected_json = json!({
			"jsonrpc": "2.0",
			"id": "req-abc",
			"error": {
				"code": -32601,
				"message": "Method not found",
				"data": "method_name"
			}
		});

		// -- Exec: Serialize
		let serialized_value = to_value(&response)?;

		// -- Check: Serialize
		assert_eq!(serialized_value, expected_json);

		// -- Exec: Deserialize
		let deserialized_response: RpcResponse = from_value(serialized_value)?;

		// -- Check: Deserialize
		assert_eq!(deserialized_response, response);
		assert_eq!(deserialized_response.id(), &id);
		assert!(!deserialized_response.is_success());
		assert!(deserialized_response.is_error());
		let (resp_id, resp_result) = deserialized_response.into_parts();
		assert_eq!(resp_id, id);
		assert_eq!(resp_result.unwrap_err(), rpc_error);

		Ok(())
	}

	#[test]
	fn test_rpc_response_error_ser_de_no_data() -> TestResult<()> {
		// -- Setup & Fixtures
		let id = RpcId::Null;
		let rpc_error = RpcError {
			code: -32700,
			message: "Parse error".to_string(),
			data: None, // No data
		};
		let response = RpcResponse::from_error(id.clone(), rpc_error.clone());
		let expected_json = json!({
			"jsonrpc": "2.0",
			"id": null,
			"error": {
				"code": -32700,
				"message": "Parse error"
				// "data" field is omitted
			}
		});

		// -- Exec: Serialize
		let serialized_value = to_value(&response)?;

		// -- Check: Serialize
		assert_eq!(serialized_value, expected_json);

		// -- Exec: Deserialize
		let deserialized_response: RpcResponse = from_value(serialized_value)?;

		// -- Check: Deserialize
		assert_eq!(deserialized_response, response);
		assert_eq!(deserialized_response.id(), &id);
		assert!(deserialized_response.is_error());
		let (resp_id, resp_result) = deserialized_response.into_parts();
		assert_eq!(resp_id, id);
		assert_eq!(resp_result.unwrap_err(), rpc_error);

		Ok(())
	}

	#[test]
	fn test_rpc_response_de_invalid() {
		// -- Setup & Fixtures
		let invalid_jsons = vec![
			// Missing jsonrpc
			json!({"id": 1, "result": "ok"}),
			// Invalid jsonrpc version
			json!({"jsonrpc": "1.0", "id": 1, "result": "ok"}),
			// Missing id
			json!({"jsonrpc": "2.0", "result": "ok"}),
			// Missing result and error
			json!({"jsonrpc": "2.0", "id": 1}),
			// Both result and error
			json!({"jsonrpc": "2.0", "id": 1, "result": "ok", "error": {"code": 1, "message": "err"}}),
			// Invalid error object (e.g., wrong type)
			json!({"jsonrpc": "2.0", "id": 1, "error": "not an object"}),
			// Invalid error object (missing code)
			json!({"jsonrpc": "2.0", "id": 1, "error": {"message": "err"}}),
			// Invalid error object (missing message)
			json!({"jsonrpc": "2.0", "id": 1, "error": {"code": 1}}),
			// Invalid id type
			json!({"jsonrpc": "2.0", "id": [1,2], "result": "ok"}),
		];

		// -- Exec & Check
		for json_value in invalid_jsons {
			let result: Result<RpcResponse, _> = from_value(json_value.clone());
			assert!(result.is_err(), "Expected error for invalid JSON: {}", json_value);
		}
	}

	// region:    --- From Router Call Tests
	#[test]
	fn test_from_call_success() -> TestResult<()> {
		// -- Setup & Fixtures
		let call_success = CallSuccess {
			id: RpcId::Number(101),
			method: "test_method".to_string(),
			value: json!({"success": true}),
		};

		// -- Exec
		let rpc_response = RpcResponse::from(call_success);

		// -- Check
		match rpc_response {
			RpcResponse::Success(RpcSuccessResponse { id, result }) => {
				assert_eq!(id, RpcId::Number(101));
				assert_eq!(result, json!({"success": true}));
			}
			RpcResponse::Error(_) => panic!("Expected RpcResponse::Success"),
		}
		Ok(())
	}

	#[test]
	fn test_from_call_error() -> TestResult<()> {
		// -- Setup & Fixtures
		let call_error = create_call_error(102, "test_method", RouterError::MethodUnknown);

		// -- Exec
		let rpc_response = RpcResponse::from(call_error);

		// -- Check
		match rpc_response {
			RpcResponse::Error(RpcErrorResponse { id, error }) => {
				assert_eq!(id, RpcId::Number(102));
				assert_eq!(error.code, RpcError::CODE_METHOD_NOT_FOUND);
				assert_eq!(error.message, "Method not found");
				assert!(error.data.is_some()); // contains RouterError::MethodUnknown display
			}
			RpcResponse::Success(_) => panic!("Expected RpcResponse::Error"),
		}
		Ok(())
	}

	#[test]
	fn test_from_call_result_ok() -> TestResult<()> {
		// -- Setup & Fixtures
		let call_result: CallResult = Ok(CallSuccess {
			id: 103.into(),
			method: "test_method".to_string(),
			value: json!("ok_data"),
		});

		// -- Exec
		let rpc_response = RpcResponse::from(call_result);

		// -- Check
		match rpc_response {
			RpcResponse::Success(RpcSuccessResponse { id, result }) => {
				assert_eq!(id, RpcId::Number(103));
				assert_eq!(result, json!("ok_data"));
			}
			RpcResponse::Error(_) => panic!("Expected RpcResponse::Success"),
		}
		Ok(())
	}

	#[test]
	fn test_from_call_result_err() -> TestResult<()> {
		// -- Setup & Fixtures
		let call_result: CallResult = Err(create_call_error(
			"err-104",
			"test_method",
			RouterError::ParamsMissingButRequested,
		));

		// -- Exec
		let rpc_response = RpcResponse::from(call_result);

		// -- Check
		match rpc_response {
			RpcResponse::Error(RpcErrorResponse { id, error }) => {
				assert_eq!(id, RpcId::String("err-104".into()));
				assert_eq!(error.code, RpcError::CODE_INVALID_PARAMS);
				assert_eq!(error.message, "Invalid params");
				assert!(error.data.is_some()); // contains RouterError::ParamsMissingButRequested display
			}
			RpcResponse::Success(_) => panic!("Expected RpcResponse::Error"),
		}
		Ok(())
	}
	// endregion: --- From Router Call Tests
}

// endregion: --- Tests
