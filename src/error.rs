use crate::FromResourcesError;
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

	// -- Others
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
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

// region:    --- Error Boilerplate

impl core::fmt::Display for Error {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
