use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use syn::{self, Ident, ItemEnum, ItemStruct};
use types::StructArgs;

use darling::FromDeriveInput;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod derive;
mod internal;
mod types;
mod utils;

use crate::derive::{clap_variant, builder, scope, parse};

#[proc_macro_derive(Interactive, attributes(interactive_skip))]
pub fn derive_interactive(input: TokenStream) -> TokenStream {
    let cratename = Ident::new("near_cli_visual", Span::call_site());

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

#[proc_macro_derive(Eclap, attributes(eclap))]
pub fn derive_eclap(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let args = StructArgs::from_derive_input(&input).unwrap();

    let clap_variant = clap_variant::gen(&args);
    let builder = builder::gen(&args);
    let scope = scope::gen(&args);

    let interactive = parse::gen_interactive(&args);
    let build = parse::gen_build(&args);

    // TODO: potentially add a module
    // let modname = format!("__eclap_gen_{}", args.ident);
    // let modname = proc_macro2::Ident::new(&modname, Span::call_site());
    let stream = quote! {
        #clap_variant
        #builder
        #scope

        #interactive
        #build
    };
    stream.into()
}
