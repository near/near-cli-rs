use syn::{Ident, Type};

pub fn ident_postfix(ident: &Ident, postfix: &str) -> Ident {
    Ident::new(&format!("{}{}", ident, postfix), ident.span())
}
