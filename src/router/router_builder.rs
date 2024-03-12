use crate::handler::RpcHandlerWrapperTrait;
use crate::router::router_inner::RouterInner;
use crate::{FromResources, Handler, ResourcesBuilder, ResourcesInner, Router};

#[derive(Debug, Default)]
pub struct RouterBuilder {
	inner: RouterInner,
	base_resources_inner: ResourcesInner,
}

impl RouterBuilder {
	/// Add a dyn_handler to the router builder.
	///
	/// ```
	/// RouterBuilder::default().add_dyn("method_name", my_handler_fn.into_dyn());
	/// ```
	///
	/// Note: This is the preferred way to add handlers to the router, as it
	///       avoids monomorphization of the add function.
	///       The `RouterInner` also has a `.add()` as a convenience function to just pass the function.
	///       See `RouterInner::add` for more details.
	pub fn append_dyn(mut self, name: &'static str, dyn_handler: Box<dyn RpcHandlerWrapperTrait>) -> Self {
		self.inner.append_dyn(name, dyn_handler);
		self
	}

	/// Add a route (name, handler function) to the builder
	///
	/// ```
	/// RouterBuilder::default().add("method_name", my_handler_fn);
	/// ```
	///
	/// Note: This is a convenient add function variant with generics,
	///       and there will be monomorphed versions of this function
	///       for each type passed. Use `RouterInner::add_dyn` to avoid this.
	pub fn append<F, T, P, R>(mut self, name: &'static str, handler: F) -> Self
	where
		F: Handler<T, P, R> + Clone + Send + Sync + 'static,
		T: Send + Sync + 'static,
		P: Send + Sync + 'static,
		R: Send + Sync + 'static,
	{
		self.inner.append_dyn(name, handler.into_dyn());
		self
	}

	pub fn append_resource<T>(mut self, val: T) -> Self
	where
		T: FromResources + Clone + Send + Sync + 'static,
	{
		self.base_resources_inner.insert(val);
		self
	}

	/// Resets the router resources with the contents of this ResourcesBuilder.
	/// Ensure to call append_resource afterwards if you want them to be included.
	pub fn set_resources_builder(mut self, resources_builder: ResourcesBuilder) -> Self {
		self.base_resources_inner = resources_builder.resources_inner;
		self
	}

	/// Extends this builder by consuming another builder.
	pub fn extend(mut self, other_builder: RouterBuilder) -> Self {
		self.inner.extend(other_builder.inner);
		self.base_resources_inner.extend(other_builder.base_resources_inner);
		self
	}

	/// Builds the `RpcRouter` from this builder.
	/// This is the typical usage, with the `RpcRouter` being encapsulated in an `Arc`,
	/// indicating it is designed for cloning and sharing across tasks/threads.
	pub fn build(self) -> Router {
		Router::new(self.inner, self.base_resources_inner)
	}
}
