#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `repl` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct ReplArgs {
    #[clap(long, aliases = ["account_id", "accountId"])]
    account_id: Option<String>,
    #[clap(long, short)]
    script: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}
