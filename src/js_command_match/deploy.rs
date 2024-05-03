#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `deploy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeployArgs {
    contract_account_id: Option<String>,
    #[clap(long, aliases = ["account_id", "accountId", "contract_name", "contractName"])]
    account_id: Option<String>,
    wasm_file_path: Option<String>,
    #[clap(long, aliases = ["wasm_file", "wasmFile"])]
    wasm_file: Option<String>,
    #[clap(long, aliases = ["init_function", "initFunction"])]
    init_function: Option<String>,
    #[clap(long, aliases = ["init_args", "initArgs"])]
    init_args: Option<String>,
    #[clap(long, aliases = ["init_gas", "initGas"], default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long, aliases = ["init_deposit", "initDeposit"], default_value = "0")]
    init_deposit: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeployArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let contract_account_id = if let Some(account_id) = &self.contract_account_id {
            account_id
        } else if let Some(account_id) = &self.account_id {
            account_id
        } else {
            return vec!["contract".to_owned(), "deploy".to_owned()];
        };
        let wasm_file = if let Some(file_path) = &self.wasm_file_path {
            file_path
        } else if let Some(wasm_file) = &self.wasm_file {
            wasm_file
        } else {
            return vec![
                "contract".to_owned(),
                "deploy".to_owned(),
                contract_account_id.to_owned(),
            ];
        };

        if self.init_function.is_some() {
            vec![
                "contract".to_owned(),
                "deploy".to_owned(),
                contract_account_id.to_owned(),
                "use-file".to_owned(),
                wasm_file.to_owned(),
                "with-init-call".to_owned(),
                self.init_function.as_deref().unwrap_or("new").to_owned(),
                "json-args".to_owned(),
                self.init_args.as_deref().unwrap_or("{}").to_owned(),
                "prepaid-gas".to_owned(),
                format!("{} TeraGas", self.init_gas / 1_000_000_000_000),
                "attached-deposit".to_owned(),
                format!("{} NEAR", self.init_deposit),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ]
        } else {
            vec![
                "contract".to_owned(),
                "deploy".to_owned(),
                contract_account_id.to_owned(),
                "use-file".to_owned(),
                wasm_file.to_owned(),
                "without-init-call".to_owned(),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ]
        }
    }
}
