#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `tx-status` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct TxStatusArgs {
    transaction_hash: String,
    #[clap(long, aliases = ["account_id", "accountId"], default_value = "near")]
    account_id: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl TxStatusArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
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
