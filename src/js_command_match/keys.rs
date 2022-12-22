#[derive(Debug, Clone, clap::Parser)]
pub struct KeysArgs {
    account_id: String,
}

impl KeysArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["account".to_owned()];
        args.push("list-keys".to_owned());
        args.push(self.account_id.to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args.push("now".to_owned());
        args
    }
}
