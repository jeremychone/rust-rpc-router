use crate::handler::RpcHandlerWrapperTrait;
use crate::{CallError, CallResponse, CallResult, Error, Request, Resources, RpcId};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// method, which calls the appropriate handler matching the method_name.
///
/// RouterInner can be extended with other RouterInners for composability.
#[derive(Default)]
pub(crate) struct RouterInner {
	route_by_name: HashMap<&'static str, Box<dyn RpcHandlerWrapperTrait>>,
}

impl fmt::Debug for RouterInner {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("RouterInner")
			.field("route_by_name", &self.route_by_name.keys())
			.finish()
	}
}

impl RouterInner {
	/// Add a dyn_handler to the router.
	///
	/// ```
	/// RouterInner::new().add_dyn("method_name", my_handler_fn.into_dyn());
	/// ```
	///
	/// Note: This is the preferred way to add handlers to the router, as it
	///       avoids monomorphization of the add function.
	///       The RouterInner also has a `.add()` as a convenience function to just pass the function.
	///       See `RouterInner::add` for more details.
	pub fn append_dyn(&mut self, name: &'static str, dyn_handler: Box<dyn RpcHandlerWrapperTrait>) {
		self.route_by_name.insert(name, dyn_handler);
	}

	pub fn extend(&mut self, other_router: RouterInner) {
		self.route_by_name.extend(other_router.route_by_name);
	}

	/// Performs the RPC call for a given Request object, which contains the `id`, method name, and parameters.
	///
	/// Returns an ResponseResult, where either the success value (Response) or the error (ResponseError)
	/// will echo back the `id` and `method` part of their construct
	pub async fn call(&self, resources: Resources, rpc_request: Request) -> CallResult {
		let Request { id, method, params } = rpc_request;

		self.call_route(resources, id, method, params).await
	}

	/// Performs the RPC call given the id, method, and params.
	///
	/// - method: The json-rpc method name.
	/// -     id: The json-rpc request ID. If None, defaults to RpcId::Null.
	/// - params: The optional json-rpc params.
	///
	/// Returns a CallResult, where either the success value (CallResponse) or the error (CallError)
	/// will include the original `id` and `method`.
	pub async fn call_route(
		&self,
		resources: Resources,
		id: RpcId,
		method: impl Into<String>,
		params: Option<Value>,
	) -> CallResult {
		let method = method.into();

		if let Some(route) = self.route_by_name.get(method.as_str()) {
			match route.call(resources, params).await {
				Ok(value) => Ok(CallResponse {
					id: id.clone(), // Clone id for the response
					method: method.clone(),
					value,
				}),
				Err(error) => Err(CallError { id, method, error }),
			}
		} else {
			Err(CallError {
				id,
				method,
				error: Error::MethodUnknown,
			})
		}
	}
}

