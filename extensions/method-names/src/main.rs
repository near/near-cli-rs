use clap::{Parser, Subcommand};

mod commands;
mod common;
mod consts;
mod types;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Online(commands::Online),
    // Wasm file of contract
    Wasm(commands::Wasm),
}

impl Commands {
    async fn process(self) {
        match self {
            Commands::Online(val) => val.process().await,
            Commands::Wasm(val) => val.process(),
        }
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    cli.command.process().await;
}
