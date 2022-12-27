#[derive(Debug, Clone, clap::Parser)]
pub struct TxStatusArgs {
    transaction_hash: String,
    #[clap(long, aliases = ["account_id", "accountId"], default_value = "near")]
    account_id: String,
}

impl TxStatusArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let network_config = std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned());
        vec![
            "transaction".to_owned(),
            "view-status".to_owned(),
            self.transaction_hash.to_owned(),
            self.account_id.to_owned(),
            "network-config".to_owned(),
            network_config,
        ]
    }
}
