#[derive(Debug, Clone, clap::Parser)]
pub struct TxStatusArgs {
    transaction_hash: String,
}

impl TxStatusArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        vec![
            "transaction".to_owned(),
            "view-status".to_owned(),
            self.transaction_hash.to_owned(),
            "near".to_owned(),
            "network-config".to_owned(),
            "testnet".to_owned(),
        ]
    }
}
