use crate::types::{StructArgs, FieldArgs};
use proc_macro2::TokenStream;
use quote::{quote};


pub fn gen(args: &StructArgs) -> TokenStream {
    let StructArgs {
        ident,
        generics: _,
        data: _,
    } = args;

    let fields = args.fields().into_iter().map(|f| {
        let FieldArgs {
            ident,
            ty,
            subcommand,
            ..
        } = f;

        // will fail if enum, newtype or tuple
        let ident = ident.as_ref().expect("only supported for regular structs");
        if *subcommand {
            // Subcommand are not apart of the Scope. So exclude it with empty field.
            quote! {}
        } else {
            quote! {
                #ident: #ty,
            }
        }
    });

    let name = format!("{}Scope", ident);
    let name = syn::Ident::new(&name, ident.span());
    quote! {
        struct #name {
            #(#fields)*
        }
    }
}