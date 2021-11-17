use crate::types::{StructArgs, FieldArgs};
use crate::utils::ident_postfix;

use proc_macro2::TokenStream;
use quote::{quote};


pub fn gen(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let builder_ident = ident_postfix(struct_ident, "Builder");
    let ((funcs, fields), scope_fields) = gen_builder_internals(args);

    quote! {
        #[derive(Default)]
        struct #builder_ident {
            #(#fields)*
        }

        impl #builder_ident {
            #(#funcs)*
        }

        impl near_cli_visual::types::Builder for #struct_ident {
            type Builder = #builder_ident;
        }

        impl near_cli_visual::types::IntoScope for #builder_ident {
            type Err = ();
            type Scope = <#struct_ident as near_cli_visual::types::Scoped>:: Scope;

            fn into_scope(self) -> Result<Self::Scope, Self::Err> {
                Ok(Self::Scope {
                    #(#scope_fields)*
                })
            }
        }
    }
}

fn gen_builder_internals(args: &StructArgs) -> ((Vec<TokenStream>, Vec<TokenStream>), Vec<TokenStream>) {
    let StructArgs {
        ident: struct_ident,
        generics: _,
        data: _,
        ..
    } = args;

    args.fields().into_iter().map(|f| {
        let FieldArgs {
            ident,
            ty,
            subcommand,
            ..
        } = f;

        if *subcommand {
            // Subcommand are not apart of the Builder. So exclude it with empty field.
            return ((quote!(), quote!()), quote!());
        }

        // will fail if enum, newtype or tuple
        let ident = ident.as_ref().expect("only supported for regular structs");

        // Builder functions. This allows us to write `set_#field` into the builder.
        let builder_fn = syn::Ident::new(&format!("set_{}", ident), struct_ident.span());
        let builder_fn = quote! {
            fn #builder_fn (mut self, val: #ty) -> Self {
                self.#ident = Some(val);
                self
            }
        };

        let builder_field = quote! {
            #ident: Option<#ty>,
        };

        let scope_field = quote! {
            #ident: self.#ident.ok_or_else(|| ())?,
        };

        ((builder_fn, builder_field), scope_field)
    })
    .unzip()
}