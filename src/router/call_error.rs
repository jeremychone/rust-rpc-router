use crate::CallResponse;
use serde_json::Value;

pub type CallResult = core::result::Result<CallResponse, CallError>;

#[derive(Debug)]
pub struct CallError {
	pub id: Value,
	pub method: String,
	pub error: crate::Error,
}

// region:    --- Error Boilerplate

impl core::fmt::Display for CallError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for CallError {}

// endregion: --- Error Boilerplate
