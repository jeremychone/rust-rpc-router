use crate::{FromResourcesError, HandlerError};
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize)]
pub enum Error {
	// -- Into Params
	ParamsDeserialize(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
	ParamsMissingButRequested,

	// -- Router
	MethodUnknown,

	// -- Handler
	FromResources(FromResourcesError),
	HandlerResultSerialize(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
	Handler(#[serde_as(as = "DisplayFromStr")] HandlerError),
}

// region:    --- Froms

impl From<HandlerError> for Error {
	fn from(val: HandlerError) -> Self {
		Self::Handler(val)
	}
}

impl From<FromResourcesError> for Error {
	fn from(val: FromResourcesError) -> Self {
		Self::FromResources(val)
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
