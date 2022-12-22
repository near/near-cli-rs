#[derive(Debug, Clone, clap::Parser)]
pub struct TxStatusArgs {
    transaction_hash: String,
}

impl TxStatusArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["transaction".to_owned()];
        args.push("view-status".to_owned());
        args.push(self.transaction_hash.to_owned());
        args.push("near".to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args
    }
}
