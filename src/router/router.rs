use crate::{CallResult, Request, RouterBuilder};
use crate::{Resources, RouterInner};
use serde_json::Value;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Router {
	inner: Arc<RouterInner>,
}

//-- Builder
impl Router {
	/// Returns a new `ResourcesBuilder`.
	/// This is equivalent to calling `Resources::default()`.
	pub fn builder() -> RouterBuilder {
		RouterBuilder::default()
	}
}

// -- Methods
impl Router {
	/// Performs the RPC call for a given RpcRequest object, which contains the `id`, method name, and parameters.
	///
	/// - Returns an CallResult that echoes the `id` and `method`, and includes the `Result<Value, rpc_router::Error>` result.
	///
	/// - The `rpc_router::Error` includes a variant `rpc_router::Error::Handler(RpcHandlerError)`,
	///   where `RpcHandlerError` allows retrieval of the application error returned by the handler
	///   through `RpcHandlerError::get::<T>(&self) -> Option<T>`.
	///   This mechanism enables application RPC handlers to return specific application errors while still utilizing
	///   the `rpc-router` result structure, thereby allowing them to retrieve their specific error type.
	pub async fn call(&self, resources: Resources, rpc_request: Request) -> CallResult {
		self.inner.call(resources, rpc_request).await
	}

	/// Performs the RPC call for a given RpcRequest object, which contains the `id`, method name, and parameters.
	///
	/// - method: The json-rpc method name.
	/// -     id: The json-rpc request `.id`, which should be sent by the client.
	///           It is required to echo it back in the json-rpc response.
	///           Can be `Value::Null`, and if None, it will be set to `Value::Null`
	/// - params: The optional json-rpc params
	///
	/// Returns an CallResult, where either the success value (CallResponse) or the error (CallError)
	/// will echo back the `id` and `method` part of their construct
	pub async fn call_route(
		&self,
		resources: Resources,
		id: Option<Value>,
		method: impl Into<String>,
		params: Option<Value>,
	) -> CallResult {
		self.inner.call_route(resources, id, method, params).await
	}

	/// For specific or advanced use cases.
	///
	/// Use `RpcRouterBuilder::default()...build()` if unsure.
	///
	/// Creates an `RpcRouter` from its inner data.
	///
	/// Note: This is intended for situations where a custom builder
	///       workflow is needed. The recommended method for creating an `RpcRouter`
	///       is via the `RpcRouterBuilder`.
	pub fn from_inner(inner: RouterInner) -> Self {
		Self { inner: Arc::new(inner) }
	}
}
