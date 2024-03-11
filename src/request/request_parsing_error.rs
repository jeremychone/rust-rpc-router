use serde::Serialize;
use serde_json::Value;
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Serialize)]
pub enum RequestParsingError {
	VersionMissing {
		id: Option<Value>,
		method: Option<String>,
	},
	VersionInvalid {
		id: Option<Value>,
		method: Option<String>,
		version: Value,
	},
	Parse(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

impl core::fmt::Display for RequestParsingError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for RequestParsingError {}
