use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{quote, ToTokens};
use syn::{
    parse_quote, Attribute, Fields, GenericParam, Generics, Ident, ItemEnum, ItemStruct, Meta,
    TypeParamBound, WhereClause,
};

pub fn contains_skip(attrs: &[Attribute]) -> bool {
    for attr in attrs.iter() {
        if let Ok(Meta::Path(path)) = attr.parse_meta() {
            if path.to_token_stream().to_string().as_str() == "interactive_skip" {
                return true;
            }
        }
    }
    false
}

pub fn add_trait_bounds(generics: &Generics, ty: TypeParamBound) -> Generics {
    let mut generics = generics.clone();
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(ty.clone());
        }
    }
    generics
}

pub fn struct_impl(input: &ItemStruct, _cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let generics = add_trait_bounds(&input.generics, parse_quote!(near_cli_visual::Interactive));
    let generics = add_trait_bounds(&generics, parse_quote!(near_cli_visual::PromptInput));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = where_clause.map_or_else(
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
                let delta = if contains_skip(&field.attrs) {
                    quote! {
                        #field_name: Default::default(),
                    }
                } else {
                    quote! {
                        #field_name: near_cli_visual::Interactive::interactive(self.#field_name),
                    }
                };

                body.extend(delta);
            }
        }
        _ => {}
    }
    Ok(quote! {
        impl #impl_generics near_cli_visual::Interactive for #name #ty_generics #where_clause {
            fn interactive(self) -> Self {
                Self { #body }
            }
        }
    })
}

pub fn enum_impl(input: &ItemEnum, _cratename: Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let generics = add_trait_bounds(&input.generics, parse_quote!(near_cli_visual::Interactive));
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let where_clause = where_clause.map_or_else(
        || WhereClause {
            where_token: Default::default(),
            predicates: Default::default(),
        },
        Clone::clone,
    );

    let mut variant_arms = TokenStream2::new();
    for variant in input.variants.iter() {
        let variant_ident = &variant.ident;

        let mut variant_header = TokenStream2::new();
        let mut variant_body = TokenStream2::new();

        if contains_skip(&variant.attrs) {
            continue;
        }

        match &variant.fields {
            Fields::Named(fields) => {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    variant_header.extend(quote! { #field_name, });
                    variant_body.extend(quote! {
                        #name::#variant_ident ( near_cli_visual::Interactive::interactive(self.#field_name) )
                    })
                }
                variant_header = quote! { { #variant_header }};
            }
            Fields::Unnamed(fields) => {
                for (field_idx, _field) in fields.unnamed.iter().enumerate() {
                    let field_ident =
                        Ident::new(format!("id{}", field_idx).as_str(), Span::call_site());
                    variant_header.extend(quote! { #field_ident, });
                    variant_body.extend(quote! {
                        #name::#variant_ident ( near_cli_visual::Interactive::interactive(#field_ident) )
                    })
                }
                variant_header = quote! { ( #variant_header )};
            }
            Fields::Unit => variant_body.extend(quote! { #name::#variant_ident }),
        }
        variant_arms.extend(quote! {
            #name::#variant_ident #variant_header => {
                #variant_body
            },
        });
    }

    Ok(quote! {
        impl #impl_generics near_cli_visual::Interactive for #name #ty_generics #where_clause {
            fn interactive(self) -> Self {
                let return_value = match self {
                    #variant_arms
                    _ => {
                        panic!("Unexpected variant");
                    }
                };
                return_value
            }
        }
    })
}
