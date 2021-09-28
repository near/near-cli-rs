mod impls;

use impls::*;
use clap::Clap;


#[derive(Debug, Clap)]
struct TopLevel {
    #[clap(subcommand)]
    cli: Option<CliQueryRequest>
}

fn main() {
    let x = TopLevel::parse();
    println!("{:?}", x);
}
