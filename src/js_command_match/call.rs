#[derive(Debug, Clone, clap::Parser)]
pub struct CallArgs {
    contract_account_id: String,
    method_name: String,
    args: String,
    #[clap(long = "masterAccount")]
    master_account: String,
}

impl CallArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        vec![
            "contract".to_owned(),
            "call-function".to_owned(),
            "as-transaction".to_owned(),
            self.contract_account_id.to_owned(),
            self.method_name.to_owned(),
            "json-args".to_owned(),
            self.args.to_owned(),
            "prepaid-gas".to_owned(),
            "30 TeraGas".to_owned(),
            "attached-deposit".to_owned(),
            "0 NEAR".to_owned(),
            "sign-as".to_owned(),
            self.master_account.to_owned(),
            "network-config".to_owned(),
            "testnet".to_owned(),
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
