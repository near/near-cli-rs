#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `dev-deploy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct DevDeployArgs {
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
    #[clap(long, aliases = ["initial_balance", "initialBalance"], default_value = "100")]
    initial_balance: String,
    #[clap(long, default_value_t = false)]
    force: bool,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl DevDeployArgs {
    pub fn to_cli_args(&self, network_config: String) {
        eprintln!("The command you tried to run is deprecated in the new NEAR CLI, but we tried our best to match the old command with the new syntax, try it instead:\n");
        eprintln!("Make sure you have the cargo-near app installed on your computer (https://github.com/near/cargo-near/blob/main/README.md)");
        eprintln!("In your project folder (cargo.toml) do the following:");
        eprintln!("1. Create a dev-account using the command:\n   cargo near create-dev-account\n");
        eprintln!(
            "2. Run the following command, after inserting the name of the created dev-account:"
        );

        if self.init_function.is_some() {
            eprintln!(
                "   {}",
                shell_words::join(vec![
                    "cargo".to_owned(),
                    "near".to_owned(),
                    "deploy".to_owned(),
                    "<created-dev-account>".to_owned(),
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
                ])
            );
        } else {
            eprintln!(
                "   {}",
                shell_words::join(vec![
                    "cargo".to_owned(),
                    "near".to_owned(),
                    "deploy".to_owned(),
                    "<created-dev-account>".to_owned(),
                    "without-init-call".to_owned(),
                    "network-config".to_owned(),
                    network_config,
                    "sign-with-keychain".to_owned(),
                    "send".to_owned(),
                ])
            );
        }
    }
}
