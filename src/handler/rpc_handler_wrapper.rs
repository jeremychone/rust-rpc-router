use crate::Result;
use crate::{PinFutureValue, RpcHandler};
use futures::Future;
use serde_json::Value;
use std::marker::PhantomData;
use std::pin::Pin;

/// `RpcHandlerWrapper` is an `RpcHandler` wrapper that implements
/// `RpcHandlerWrapperTrait` for type erasure, enabling dynamic dispatch.
/// Generics:
/// - `H`: The handler trait for the function
/// - `K`: The Resources, meaning the type passed in the call that has the `FromResources` trait for the various `T` types (cannot use `R`, as it is reserved for the
/// - `T`: The type (can be a tuple when multiple) for the function parameters
/// - `P`: The JSON RPC parameter
/// - `R`: The response type
///
/// Thus, all these types except `H` will match the generic of the `H` handler trait. We keep them in phantom data.
#[derive(Clone)]
pub struct RpcHandlerWrapper<H, K, T, P, R> {
	handler: H,
	_marker: PhantomData<(K, T, P, R)>,
}

// Constructor
impl<H, K, T, P, R> RpcHandlerWrapper<H, K, T, P, R> {
	pub fn new(handler: H) -> Self {
		Self {
			handler,
			_marker: PhantomData,
		}
	}
}

// Call Impl
impl<H, K, T, P, R> RpcHandlerWrapper<H, K, T, P, R>
where
	H: RpcHandler<K, T, P, R> + Send + Sync + 'static,
	K: Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	pub fn call(&self, rpc_resources: K, params: Option<Value>) -> H::Future {
		// Note: Since handler is a FnOnce, we can use it only once, so we clone it.
		//       This is likely optimized by the compiler.
		let handler = self.handler.clone();
		RpcHandler::call(handler, rpc_resources, params)
	}
}

/// `RpcHandlerWrapperTrait` enables `RpcHandlerWrapper` to become a trait object,
/// allowing for dynamic dispatch.
pub trait RpcHandlerWrapperTrait<K>: Send + Sync
where
	K: Send + Sync + 'static,
{
	fn call(&self, rpc_resources: K, params: Option<Value>) -> PinFutureValue;
}

impl<H, K, T, P, R> RpcHandlerWrapperTrait<K> for RpcHandlerWrapper<H, K, T, P, R>
where
	H: RpcHandler<K, T, P, R> + Clone + Send + Sync + 'static,
	K: Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	fn call(&self, rpc_resources: K, params: Option<Value>) -> Pin<Box<dyn Future<Output = Result<Value>> + Send>> {
		Box::pin(self.call(rpc_resources, params))
	}
}
