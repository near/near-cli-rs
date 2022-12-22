#[derive(Debug, Clone, clap::Parser)]
pub struct DeployArgs {
    contract_account_id: String,
    wasm_file: String,
    init_function: String,
    init_args: String,
    init_gas: String,
    init_deposit: String,
}

impl DeployArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["contract".to_owned()];
        args.push("deploy".to_owned());
        args.push(self.contract_account_id.to_owned());
        args.push("use-file".to_owned());
        args.push(self.wasm_file.to_owned());
        args.push("with-init-call".to_owned());
        args.push(self.init_function.to_owned());
        args.push(self.init_args.to_owned());
        args.push("--prepaid-gas".to_owned());
        args.push(self.init_gas.to_owned());
        args.push("--attached-deposit".to_owned());
        args.push(self.init_deposit.to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args.push("sign-with-keychain".to_owned());
        args.push("send".to_owned());
        args
    }
}
