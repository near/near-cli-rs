use core::convert::TryFrom;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{Fields, Ident, ItemEnum, ItemStruct, WhereClause};


pub fn struct_impl(input: &ItemStruct, cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );
    let mut body = TokenStream2::new();
    match &input.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let delta = quote! {
                    #field_name: near_cli_visual::Interactive::interactive(self.#field_name),
                };
                body.extend(delta);
            }
        }
        _ => {}
    }
    Ok(quote! {
        impl #impl_generics near_cli_visual::Interactive<Self> for #name #ty_generics #where_clause {
            fn interactive(self) -> Self {
                Self { #body }
            }
        }
    })
}


pub fn enum_impl(input: &ItemEnum, cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let mut where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );

    let mut variant_arms = TokenStream2::new();
    for (variant_idx, variant) in input.variants.iter().enumerate() {
        let variant_idx = u8::try_from(variant_idx).expect("up to 256 enum variants are supported");
        let variant_ident = &variant.ident;

        let mut variant_header = TokenStream2::new();
        let mut variant_body = TokenStream2::new();

        match &variant.fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    variant_header.extend(quote! { #field_name, });
                    variant_body.extend(quote! {
                        near_cli_visual::Interactive::interactive(self.#field_name)
                    })
                }
                variant_header = quote! { { #variant_header }};
            }
            Fields::Unnamed(fields) => {
                for (field_idx, field) in fields.unnamed.iter().enumerate() {
                    let field_ident = Ident::new(format!("id{}", field_idx).as_str(), Span::call_site());
                    variant_header.extend(quote! { #field_ident, });
                    variant_body.extend(quote! {
                        near_cli_visual::Interactive::interactive(#field_ident);
                    })
                }
                variant_header = quote! { ( #variant_header )};
            }
            Fields::Unit => {}
        }
        variant_arms.extend(quote! {
            #name::#variant_ident #variant_header => {
                #variant_body
            },
        });
    }

    Ok(quote! {
        impl #impl_generics near_cli_visual::Interactive<Self> for #name #ty_generics #where_clause {
            fn interactive(self) -> Self {
                let return_value = match self {
                    #variant_arms
                    _ => {
                        panic!("Unexpected variant");
                    }
                };
                Ok(return_value)
            }
        }
    })
}
