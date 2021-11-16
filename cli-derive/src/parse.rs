use proc_macro2::TokenStream;
use quote::quote;

use crate::types::{StructArgs, FieldArgs};


pub fn gen_interactive(args: &StructArgs) -> TokenStream {
    let struct_ident = &args.ident;
    let fields = gen_interactive_fields(args);

    quote! {
        impl near_cli_visual::types::Interactive for #struct_ident {
            fn interactive(clap: Option<&Self::Clap>, mut builder: Self::Builder) -> Self::Builder {
                if let Some(clap) = clap {
                    #(#fields)*
                }

                builder
            }
        }
    }
}

fn gen_interactive_fields(args: &StructArgs) -> Vec<TokenStream> {
    let struct_ident = &args.ident;
    args.fields().into_iter().map(|field| {
        let FieldArgs {
            ident: field_ident,
            ty,
            prompt_msg,
            prompt_fn,
            ..
        } = field;

        if prompt_msg.is_none() && prompt_fn.is_none() {
            // Skip if not present
            return quote!();
        }

        let field_ident = field_ident.as_ref().expect("Enum/tuples/newtypes are unsupported");
        let mut prompter = None;
        if let Some(prompt_msg) = prompt_msg {
            prompter = Some(quote! { near_cli_visual::prompt_input_with_msg(#prompt_msg) });
        }
        else if let Some(prompt_fn) = prompt_fn {
            let prompt_fn = syn::Ident::new(&prompt_fn, struct_ident.span());
            prompter = Some(quote! { #prompt_fn () });
        }
        let interactive = prompter.expect(
            &format!("Did not specify how to prompt {}::{} with either prompt_msg or prompt_fn",
                struct_ident, field_ident));

        let builder_fn = syn::Ident::new(&format!("set_{}", field_ident), struct_ident.span());

        // quote! {
        //     let value = clap . #field_ident . as_ref().unwrap_or_else(|| {
        //         #interactive
        //     });
        //     let builder = builder . #builder_fn (value)

        // }

        quote! {
            builder = builder . #builder_fn (
                match clap . #field_ident . as_ref() {
                    Some(value) => value.clone(),
                    None => #interactive,
                }
            );
        }
    })
    .collect()
}

