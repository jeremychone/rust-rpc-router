use crate::RpcId;
use serde::Serialize;
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

/// Error type for failures during `RpcResponse` parsing or validation.
#[serde_as]
#[derive(Debug, Serialize)]
pub enum RpcResponseParsingError {
	InvalidJsonRpcVersion {
		id: Option<RpcId>,
		expected: &'static str,
		actual: Option<Value>,
	},
	MissingJsonRpcVersion {
		id: Option<RpcId>,
	},
	MissingId,
	InvalidId(#[serde_as(as = "DisplayFromStr")] crate::RpcRequestParsingError),
	MissingResultAndError {
		id: RpcId,
	},
	BothResultAndError {
		id: RpcId,
	},
	InvalidErrorObject(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
	Serde(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

// region:    --- Error Boilerplate

impl core::fmt::Display for RpcResponseParsingError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for RpcResponseParsingError {}

// endregion: --- Error Boilerplate

// region:    --- Froms

impl From<serde_json::Error> for RpcResponseParsingError {
	fn from(e: serde_json::Error) -> Self {
		RpcResponseParsingError::Serde(e)
	}
}

impl From<crate::RpcRequestParsingError> for RpcResponseParsingError {
	fn from(e: crate::RpcRequestParsingError) -> Self {
		RpcResponseParsingError::InvalidId(e)
	}
}

// endregion: --- Froms
