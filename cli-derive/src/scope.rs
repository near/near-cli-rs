use crate::types::{StructArgs, FieldArgs};
use proc_macro2::TokenStream;
use quote::{quote};


pub fn gen(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let scope_ident = format!("{}Scope", struct_ident);
    let scope_ident = syn::Ident::new(&scope_ident, struct_ident.span());
    let fields = gen_scope_internals(args);

    quote! {
        #[derive(Clone)]
        struct #scope_ident {
            #(#fields)*
        }

        impl near_cli_visual::types::Scoped for #struct_ident {
            type Scope = #scope_ident;
        }
    }
}

fn gen_scope_internals(args: &StructArgs) -> Vec<TokenStream> {
    let StructArgs {
        ident: struct_ident,
        generics: _,
        data: _,
    } = args;

    args.fields().into_iter().map(|f| {
        let FieldArgs {
            ident,
            ty,
            subcommand,
            ..
        } = f;

        if *subcommand {
            // Subcommand are not apart of the Scope. So exclude it with empty field.
            return quote!();
        }

        // will fail if enum, newtype or tuple
        let ident = ident.as_ref().expect("only supported for regular structs");

        quote! {
            pub #ident: #ty,
        }
    })
    .collect()
}
