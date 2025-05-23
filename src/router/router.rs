use crate::router::router_inner::RouterInner;
use crate::{CallResult, ResourcesInner, RouterBuilder, RpcRequest};
use crate::{FromResources, Resources, RpcId};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Router {
	inner: Arc<RouterInner>,
	base_resources: Resources,
}

//-- Builder
impl Router {
	/// Returns a new `RouterBuilder`.
	/// This is equivalent to calling `Router::builder()`.
	pub fn builder() -> RouterBuilder {
		RouterBuilder::default()
	}
}
impl FromResources for Router {}

// -- Methods
impl Router {
	/// Performs the RPC call for a given RpcRequest object (i.e., `.id, .method, .params`)
	/// with the eventual resources of the router.
	///
	/// To add additional resources on top of the router's resources, call `.call_with_resources(request, resources)`
	///
	/// - Returns an CallResult that echoes the `id` and `method`, and includes the `Result<Value, rpc_router::Error>` result.
	///
	/// - The `rpc_router::Error` includes a variant `rpc_router::Error::Handler(HandlerError)`,
	///   where `HandlerError` allows retrieval of the application error returned by the handler
	///   through `HandlerError::get::<T>(&self) -> Option<T>`.
	///   This mechanism enables application RPC handlers to return specific application errors while still utilizing
	///   the `rpc-router` result structure, thereby allowing them to retrieve their specific error type.
	///
	pub async fn call(&self, rpc_request: RpcRequest) -> CallResult {
		self.inner.call(self.base_resources.clone(), rpc_request).await
	}

	/// Similar to `.call(...)`, but takes an additional `Resources` parameter that will be overlaid on top
	/// of the eventual base router resources.
	///
	/// Note: The router will first try to get the resource from the overlay, and then,
	///       will try the base router resources.
	pub async fn call_with_resources(&self, rpc_request: RpcRequest, additional_resources: Resources) -> CallResult {
		let resources = self.compute_call_resources(additional_resources);

		self.inner.call(resources, rpc_request).await
	}

	/// Lower level function to `.call` which take all Rpc Request properties as value.
	/// If id is None, it will be set a Value::Null
	///
	/// This also use router base resources.
	///
	/// To add additional resources on top of the router's resources, call `.call_route_with_resources(request, resources)`
	///
	/// - method: The json-rpc method name.
	/// -     id: The json-rpc request ID. If None, defaults to RpcId::Null.
	/// - params: The optional json-rpc params
	///
	/// Returns an CallResult, where either the success value (CallSuccess) or the error (CallError)
	/// will echo back the `id` and `method` part of their construct
	pub async fn call_route(&self, id: Option<RpcId>, method: impl Into<String>, params: Option<Value>) -> CallResult {
		let id = id.unwrap_or_default(); // Default to RpcId::Null if None
		self.inner.call_route(self.base_resources.clone(), id, method, params).await
	}

	/// Similar to `.call_route`, but takes an additional `Resources` parameter that will be overlaid on top
	/// of the eventual base router resources.
	///
	/// Note: The router will first try to get the resource from the overlay, and then,
	///       will try the base router resources.
	pub async fn call_route_with_resources(
		&self,
		id: Option<RpcId>,
		method: impl Into<String>,
		params: Option<Value>,
		additional_resources: Resources,
	) -> CallResult {
		let resources = self.compute_call_resources(additional_resources);
		let id = id.unwrap_or_default(); // Default to RpcId::Null if None

		self.inner.call_route(resources, id, method, params).await
	}
}

// Crate only method
impl Router {
	/// For specific or advanced use cases.
	///
	/// Use `RouterBuilder::default()...build()` if unsure.
	///
	/// Creates an `Router` from its inner data.
	///
	/// Note: This is intended for situations where a custom builder
	///       workflow is needed. The recommended method for creating an `Router`
	///       is via the `RouterBuilder`.
	pub(crate) fn new(inner: RouterInner, resources_inner: ResourcesInner) -> Self {
		Self {
			inner: Arc::new(inner),
			base_resources: Resources::from_base_inner(resources_inner),
		}
	}

	pub(crate) fn compute_call_resources(&self, call_resources: Resources) -> Resources {
		if self.base_resources.is_empty() {
			call_resources
		} else {
			self.base_resources.new_with_overlay(call_resources)
		}
	}
}
