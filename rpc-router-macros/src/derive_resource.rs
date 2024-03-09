use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive_rpc_resource_inner(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = parse_macro_input!(input as DeriveInput);

	// Build the impl
	let name = input.ident; // Gets the identifier of the enum/struct
	let expanded = quote! {
		// Generate the trait implementation
		impl rpc_router::FromRpcResources for #name {}
	};

	// Convert back to a token stream and return it
	TokenStream::from(expanded)
}
