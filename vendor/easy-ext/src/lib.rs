//! An attribute macro for easily writing [extension trait pattern][rfc0445].
//!
//! # Examples
//!
//! ```rust
//! use easy_ext::ext;
//!
//! #[ext(ResultExt)]
//! impl<T, E> Result<T, E> {
//!     pub fn err_into<U>(self) -> Result<T, U>
//!     where
//!         E: Into<U>,
//!     {
//!         self.map_err(Into::into)
//!     }
//! }
//! ```
//!
//! Code like this will be generated:
//!
//! ```rust
//! pub trait ResultExt<T, E> {
//!     fn err_into<U>(self) -> Result<T, U>
//!     where
//!         E: Into<U>;
//! }
//!
//! impl<T, E> ResultExt<T, E> for Result<T, E> {
//!     fn err_into<U>(self) -> Result<T, U>
//!     where
//!         E: Into<U>,
//!     {
//!         self.map_err(Into::into)
//!     }
//! }
//! ```
//!
//! You can elide the trait name.
//!
//! ```rust
//! use easy_ext::ext;
//!
//! #[ext]
//! impl<T, E> Result<T, E> {
//!     fn err_into<U>(self) -> Result<T, U>
//!     where
//!         E: Into<U>,
//!     {
//!         self.map_err(Into::into)
//!     }
//! }
//! ```
//!
//! Note that in this case, `#[ext]` assigns a random name, so you cannot
//! import/export the generated trait.
//!
//! ## Visibility
//!
//! There are two ways to specify visibility.
//!
//! ### Impl-level visibility
//!
//! The first way is to specify visibility as the first argument to the `#[ext]`
//! attribute. For example:
//!
//! ```rust
//! use easy_ext::ext;
//!
//! // unnamed
//! #[ext(pub)]
//! impl str {
//!     fn foo(&self) {}
//! }
//!
//! // named
//! #[ext(pub StrExt)]
//! impl str {
//!     fn bar(&self) {}
//! }
//! ```
//!
//! ### Associated-item-level visibility
//!
//! Another way is to specify visibility at the associated item level.
//!
//! For example, if the method is `pub` then the trait will also be `pub`:
//!
//! ```rust
//! use easy_ext::ext;
//!
//! #[ext(ResultExt)] // generate `pub trait ResultExt`
//! impl<T, E> Result<T, E> {
//!     pub fn err_into<U>(self) -> Result<T, U>
//!     where
//!         E: Into<U>,
//!     {
//!         self.map_err(Into::into)
//!     }
//! }
//! ```
//!
//! This is useful when migrate from an inherent impl to an extension trait.
//!
//! Note that the visibility of all the associated items in the `impl` must be identical.
//!
//! Note that you cannot specify impl-level visibility and associated-item-level visibility at the same time.
//!
//! ## [Supertraits](https://doc.rust-lang.org/reference/items/traits.html#supertraits)
//!
//! If you want the extension trait to be a subtrait of another trait,
//! add `Self: SubTrait` bound to the `where` clause.
//!
//! ```rust
//! use easy_ext::ext;
//!
//! #[ext(Ext)]
//! impl<T> T
//! where
//!     Self: Default,
//! {
//!     fn method(&self) {}
//! }
//! ```
//!
//! ## Supported items
//!
//! ### [Associated functions (methods)](https://doc.rust-lang.org/book/ch05-03-method-syntax.html)
//!
//! ```rust
//! use easy_ext::ext;
//!
//! #[ext(Ext)]
//! impl<T> T {
//!     fn method(&self) {}
//! }
//! ```
//!
//! ### [Associated constants](https://doc.rust-lang.org/edition-guide/rust-2018/trait-system/associated-constants.html)
//!
//! ```rust
//! use easy_ext::ext;
//!
//! #[ext(Ext)]
//! impl<T> T {
//!     const MSG: &'static str = "Hello!";
//! }
//! ```
//!
//! [rfc0445]: https://github.com/rust-lang/rfcs/blob/master/text/0445-extension-trait-conventions.md

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms, single_use_lifetimes),
        allow(dead_code, unused_variables)
    )
))]
#![forbid(unsafe_code)]
#![warn(future_incompatible, rust_2018_idioms, unreachable_pub)]
// It cannot be included in the published code because these lints have false positives in the minimum required version.
#![cfg_attr(test, warn(single_use_lifetimes))]
#![warn(clippy::all, clippy::default_trait_access)]

