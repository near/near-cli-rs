use near_cli_derive::Eclap;


#[derive(Eclap)]
struct Welp {
    #[eclap(single, subcommand)]
    bar: bool,
    baz: bool,
}


fn main() {

}