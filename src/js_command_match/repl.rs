#[derive(Debug, Clone, clap::Parser)]
pub struct ReplArgs {
    #[clap(long, aliases = ["account_id", "accountId"])]
    account_id: Option<String>,
    #[clap(long, short)]
    script: Option<String>,
}
