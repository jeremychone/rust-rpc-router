use crate::{FromResourcesError, RpcHandlerError};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
	// -- RPC Router
	RpcMethodUnknown(String),
	RpcIntoParamsMissing,

	// -- FromResources
	FromResources(FromResourcesError),

	// -- Handler
	Handler(#[serde_as(as = "DisplayFromStr")] RpcHandlerError),

	// -- Others
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

// region:    --- Froms

impl From<RpcHandlerError> for Error {
	fn from(val: RpcHandlerError) -> Self {
		Self::Handler(val)
	}
}

impl From<FromResourcesError> for Error {
	fn from(val: FromResourcesError) -> Self {
		Self::FromResources(val)
	}
}

impl From<serde_json::Error> for Error {
	fn from(val: serde_json::Error) -> Self {
		Self::SerdeJson(val)
	}
}

// endregion: --- Froms

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
