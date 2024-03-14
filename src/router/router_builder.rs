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

	/// Extends this builder by consuming another builder.
	pub fn extend(mut self, other_builder: RouterBuilder) -> Self {
		self.inner.extend(other_builder.inner);
		self.base_resources_inner.extend(other_builder.base_resources_inner);
		self
	}

	pub fn append_resource<T>(mut self, val: T) -> Self
	where
		T: FromResources + Clone + Send + Sync + 'static,
	{
		self.base_resources_inner.insert(val);
		self
	}

	/// If Some, will extends the current Base Resources with the content of this resources_builder.
	/// If None, just do nothing.
	pub fn extend_resources(mut self, resources_builder: Option<ResourcesBuilder>) -> Self {
		if let Some(resources_builder) = resources_builder {
			// if self resources empty, no need to extend
			if self.base_resources_inner.is_empty() {
				self.base_resources_inner = resources_builder.resources_inner
			}
			// if not empty, we extend
			else {
				self.base_resources_inner.extend(resources_builder.resources_inner)
			}
		}
		self
	}

	/// Resets the router's resources with the contents of this ResourcesBuilder.
	///
	/// Ensure to call `append_resource` and/or `.extend_resources` afterwards
	/// if they operation needs to be included.
	///
	/// Note: `.extend_resources(Option<ResourcesBuilder>)` is the additive function
	///        typically used.
	pub fn set_resources(mut self, resources_builder: ResourcesBuilder) -> Self {
		self.base_resources_inner = resources_builder.resources_inner;
		self
	}

	/// Builds the `RpcRouter` from this builder.
	/// This is the typical usage, with the `RpcRouter` being encapsulated in an `Arc`,
	/// indicating it is designed for cloning and sharing across tasks/threads.
	pub fn build(self) -> Router {
		Router::new(self.inner, self.base_resources_inner)
	}
}
