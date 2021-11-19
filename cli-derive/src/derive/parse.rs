use proc_macro2::TokenStream;
use quote::quote;

use crate::types::{FieldArgs, StructArgs};
use crate::utils::{fetch_subcommand, unwrap_ident};

pub fn gen_interactive(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let (clap_fields, nonclap) = gen_interactive_fields(args);

    quote! {
        impl near_cli_visual::types::Interactive for #struct_ident {
            fn interactive(clap: Option<&Self::Clap>, mut builder: Self::Builder) -> Self::Builder {
                if let Some(clap) = clap {
                    #(#clap_fields)*
                }
                else {
                    #(#nonclap)*
                }

                builder
            }
        }
    }
}

fn gen_interactive_fields(args: &StructArgs) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let struct_ident = &args.ident;
    args.fields()
        .into_iter()
        .map(|field| {
            let FieldArgs {
                ident: field_ident,
                prompt_msg,
                prompt_fn,
                ..
            } = field;

            if prompt_msg.is_none() && prompt_fn.is_none() {
                // Skip if not present
                return (quote!(), quote!());
            }

            let field_ident = unwrap_ident(field_ident);
            let mut prompter = None;
            if let Some(prompt_msg) = prompt_msg {
                prompter = Some(quote! { near_cli_visual::prompt_input_with_msg(#prompt_msg) });
            } else if let Some(prompt_fn) = prompt_fn {
                let prompt_fn = syn::Ident::new(&prompt_fn, struct_ident.span());
                prompter = Some(quote! { #prompt_fn () });
            }
            let interactive = prompter.expect(&format!(
                "Did not specify how to prompt {}::{} with either prompt_msg or prompt_fn",
                struct_ident, field_ident
            ));

            let builder_fn = syn::Ident::new(&format!("set_{}", field_ident), struct_ident.span());

            let clap_fields = quote! {
                builder = builder . #builder_fn (
                    match clap . #field_ident . as_ref() {
                        Some(value) => value.clone(),
                        None => #interactive,
                    }
                );
            };

            let nonclap = quote! {
                builder = builder . #builder_fn ( #interactive );
            };

            (clap_fields, nonclap)
        })
        .unzip()
}

pub fn gen_build(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let build_retry_loop = gen_build_retry_loop(args);
    let subcommand = gen_build_subcommand(args);
    let fields = gen_build_fields(args);

    quote! {
        impl near_cli_visual::types::Build for #struct_ident {
            type Err = ();

            fn build(clap: Option<Self::Clap>, mut builder: Self::Builder) -> Result<Self, Self::Err> {
                let scope = #build_retry_loop;

                Ok(Self {
                    #(#fields)*

                    #subcommand
                })
            }
        }

    }
}

// The loop where we call into Interactive/Validate
pub fn gen_build_retry_loop(args: &StructArgs) -> TokenStream {
    let StructArgs { enable, .. } = args;

    let mut validate = quote!( break Ok(builder); );
    if let Some(enable) = enable {
        if enable.validator {
            validate = quote! {
                count -= 1;
                let valid = <Self as near_cli_visual::types::Validate>::validate(clap.as_ref(), &builder);
                if valid.is_ok() {
                    break Ok(builder);
                }
                else if count == 0 {
                    break Err(());
                }
            };
        }
    }

    quote! {{
        use near_cli_visual::types::IntoScope;

        let mut count = near_cli_visual::consts::max_build_retry();
        let builder: Result<Self::Builder, ()> = loop {
            builder = <Self as near_cli_visual::types::Interactive>::interactive(clap.as_ref(), builder);
            #validate
        };

        let scope = builder?.into_scope()?;

        scope
    }}
}

pub fn gen_build_subcommand(args: &StructArgs) -> TokenStream {
    if let Some(FieldArgs { ident, ty, single, .. }) = fetch_subcommand(args) {
        let enum_sub_interactive = if *single {
            quote! {{ clap.unwrap_single_subcommand() }}
        } else {
            // let prompt_msg = prompt_msg.expect("prompt_msg required for choosing subcommand");
            // TODO: try to get enum version working too. the above doesn't do much. Need inner value, Probably need a #[derive(Eclap)] for enum
            // quote! {{ near_cli_visual::prompt_variant :: <  > () }}
            quote!()
        };

        let ident = unwrap_ident(ident);
        return quote! {
            #ident : {
                // Here, we're trying to get inner value of the enum
                let mut sub_clap = None;
                if let Some(clap) = clap {
                    if let Some(clap) = clap.subcommand {
                        sub_clap = Some(#enum_sub_interactive)
                    }
                }

                let sub_builder = <#ty as near_cli_visual::types::BuilderFrom<Self>>::builder_from(&scope);
                let subcommand = <#ty as near_cli_visual::types::Build>::build(sub_clap, sub_builder)?;

                subcommand
            },
        };
    }

    quote!()
}

fn gen_build_fields(args: &StructArgs) -> Vec<TokenStream> {
    args.fields()
        .into_iter()
        .map(|field| {
            let FieldArgs {
                ident: field_ident,
                subcommand,
                ..
            } = field;

            if *subcommand {
                return quote!();
            }

            let field_ident = unwrap_ident(field_ident);
            quote! {
                #field_ident : scope . #field_ident,
            }
        })
        .collect()
}
