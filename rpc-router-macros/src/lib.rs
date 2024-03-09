// region:    --- Modules

mod derive_handler_error;
mod derive_params;
mod derive_resource;

use proc_macro::TokenStream;

use crate::derive_handler_error::drive_rpc_handler_error_inner;
use crate::derive_params::derive_rpc_params_inner;
use crate::derive_resource::derive_rpc_resource_inner;

// endregion: --- Modules

/// Will implement `IntoHandlerError` for this target type.
/// The target type must implement `std::error::Error`
#[proc_macro_derive(RpcHandlerError)]
pub fn derive_rpc_handler_error(input: TokenStream) -> TokenStream {
	drive_rpc_handler_error_inner(input)
}

/// Will implement `IntoParams` for this target type.
/// The target type must implement `Deserialize`
#[proc_macro_derive(RpcParams)]
pub fn derive_rpc_params(input: TokenStream) -> TokenStream {
	derive_rpc_params_inner(input)
}

/// Will implement `FromResources` for this target type.
/// The target type must implement `Clone + Send + Sync`
#[proc_macro_derive(RpcResource)]
pub fn derive_rpc_resource(input: TokenStream) -> TokenStream {
	derive_rpc_resource_inner(input)
}
