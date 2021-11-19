use darling::{ast, FromDeriveInput, FromField, FromMeta};

#[derive(Debug, FromDeriveInput)]
// This line says that we want to process all attributes declared with `my_trait`,
// and that darling should panic if this receiver is given an enum.
#[darling(attributes(eclap), supports(struct_any))]
pub struct StructArgs {
    /// The struct ident.
    pub ident: syn::Ident,

    /// The type's generics. Will be filled in if we have any generics on our
    /// struct.
    pub generics: syn::Generics,

    /// Receives the body of the struct or enum. We don't care about
    /// struct fields because we previously told darling we only accept structs.
    pub data: ast::Data<(), FieldArgs>,

    // After the above, we can put any other fields we want to receive in the following:
    /// Set of enable flags for the struct to be used in the `#[eclap]` attribute.
    #[darling(default)]
    pub enable: Option<EnableArgs>,
}

#[derive(Debug, FromField)]
#[darling(attributes(eclap))]
pub struct FieldArgs {
    /// Field name. For tuples, newtypes, or enum bodies, this is None.
    pub ident: Option<syn::Ident>,

    /// This magic field name pulls the type from the input.
    pub ty: syn::Type,

    // After the above, we can put any other fields we want to receive in the following:
    #[darling(default)]
    pub subcommand: bool,

    /// Specifies that this is a single subcommand field
    #[darling(default)]
    pub single: bool,

    #[darling(default)]
    pub skip: bool,

    #[darling(default)]
    pub prompt_msg: Option<String>,

    #[darling(default)]
    pub prompt_fn: Option<String>,
}

#[derive(Debug, FromMeta)]
pub struct EnableArgs {
    #[darling(default)]
    pub builder_from: bool,

    #[darling(default)]
    pub validator: bool,
}

pub enum Flavor {
    None,
    Skip,
    Subcommand,
}

use std::iter::IntoIterator;

impl StructArgs {
    pub fn fields(&self) -> Vec<&FieldArgs> {
        let fields = self
            .data
            .as_ref()
            .take_struct()
            .expect("Should never be enum")
            .fields;

        fields
    }
}
