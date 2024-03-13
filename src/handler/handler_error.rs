use serde::{Serialize, Serializer};
use serde_json::Value;
use std::any::{Any, TypeId};
use std::collections::HashMap;

pub type HandlerResult<T> = core::result::Result<T, HandlerError>;

type AnyMap = HashMap<TypeId, Box<dyn Any + Send + Sync>>;

#[derive(Debug)]
pub struct HandlerError {
	holder: AnyMap,
	type_name: &'static str,
}

impl HandlerError {
	pub fn new<T>(val: T) -> HandlerError
	where
		T: Any + Send + Sync,
	{
		let mut holder = AnyMap::with_capacity(1);
		let type_name = std::any::type_name::<T>();
		holder.insert(TypeId::of::<T>(), Box::new(val));
		HandlerError { holder, type_name }
	}
}

impl HandlerError {
	/// Returns an option containing a reference if the error contained within this error
	/// matches the requested type.
	pub fn get<T: Any + Send + Sync>(&self) -> Option<&T> {
		self.holder
			.get(&TypeId::of::<T>())
			.and_then(|boxed_any| boxed_any.downcast_ref::<T>())
	}

	/// Same as `get::<T>()` but remove the date so that it returns a owned value.
	pub fn remove<T: Any + Send + Sync>(&mut self) -> Option<T> {
		self.holder.remove(&TypeId::of::<T>()).and_then(|boxed_any| {
			// Attempt to downcast the Box<dyn Any> into Box<T>. If successful, take the value out of the box.
			(boxed_any as Box<dyn Any>).downcast::<T>().ok().map(|boxed| *boxed)
		})
	}

	/// Return the type name of the error hold by this RpcHandlerError
	pub fn type_name(&self) -> &'static str {
		self.type_name
	}
}

// Implementing Serialize for RpcHandlerError
impl Serialize for HandlerError {
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
pub trait IntoHandlerError
where
	Self: Sized + Send + Sync + 'static,
{
	fn into_handler_error(self) -> HandlerError {
		HandlerError::new(self)
	}
}

impl IntoHandlerError for HandlerError {
	fn into_handler_error(self) -> HandlerError {
		self
	}
}

impl IntoHandlerError for String {
	fn into_handler_error(self) -> HandlerError {
		HandlerError::new(self)
	}
}

impl IntoHandlerError for &'static str {
	fn into_handler_error(self) -> HandlerError {
		HandlerError::new(self)
	}
}

impl IntoHandlerError for Value {
	fn into_handler_error(self) -> HandlerError {
		HandlerError::new(self)
	}
}

// endregion: --- IntoRpcHandlerError

// region:    --- Error Boilerplate

impl core::fmt::Display for HandlerError {
	fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for HandlerError {}

// endregion: --- Error Boilerplate
