//! rpc::router module provides the type and implementation for
//! json rpc routing.
//!
//! It has the following constructs:
//!
//! - `Router` holds the HashMap of `method_name: Box<dyn RpcHandlerWrapperTrait>`.
//! - `RpcHandler` trait is implemented for any async function that, with
//!   `(S1, S2, ...[impl IntoParams])`, returns `web::Result<Serialize>` where S1, S2, ... are
//!   types that implement `FromResources` (see router/from_resources.rs and src/resources.rs).
//! - `IntoParams` is the trait to implement to instruct how to go from `Option<Value>` json-rpc params
//!   to the handler's param types.
//! - `IntoParams` has a default `into_params` implementation that will return an error if the params are missing.
//!
//! ```ignore // Example needs update for new types/macros
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

mod support;

mod error;
mod handler;
mod params;
mod resource;
mod router;
mod rpc_id;
mod rpc_request; // Added rpc_id module

// -- Flatten
pub use self::error::{Error, Result};
pub use handler::{Handler, HandlerError, HandlerResult, IntoHandlerError, RpcHandlerWrapperTrait};
pub use params::*;
pub use resource::*;
pub use router::*;
pub use rpc_id::*;
pub use rpc_request::*; // Export RpcId

// -- Export proc macros
pub use rpc_router_macros::RpcHandlerError;
pub use rpc_router_macros::RpcParams;
pub use rpc_router_macros::RpcResource;

// endregion: --- Modules
