#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `deploy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DeployArgs {
    contract_account_id: String,
    wasm_file: String,
    #[clap(long, aliases = ["init_function", "initFunction"])]
    init_function: Option<String>,
    #[clap(long, aliases = ["init_args", "initArgs"], default_value = "{}")]
    init_args: String,
    #[clap(long, aliases = ["init_gas", "initGas"], default_value_t = 30_000_000_000_000)]
    init_gas: u64,
    #[clap(long, aliases = ["init_deposit", "initDeposit"], default_value = "0")]
    init_deposit: String,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DeployArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
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
