use crate::Result;
use crate::RpcHandlerWrapper;
use crate::RpcHandlerWrapperTrait;
use futures::Future;
use serde_json::Value;

/// The `Handler` trait that will be implemented by rpc handler functions.
///
/// Key points:
/// - Rpc handler functions are asynchronous, thus returning a Future of Result<Value>.
/// - The call format is normalized to two `impl FromResources` arguments (for now) and one optionals  `impl IntoParams`, which represent the json-rpc's optional value.
/// - `into_box` is a convenient method for converting a RpcHandler into a Boxed dyn RpcHandlerWrapperTrait,
///   allowing for dynamic dispatch by the Router.
/// - A `RpcHandler` will typically be implemented for static functions, as `FnOnce`,
///   enabling them to be cloned with none or negligible performance impact,
///   thus facilitating the use of RpcRoute dynamic dispatch.
/// - `T` is the tuple of `impl FromResources` arguments.
/// - `P` is the `impl IntoParams` argument.
///
pub trait RpcHandler<K, T, P, R>: Clone
where
	K: Send + Sync + 'static,
	T: Send + Sync + 'static,
	P: Send + Sync + 'static,
	R: Send + Sync + 'static,
{
	/// The type of future calling this handler returns.
	type Future: Future<Output = Result<Value>> + Send + 'static;

	/// Call the handler.
	fn call(self, rpc_resources: K, params: Option<Value>) -> Self::Future;

	/// Convert this RpcHandler into a Boxed dyn RpcHandlerWrapperTrait,
	/// for dynamic dispatch by the Router.
	fn into_dyn(self) -> Box<dyn RpcHandlerWrapperTrait<K>>
	where
		Self: Sized + Send + Sync + 'static,
	{
		Box::new(RpcHandlerWrapper::new(self)) as Box<dyn RpcHandlerWrapperTrait<K>>
	}
}
