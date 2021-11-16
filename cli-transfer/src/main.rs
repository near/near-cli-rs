use near_cli_derive::Eclap;


#[derive(Eclap)]
struct A {
    #[eclap(single, subcommand)]
    bar: B,

    #[eclap(prompt_msg = "For what baz?")]
    baz: bool,
}

#[derive(Eclap)]
struct B {
    #[eclap(prompt_msg = "For what bar?")]
    bar: bool,
}


fn main() {

}