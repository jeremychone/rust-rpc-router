//! rpc::router module provides the type and implementation for
//! json rpc routing.
//!
//! It has the following constructs:
//!
//! - `RpcRouter` holds the HashMap of `method_name: Box<dyn RpcHandlerWrapperTrait>`.
//! - `RpcHandler` trait is implemented for any async function that, with
//!   `(S1, S2, ...[impl IntoParams])`, returns `web::Result<Serialize>` where S1, S2, ... are
//!    types that implement `FromResources` (see router/from_resources.rs and src/resources.rs).
//! - `IntoParams` is the trait to implement to instruct how to go from `Option<Value>` json-rpc params
//!   to the handler's param types.
//! - `IntoParams` has a default `into_params` implementation that will return an error if the params are missing.
//!
//! ```
//! #[derive(Deserialize)]
//! pub struct ParamsIded {
//!   id: i64,
//! }
//!
//! impl IntoParams for ParamsIded {}
//! ```
//!
//! - For custom `IntoParams` behavior, implement the `IntoParams::into_params` function.
//! - Implementing `IntoDefaultParams` on a type that implements `Default` will auto-implement `IntoParams`
//!   and call `T::default()` when the params `Option<Value>` is None.
//!

// region:    --- Modules

mod error;
mod handler;
mod into_rpc_params;
mod resource;
mod rpc_request;
mod rpc_router;

// -- Flatten
pub use self::error::{Error, Result};
pub use handler::RpcHandler;
pub use into_rpc_params::{IntoDefaultRpcParams, IntoRpcParams};
pub use resource::*;
pub use rpc_request::*;
pub use rpc_router::*;

use futures::Future;
use serde_json::Value;
use std::pin::Pin;

// endregion: --- Modules

pub type PinFutureValue = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
