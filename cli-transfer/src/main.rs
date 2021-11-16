use near_cli_derive::Eclap;


#[derive(Eclap)]
struct Welp {
    #[eclap(single, subcommand)]
    bar: bool,

    #[eclap(prompt_msg = "What is the msg of time?")]
    baz: bool,
}


fn main() {

}