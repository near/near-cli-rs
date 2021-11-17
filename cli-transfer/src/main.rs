use near_cli_derive::Eclap;
use near_cli_visual::types::{BuilderFrom, Validate, Scoped};

#[derive(Debug, Eclap)]
// #[eclap(enable(validator))]
struct A {
    #[eclap(single, subcommand)]
    bar: B,

    #[eclap(prompt_msg = "To baz or not to?")]
    baz: bool,
}

impl Validate for A {
    type Err = ();

    fn validate(clap: Option<&Self::Clap>, builder: &Self::Builder) -> Result<(), Self::Err> {
        Ok(())
    }
}

#[derive(Debug, Eclap)]
struct B {
    #[eclap(prompt_msg = "To bar or not to?")]
    bar: bool,
}

impl BuilderFrom<A> for B {
    fn builder_from(a: &<A as Scoped>::Scope) -> Self::Builder {
        Self::Builder {
            bar: None,
        }
    }
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