use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

pub fn drive_rpc_handler_error_inner(input: TokenStream) -> TokenStream {
	// Parse the input tokens into a syntax tree
	let input = parse_macro_input!(input as DeriveInput);

	// Build the impl
	let name = input.ident; // Gets the identifier of the enum/struct
	let expanded = quote! {
		// Generate the trait implementation
		impl rpc_router::IntoHandlerError for #name {}
	};

	// Convert back to a token stream and return it
	TokenStream::from(expanded)
}
