use serde::Serialize;
use serde_json::Value;
use serde_with::{DisplayFromStr, serde_as};

/// The RPC Request Parsing error is used when utilizing `value.try_into()?` or `Request::from_value(value)`.
/// The design intent is to validate and provide as much context as possible when a specific validation fails.
///
/// Note: By design, we do not capture the "params" because they could be indefinitely large.
///
/// Note: In future releases, the capture of Value objects or arrays for those error variants
///       will be replaced with Value::String containing a message such as
///       `"[object/array redacted, 'id' must be of type number, string, or equal to null]"`
///       or `"[object/array redacted, 'method' must be of type string]"`
///       This approach aims to provide sufficient context for debugging the issue while preventing
///       the capture of indefinitely large values in the logs.
#[serde_as]
#[derive(Debug, Serialize)]
pub enum RequestParsingError {
	RequestInvalidType {
		actual_type: String,
	},

	VersionMissing {
		id: Option<Value>, // Keep Value here as RpcId parsing might not have happened yet
		method: Option<String>,
	},
	VersionInvalid {
		id: Option<Value>, // Keep Value here
		method: Option<String>,
		version: Value,
	},

	MethodMissing {
		id: Option<Value>, // Keep Value here
	},
	MethodInvalidType {
		id: Option<Value>, // Keep Value here
		method: Value,
	},

	MethodInvalid {
		actual: String,
	},

	IdMissing {
		method: Option<String>,
	},
	IdInvalid {
		actual: String,
		cause: String,
	},

	Parse(#[serde_as(as = "DisplayFromStr")] serde_json::Error), // Generic serde error if basic JSON is invalid
}

impl core::fmt::Display for RequestParsingError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for RequestParsingError {}
