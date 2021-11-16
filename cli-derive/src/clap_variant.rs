use crate::utils::ident_postfix;
use crate::types::{StructArgs, FieldArgs};

use proc_macro2::TokenStream;
use quote::{quote};
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

fn gen_clap_internals(args : &StructArgs) -> (TokenStream, Vec<TokenStream>) {
    let StructArgs {
        ident: struct_ident,
        generics: _,
        data: _,
    } = args;

    // let mut passthru = None;
    let mut sub_args = SubcommandArgs {
        ident: struct_ident.clone(),

        // by default, if no one specifies `single`, then there's no passthru
        // code to generate for this clap variant.
        passthru: quote!(),
    };

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

        let ident = ident.as_ref().expect("Enums/tuples/newtypes not supported");
        let mut ident = quote!(#ident);
        let mut ty = quote!(#ty);
        let mut qualifiers = quote! {};
        if *subcommand {
            // this is a subcommand. ClapVariant will call it `subcommand` instead
            ident = quote!(subcommand);
            qualifiers = quote! { #[clap(subcommand)] };

            // Single enum wrapper. Generate it and replace the type with it.
            if *single {
                let (ident, code) = gen_clap_enum_pass(struct_ident, &ty);
                ty = quote!(#ident);
                sub_args.ident = ident;
                sub_args.passthru = code;
            }
        }

        quote! {
            #qualifiers
            #ident: Option<#ty>,
        }
    })
    .collect();

    (sub_args.passthru, fields)
}

struct SubcommandArgs {
    ident: Ident,
    passthru: TokenStream,
}

fn gen_clap_enum_pass(struct_ident: &Ident, ty: &TokenStream) -> (Ident, TokenStream) {
    let passthru_ident = ident_postfix(struct_ident, "ClapVariantPassThru");
    let code = quote! {
        #[derive(clap::Parser)]
        enum #passthru_ident {
            PassThru(#ty)
        }

        impl #passthru_ident {
            fn unwrap_single_subcommand(self) -> #ty {
                match self {
                    #passthru_ident::PassThru(x) => x,
                    _ => panic!("Expected single subcommand"),
                }
            }
        }

        // fn build<T: near_cli_visual::types::Scoped>(
        //     clap: &Option<<#ty as near_cli_visual::types::ClapVariant>::Clap>,
        //     scope: &T::Scope,
        // ) -> Result<#ty, ()> {
        //     let sub_builder = <#ty as near_cli_visual::types::BuilderFrom<T>>::builder_from(&scope);
        //     let subcommand = <#ty as near_cli_visual::types::Build>::build(clap, sub_builder);

        //     subcommand
        // }
    };

    (passthru_ident, code)
}
