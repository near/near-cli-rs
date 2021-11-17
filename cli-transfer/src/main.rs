use near_cli_derive::Eclap;
use near_cli_visual::types::{BuilderFrom, Validate, Scoped};

#[derive(Debug, Eclap)]
struct A {
    #[eclap(single, subcommand)]
    bar: B,

    #[eclap(prompt_msg = "To baz or not to?")]
    baz: bool,
}

#[derive(Debug, Eclap)]
#[eclap(disable(builder_from), enable(validator))]
struct B {
    #[eclap(prompt_msg = "To bar or not to?")]
    bar: bool,
}

impl Validate for B {
    type Err = ();

    fn validate(clap: Option<&Self::Clap>, builder: &Self::Builder) -> Result<(), Self::Err> {
        Ok(())
    }
}


fn main() {
    use near_cli_visual::types::InteractiveParse;

    let a = A::iparse();
    println!("{:?}", a);
}