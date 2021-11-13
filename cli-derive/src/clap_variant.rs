use crate::types::{StructArgs, FieldArgs};
use proc_macro2::TokenStream;
use quote::{quote};
use syn::{Ident, Type};


pub fn gen(args: &StructArgs) -> TokenStream {
    let StructArgs {
        ident: struct_ident,
        generics: _,
        data: _,
    } = args;
    let mut passthru = None;
    let fields = args.fields().into_iter().map(|f| {
        let FieldArgs {
            ident,
            ty,
            single,
            subcommand,
            ..
        } = f;

        // TODO: potential do not generate clap variant option if we skip it.
        // let field_ty = if f.skip {
        //     quote!(#field_ty)
        // } else {
        //     quote! {
        //         Option<#field_ty>
        //     }
        // };

        let mut ty = quote!(#ty);
        let mut qualifiers = quote! {};
        if *subcommand {
            // qualifiers = quote! { #[clap(subcommand)] };
            qualifiers = quote! {};
            if *single {
                let name = format!("{}ClapVariantPassThru", struct_ident);
                let name = syn::Ident::new(&name, struct_ident.span());
                passthru = Some(gen_clap_enum_pass(struct_ident, &ty));
                ty = quote!(#name);
            }
        }

        let ty = quote! { Option<#ty> };
        let field = if let Some(ident) = ident {
            quote! { #ident: #ty, }
        } else {
            ty
        };

        quote! {
            #qualifiers
            #field
        }
    })
    .collect::<Vec<_>>();

    let name = format!("{}ClapVariant", struct_ident);
    let name = syn::Ident::new(&name, struct_ident.span());
    let passthru = passthru.unwrap_or_else(|| quote! {});

    quote! {
        // #[derive(Parser)]
        struct #name {
            #(#fields)*
        }

        #passthru
    }
}

fn gen_clap_enum_pass(ident: &Ident, ty: &TokenStream) -> TokenStream {
    let name = format!("{}ClapVariantPassThru", ident);
    let name = syn::Ident::new(&name, ident.span());
    quote! {
        // #[derive(Parser)]
        enum #name {
            PassThru(#ty)
        }
    }
}
