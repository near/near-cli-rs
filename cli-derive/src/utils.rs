use syn::Ident;

use crate::types::{FieldArgs, StructArgs};

pub fn ident_postfix(ident: &Ident, postfix: &str) -> Ident {
    Ident::new(&format!("{}{}", ident, postfix), ident.span())
}

pub fn unwrap_ident(ident: &Option<Ident>) -> &Ident {
    ident.as_ref().expect("Enum/tuples/newtypes are unsupported")
}

pub fn fetch_subcommand(args: &StructArgs) -> Option<&FieldArgs> {
    for field_args @ FieldArgs {
        subcommand,
        ..
    } in args.fields()
    {
        if *subcommand {
            return Some(field_args);
        }
    }

    None
}