// older compilers require explicit `extern crate`.
#[allow(unused_extern_crates)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use std::{collections::hash_map::DefaultHasher, hash::Hasher, mem};
use syn::{
    parse::{Parse, ParseStream},
    parse_quote,
    punctuated::Punctuated,
    token,
    visit_mut::VisitMut,
    Attribute, Error, GenericParam, Generics, Ident, ImplItem, ItemImpl, ItemTrait, PredicateType,
    Result, Token, TraitItem, TraitItemConst, TraitItemMethod, Type, TypeParam, TypePath,
    Visibility, WherePredicate,
};

macro_rules! error {
    ($span:expr, $msg:expr) => {
        syn::Error::new_spanned(&$span, $msg)
    };
    ($span:expr, $($tt:tt)*) => {
        error!($span, format!($($tt)*))
    };
}

/// An attribute macro for easily writing [extension trait pattern][rfc0445].
///
/// See crate level documentation for details.
///
/// [rfc0445]: https://github.com/rust-lang/rfcs/blob/master/text/0445-extension-trait-conventions.md
#[proc_macro_attribute]
pub fn ext(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut args: Args = syn::parse_macro_input!(args);
    if args.name.is_none() {
        args.name = Some(format_ident!("__ExtTrait{}", hash(&input)));
    }

    let mut item: ItemImpl = syn::parse_macro_input!(input);

    trait_from_impl(&mut item, args)
        .map(ToTokens::into_token_stream)
        .map(|mut tokens| {
            tokens.extend(item.into_token_stream());
            tokens
        })
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

struct Args {
    // impl-level visibility
    vis: Option<Visibility>,
    // trait name
    name: Option<Ident>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let vis: Visibility = input.parse()?;
        let name: Option<Ident> = input.parse()?;
        Ok(Args { vis: if let Visibility::Inherited = vis { None } else { Some(vis) }, name })
    }
}

fn determine_trait_generics<'a>(generics: &mut Generics, self_ty: &'a Type) -> Option<&'a Ident> {
    if let Type::Path(TypePath { path, qself: None }) = self_ty {
        if let Some(ident) = path.get_ident() {
            let i = generics.params.iter().position(|param| {
                if let GenericParam::Type(param) = param { param.ident == *ident } else { false }
            });
            if let Some(i) = i {
                let mut params = mem::replace(&mut generics.params, Punctuated::new())
                    .into_iter()
                    .collect::<Vec<_>>();
                let param = params.remove(i);
                generics.params = params.into_iter().collect();

                if let GenericParam::Type(TypeParam {
                    colon_token: Some(colon_token),
                    bounds,
                    ..
                }) = param
                {
                    generics.make_where_clause().predicates.push(WherePredicate::Type(
                        PredicateType {
                            lifetimes: None,
                            bounded_ty: parse_quote!(Self),
                            colon_token,
                            bounds,
                        },
                    ));
                }

                return Some(ident);
            }
        }
    }
    None
}

