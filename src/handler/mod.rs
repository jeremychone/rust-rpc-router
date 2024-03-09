// region:    --- Modules

mod impl_rpc_handlers;
mod rpc_handler;
mod rpc_handler_error;
mod rpc_handler_wrapper;

// -- Flatten
pub use rpc_handler::*;
pub use rpc_handler_error::*;
pub use rpc_handler_wrapper::*;

use futures::Future;
use serde_json::Value;
use std::pin::Pin;

// endregion: --- Modules

type PinFutureValue = Pin<Box<dyn Future<Output = crate::Result<Value>> + Send>>;
