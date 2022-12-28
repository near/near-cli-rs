#[derive(Debug, Clone, clap::Parser)]
pub struct StakeArgs {
    account_id: String,
    staking_key: String,
    amount: String,
}
