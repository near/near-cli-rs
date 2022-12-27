#[derive(Debug, Clone, clap::Parser)]
pub struct CallArgs {
    contract_account_id: String,
    method_name: String,
    args: String,
    #[clap(long, aliases = ["account_id", "accountId"])]
    account_id: String,
    #[clap(long, default_value_t = 30_000_000_000_000)]
    gas: u64,
    #[clap(long, default_value = "0")]
    deposit: String,
}

impl CallArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "contract".to_owned(),
            "call-function".to_owned(),
            "as-transaction".to_owned(),
            self.contract_account_id.to_owned(),
            self.method_name.to_owned(),
            "json-args".to_owned(),
            self.args.to_owned(),
            "prepaid-gas".to_owned(),
            format!("{} TeraGas", self.gas / 1_000_000_000_000),
            "attached-deposit".to_owned(),
            format!("{} NEAR", self.deposit),
            "sign-as".to_owned(),
            self.account_id.to_owned(),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
