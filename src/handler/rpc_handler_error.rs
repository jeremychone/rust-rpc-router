use serde::{Serialize, Serializer};
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type RpcHandlerResult<T> = core::result::Result<T, RpcHandlerError>;

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

#[derive(Debug)]
pub struct RpcHandlerError {
	holder: AnyMap,
	type_name: &'static str,
}

impl RpcHandlerError {
	pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
		self.holder
			.get(&TypeId::of::<T>())
			.and_then(|boxed_any| boxed_any.downcast_ref::<T>())
	}

	pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
		self.holder.remove(&TypeId::of::<T>()).and_then(|boxed_any| {
			// Attempt to downcast the Box<dyn Any> into Box<T>. If successful, take the value out of the box.
			(boxed_any as Box<dyn Any>).downcast::<T>().ok().map(|boxed| *boxed)
		})
	}

	pub fn get_type_name(&self) -> &'static str {
		self.type_name
	}
}

// Implementing Serialize for RpcHandlerError
impl Serialize for RpcHandlerError {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		// By default, serialization will only serialize an informative message regarding the type of error contained,
		// as we do not have more information at this point.
		// NOTE: It is currently uncertain whether we should require serialization for the RpcHandlerError contained type.
		serializer.serialize_str(&format!("RpcHandlerError containing error '{}'", self.type_name))
	}
}

// region:    --- IntoRpcHandlerError

/// A trait with a default implementation that converts any application error
/// into a `RpcHandlerError`. This allows the application code
/// to query and extract the specified application error.
pub trait IntoRpcHandlerError
where
	Self: std::error::Error + Sized + Send + Sync + 'static,
	Self: Serialize,
{
	fn into_handler_error(self) -> RpcHandlerError {
		let mut holder = AnyMap::with_capacity(1);
		let type_name = std::any::type_name::<Self>();
		holder.insert(TypeId::of::<Self>(), Box::new(self));
		RpcHandlerError { holder, type_name }
	}
}

impl IntoRpcHandlerError for RpcHandlerError {
	fn into_handler_error(self) -> RpcHandlerError {
		self
	}
}

// endregion: --- IntoRpcHandlerError

// region:    --- Error Boilerplate

impl core::fmt::Display for RpcHandlerError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for RpcHandlerError {}

// endregion: --- Error Boilerplate
