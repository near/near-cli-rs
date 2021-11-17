use near_cli_derive::Eclap;
use near_cli_visual::types::Validate;

#[derive(Debug, Eclap)]
struct A {
    #[eclap(single, subcommand)]
    bar: B,

    #[eclap(prompt_msg = "To baz or not to?")]
    baz: bool,
}

#[derive(Debug, Eclap)]
#[eclap(enable(validator))]
struct B {
    #[eclap(prompt_msg = "To bar or not to?")]
    bar: bool,
}

impl Validate for B {
    type Err = ();

    fn validate(_clap: Option<&Self::Clap>, _builder: &Self::Builder) -> Result<(), Self::Err> {
        Ok(())
    }
}

fn main() {
    use near_cli_visual::types::InteractiveParse;

    let a = A::iparse();
    println!("{:?}", a);
}
