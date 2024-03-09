use crate::Resources;
use serde::Serialize;
use std::any::type_name;

pub trait FromResources {
	fn from_resources(resources: &Resources) -> FromResourcesResult<Self>
	where
		Self: Sized + Clone + Send + Sync + 'static,
	{
		resources
			.get::<Self>()
			.ok_or_else(FromResourcesError::resource_not_found::<Self>)
	}
}

/// Implements `FromResources` to allow requesting Option<T>
/// when T implements FromResources.
impl<T> FromResources for Option<T>
where
	T: FromResources,
	T: Sized + Clone + Send + Sync + 'static,
{
	fn from_resources(resources: &Resources) -> FromResourcesResult<Self> {
		Ok(resources.get::<T>())
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
