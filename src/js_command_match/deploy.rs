#[derive(Debug, Clone, clap::Parser)]
pub struct DeployArgs {
    contract_account_id: String,
    wasm_file: String,
    init_function: String,
    init_args: String,
    #[clap(default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(default_value = "0")]
    init_deposit: String,
}

impl DeployArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        vec![
            "contract".to_owned(),
            "deploy".to_owned(),
            self.contract_account_id.to_owned(),
            "use-file".to_owned(),
            self.wasm_file.to_owned(),
            "with-init-call".to_owned(),
            self.init_function.to_owned(),
            self.init_args.to_owned(),
            "--prepaid-gas".to_owned(),
            format!("{} TeraGas", self.init_gas / 1_000_000_000_000),
            "--attached-deposit".to_owned(),
            format!("{} NEAR", self.init_deposit),
            "network-config".to_owned(),
            "testnet".to_owned(),
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
