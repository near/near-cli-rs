#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `add-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct AddKeyArgs {
    account_id: String,
    access_key: String,
    #[clap(long, aliases = ["contract_id", "contractId"], default_value = None)]
    contract_id: Option<String>,
    #[clap(long, aliases = ["method_names", "methodNames"], requires = "contract_id", value_delimiter = ',', num_args = 1..)]
    method_names: Vec<String>,
    #[clap(long, default_value = None)]
    allowance: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl AddKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        if let Some(contract_id) = self.contract_id.as_deref() {
            let allowance = if let Some(allowance) = &self.allowance {
                format!("{} NEAR", allowance)
            } else {
                "unlimited".to_string()
            };
            return vec![
                "account".to_owned(),
                "add-key".to_owned(),
                self.account_id.to_owned(),
                "grant-function-call-access".to_owned(),
                "--allowance".to_owned(),
                allowance,
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
