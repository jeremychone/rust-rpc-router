use crate::RpcResources;
use serde::Serialize;
use std::any::type_name;

pub trait FromRpcResources {
	fn from_resources(rpc_resources: &RpcResources) -> FromResourcesResult<Self>
	where
		Self: Sized + Clone + Send + Sync + 'static,
	{
		rpc_resources
			.get::<Self>()
			.ok_or_else(FromResourcesError::resource_not_found::<Self>)
	}
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
