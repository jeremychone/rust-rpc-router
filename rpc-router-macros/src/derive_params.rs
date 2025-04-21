use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, GenericParam, parse_macro_input};

pub fn derive_rpc_params_inner(input: TokenStream) -> TokenStream {
	let input = parse_macro_input!(input as DeriveInput);
	let name = &input.ident;
	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	let expanded = if input.generics.params.is_empty() {
		// Non-generic struct
		quote! {
			impl rpc_router::IntoParams for #name {}
		}
	} else {
		// Generic struct
		// Extend where_clause with DeserializeOwned + Send for each type parameter
		let where_clause = where_clause.map_or_else(
			|| {
				let constraints = input.generics.params.iter().filter_map(|p| {
					if let GenericParam::Type(type_param) = p {
						Some(quote! { #type_param: serde::de::DeserializeOwned + Send })
					} else {
						None
					}
				});
				quote! { where #(#constraints,)* }
			},
			|where_clause| quote! { #where_clause },
		);

		quote! {
			impl #impl_generics rpc_router::IntoParams for #name #ty_generics #where_clause {}
		}
	};
	// Convert back to a token stream and return it
	TokenStream::from(expanded)
}
