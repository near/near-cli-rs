use crate::types::{FieldArgs, StructArgs};
use crate::utils::{unwrap_ident, ident_postfix};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

pub fn gen(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let name = ident_postfix(struct_ident, "ClapVariant");
    let (passthru, fields) = gen_clap_internals(args);

    quote! {
        #[derive(clap::Parser)]
        struct #name {
            #(#fields)*
        }

        #passthru

        impl near_cli_visual::types::ClapVariant for #struct_ident {
            type Clap = #name;
        }
    }
}

fn gen_clap_internals(args: &StructArgs) -> (TokenStream, Vec<TokenStream>) {
    let StructArgs {
        ident: struct_ident,
        generics: _,
        data: _,
        ..
    } = args;

    // let mut passthru = None;
    let mut sub_args = SubcommandArgs {
        ident: struct_ident.clone(),

        // by default, if no one specifies `single`, then there's no passthru
        // code to generate for this clap variant.
        passthru: quote!(),
    };

    let fields = args
        .fields()
        .into_iter()
        .map(|f| {
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

            let ident = unwrap_ident(ident);
            let mut ident = quote!(#ident);
            let mut field_ty = quote!(#ty);
            let mut qualifiers = quote! {};
            if *subcommand {
                // this is a subcommand. ClapVariant will call it `subcommand` instead
                ident = quote!(subcommand);
                qualifiers = quote! { #[clap(subcommand)] };

                // Single enum wrapper. Generate it and replace the type with it.
                if *single {
                    let (ident, code) = gen_clap_enum_pass(struct_ident, &ty);
                    field_ty = quote!(#ident);
                    sub_args.ident = ident;
                    sub_args.passthru = code;
                }
            }

            quote! {
                #qualifiers
                #ident: Option<#field_ty>,
            }
        })
        .collect();

    (sub_args.passthru, fields)
}

struct SubcommandArgs {
    ident: Ident,
    passthru: TokenStream,
}

fn gen_clap_enum_pass(struct_ident: &Ident, ty: &Type) -> (Ident, TokenStream) {
    let clap_ty = Ident::new(&format!("{}ClapVariant", quote!(#ty)), struct_ident.span());
    let passthru_ident = ident_postfix(struct_ident, "ClapVariantPassThru");
    let code = quote! {
        #[derive(clap::Parser)]
        enum #passthru_ident {
            #ty ( #clap_ty )
        }

        impl #passthru_ident {
            fn unwrap_single_subcommand(self) -> #clap_ty {
                match self {
                    #passthru_ident :: #ty (x) => x,
                    _ => panic!("Expected single subcommand"),
                }
            }
        }
    };

    (passthru_ident, code)
}
