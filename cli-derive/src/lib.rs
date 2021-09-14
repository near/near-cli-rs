#![allow(dead_code)]
#![allow(unused)]


use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{self, Ident, ItemEnum, ItemStruct, ItemUnion};

mod internal;


#[proc_macro_derive(Interactive)]
pub fn derive_interactive(input: TokenStream) -> TokenStream {
    let cratename = Ident::new("near_cli_visual",Span::call_site());

    let outcome = if let Ok(input) = syn::parse::<ItemStruct>(input.clone()) {
        crate::internal::struct_impl(&input, cratename)
    } else if let Ok(input) = syn::parse::<ItemEnum>(input.clone()) {
        crate::internal::enum_impl(&input, cratename)
    // } else if let Ok(input) = syn::parse::<ItemUnion>(input.clone()) {
    //     union_ser(&input)
    } else {
        // Derive macros can only be defined on structs, enums, and unions.
        unreachable!()
    };

    TokenStream::from(match outcome {
        Ok(res) => res,
        // Ok(res) => panic!(res.to_string()),
        Err(err) => err.to_compile_error(),
    })
}
