use crate::{CallResponse, RpcId};

pub type CallResult = core::result::Result<CallResponse, CallError>;

/// The Error type returned by `rpc_router.call...` functions.
///
/// NOTE: CallResponse & CallError
///       are not designed to be the JSON-RPC Response
///       or Error, but to provide the necessary context
///       to build those, as well as the useful `method name`
///       context for tracing/login.
#[derive(Debug)]
pub struct CallError {
	pub id: RpcId,
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