fn trait_from_impl(item: &mut ItemImpl, args: Args) -> Result<ItemTrait> {
    /// Replace `self_ty` with `Self`.
    struct ReplaceParam<'a> {
        self_ty: &'a Ident,
    }

    impl VisitMut for ReplaceParam<'_> {
        fn visit_ident_mut(&mut self, ident: &mut Ident) {
            if *ident == *self.self_ty {
                *ident = Ident::new("Self", ident.span());
            }
        }
    }

    let name = args.name.unwrap();
    let mut generics = item.generics.clone();
    let mut visitor = determine_trait_generics(&mut generics, &item.self_ty)
        .map(|self_ty| ReplaceParam { self_ty });

    if let Some(visitor) = &mut visitor {
        visitor.visit_generics_mut(&mut generics);
    }
    let ty_generics = generics.split_for_impl().1;
    let trait_ = parse_quote!(#name #ty_generics);
    item.trait_ = Some((None, trait_, <Token![for]>::default()));

    // impl-level visibility
    let impl_vis = args.vis;
    // assoc-item-level visibility
    let mut assoc_vis = None;
    let mut items = Vec::with_capacity(item.items.len());
    item.items.iter_mut().try_for_each(|item| {
        trait_item_from_impl_item(item, &mut assoc_vis, &impl_vis).map(|mut item| {
            if let Some(visitor) = &mut visitor {
                visitor.visit_trait_item_mut(&mut item);
            }
            items.push(item)
        })
    })?;

    let mut attrs = item.attrs.clone();
    find_remove(&mut item.attrs, "doc"); // https://github.com/taiki-e/easy-ext/issues/20
    attrs.push(parse_quote!(#[allow(patterns_in_fns_without_body)])); // mut self

    Ok(ItemTrait {
        attrs,
        // impl-level visibility > assoc-item-level visibility > inherited visibility
        vis: impl_vis.unwrap_or_else(|| assoc_vis.unwrap_or(Visibility::Inherited)),
        unsafety: item.unsafety,
        auto_token: None,
        trait_token: <Token![trait]>::default(),
        ident: name,
        generics,
        colon_token: None,
        supertraits: Punctuated::new(),
        brace_token: token::Brace::default(),
        items,
    })
}

fn trait_item_from_impl_item(
    impl_item: &mut ImplItem,
    prev_vis: &mut Option<Visibility>,
    impl_vis: &Option<Visibility>,
) -> Result<TraitItem> {
    fn compare_visibility(x: &Visibility, y: &Visibility) -> bool {
        match (x, y) {
            (Visibility::Public(_), Visibility::Public(_))
            | (Visibility::Crate(_), Visibility::Crate(_))
            | (Visibility::Inherited, Visibility::Inherited) => true,
            (Visibility::Restricted(x), Visibility::Restricted(y)) => {
                x.to_token_stream().to_string() == y.to_token_stream().to_string()
            }
            _ => false,
        }
    }

    fn check_visibility(
        current: Visibility,
        prev: &mut Option<Visibility>,
        impl_vis: &Option<Visibility>,
        span: &dyn ToTokens,
    ) -> Result<()> {
        if impl_vis.is_some() {
            return if let Visibility::Inherited = current {
                Ok(())
            } else {
                Err(error!(current, "all associated items must have inherited visibility"))
            };
        }
        match prev {
            None => *prev = Some(current),
            Some(prev) if compare_visibility(prev, &current) => {}
            Some(prev) => {
                return if let Visibility::Inherited = prev {
                    Err(error!(current, "all associated items must have inherited visibility"))
                } else {
                    Err(error!(
                        if let Visibility::Inherited = current { span } else { &current },
                        "all associated items must have a visibility of `{}`",
                        prev.to_token_stream(),
                    ))
                };
            }
        }
        Ok(())
    }

    match impl_item {
        ImplItem::Const(impl_const) => {
            let vis = mem::replace(&mut impl_const.vis, Visibility::Inherited);
            check_visibility(vis, prev_vis, impl_vis, &impl_const.ident)?;

            let attrs = impl_const.attrs.clone();
            find_remove(&mut impl_const.attrs, "doc"); // https://github.com/taiki-e/easy-ext/issues/20
            Ok(TraitItem::Const(TraitItemConst {
                attrs,
                const_token: impl_const.const_token,
                ident: impl_const.ident.clone(),
                colon_token: impl_const.colon_token,
                ty: impl_const.ty.clone(),
                default: None,
                semi_token: impl_const.semi_token,
            }))
        }
        ImplItem::Method(impl_method) => {
            let vis = mem::replace(&mut impl_method.vis, Visibility::Inherited);
            check_visibility(vis, prev_vis, impl_vis, &impl_method.sig.ident)?;

            let mut attrs = impl_method.attrs.clone();
            find_remove(&mut impl_method.attrs, "doc"); // https://github.com/taiki-e/easy-ext/issues/20
            find_remove(&mut attrs, "inline"); // `#[inline]` is ignored on function prototypes
            Ok(TraitItem::Method(TraitItemMethod {
                attrs,
                sig: impl_method.sig.clone(),
                default: None,
                semi_token: Some(Token![;](impl_method.block.brace_token.span)),
            }))
        }
        _ => Err(error!(impl_item, "unsupported item")),
    }
}

fn find_remove(attrs: &mut Vec<Attribute>, ident: &str) {
    while let Some(i) = attrs.iter().position(|attr| attr.path.is_ident(ident)) {
        attrs.remove(i);
    }
}

/// Returns the hash value of the input AST.
fn hash(input: &TokenStream) -> u64 {
    let mut hasher = DefaultHasher::new();
    hasher.write(input.to_string().as_bytes());
    hasher.finish()
}
