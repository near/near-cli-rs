#[derive(Debug, Clone, clap::Parser)]
pub struct AddKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = ["contract_id", "contractId"], default_value = None)]
    contract_id: Option<String>,
    #[clap(long, aliases = ["method_names", "methodNames"], requires = "contract_id", value_delimiter = ',', num_args = 1..)]
    method_names: Vec<String>,
    #[clap(long, default_value = "0")]
    allowance: String,
}

impl AddKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        if let Some(contract_id) = self.contract_id.as_deref() {
            return vec![
                "account".to_owned(),
                "add-key".to_owned(),
                self.account_id.to_owned(),
                "grant-function-call-access".to_owned(),
                "--allowance".to_owned(),
                format!("{} NEAR", self.allowance),
                "--receiver-account-id".to_owned(),
                contract_id.to_owned(),
                "--method-names".to_owned(),
                self.method_names.join(","),
                "use-manually-provided-public-key".to_owned(),
                self.access_key.to_owned(),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ];
        }
        vec![
            "account".to_owned(),
            "add-key".to_owned(),
            self.account_id.to_owned(),
            "grant-full-access".to_owned(),
            "use-manually-provided-public-key".to_owned(),
            self.access_key.to_owned(),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
