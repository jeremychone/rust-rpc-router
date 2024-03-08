use serde::Serialize;
use std::any::type_name;

pub trait FromRpcResources<K> {
	fn from_resources(rpc_resources: &K) -> FromResourcesResult<Self>
	where
		Self: Sized;
}

// region:    --- Error

pub type FromResourcesResult<T> = core::result::Result<T, FromResourcesError>;

#[derive(Debug, Serialize)]
pub enum FromResourcesError {
	ResourceNotFound(&'static str),
}

impl FromResourcesError {
	pub fn resource_not_found<T: ?Sized>() -> FromResourcesError {
		let name: &'static str = type_name::<T>();
		Self::ResourceNotFound(name)
	}
}

impl core::fmt::Display for FromResourcesError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for FromResourcesError {}

// endregion: --- Error
