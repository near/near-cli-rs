#[derive(Debug, Clone, clap::Parser)]
pub struct DeployArgs {
    #[clap(long = "accountId")]
    contract_account_id: String,
    #[clap(long = "wasmFile")]
    wasm_file: String,
    #[clap(long = "initFunction")]
    init_function: Option<String>,
    #[clap(long = "initArgs", default_value = "{}")]
    init_args: String,
    #[clap(long = "initGas", default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long = "initDeposit", default_value = "0")]
    init_deposit: String,
}

impl DeployArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let network_config = std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned());
        if let Some(init_function) = self.init_function.as_deref() {
            vec![
                "contract".to_owned(),
                "deploy".to_owned(),
                self.contract_account_id.to_owned(),
                "use-file".to_owned(),
                self.wasm_file.to_owned(),
                "with-init-call".to_owned(),
                init_function.to_owned(),
                self.init_args.to_owned(),
                "--prepaid-gas".to_owned(),
                format!("{} TeraGas", self.init_gas / 1_000_000_000_000),
                "--attached-deposit".to_owned(),
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
                self.contract_account_id.to_owned(),
                "use-file".to_owned(),
                self.wasm_file.to_owned(),
                "without-init-call".to_owned(),
                "network-config".to_owned(),
                network_config,
                "sign-with-keychain".to_owned(),
                "send".to_owned(),
            ]
        }
    }
}
