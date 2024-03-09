#![allow(clippy::module_inception)] // not publicly exposed

// region:    --- Modules

mod handler;
mod handler_error;
mod handler_wrapper;
mod impl_handlers;

// -- Flatten
pub use handler::*;
pub use handler_error::*;
pub use handler_wrapper::*;

use futures::Future;
use serde_json::Value;
use std::pin::Pin;

// endregion: --- Modules

type PinFutureValue = Pin<Box<dyn Future<Output = crate::Result<Value>> + Send>>;
