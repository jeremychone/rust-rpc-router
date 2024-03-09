use crate::handler::RpcHandlerWrapperTrait;
use crate::{CallError, CallResponse, CallResult, Error, Handler, Request, Resources};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// method, which calls the appropriate handler matching the method_name.
///
/// RouterInner can be extended with other RouterInners for composability.
#[derive(Default)]
pub struct RouterInner {
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

	/// Add an handler function to the router.
	///
	/// ```
	/// RouterInner::new().add("method_name", my_handler_fn);
	/// ```
	///
	/// Note: This is a convenient add function variant with generics,
	///       and there will be monomorphed versions of this function
	///       for each type passed. Use `RouterInner::add_dyn` to avoid this.
	pub fn add<F, T, P, R>(&mut self, name: &'static str, handler: F)
	where
		F: Handler<T, P, R> + Clone + Send + Sync + 'static,
		T: Send + Sync + 'static,
		P: Send + Sync + 'static,
		R: Send + Sync + 'static,
	{
		self.append_dyn(name, handler.into_dyn());
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

		self.call_route(resources, Some(id), method, params).await
	}

	/// Performs the RPC call for a given Request object, which contains the `id`, method name, and parameters.
	///
	/// - method: The json-rpc method name.
	/// -     id: The json-rpc request `.id`, which should be sent by the client.
	///           It is required to echo it back in the json-rpc response.
	///           Can be `Value::Null`, and if None, it will be set to `Value::Null`
	/// - params: The optional json-rpc params
	///
	/// Returns an ResponseResult, where either the success value (Response) or the error (ResponseError)
	/// will echo back the `id` and `method` part of their construct
	pub async fn call_route(
		&self,
		resources: Resources,
		id: Option<Value>,
		method: impl Into<String>,
		params: Option<Value>,
	) -> CallResult {
		let method = method.into();
		let id = id.unwrap_or(Value::Null);

		if let Some(route) = self.route_by_name.get(method.as_str()) {
			match route.call(resources, params).await {
				Ok(value) => Ok(CallResponse { id, method, value }),
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
