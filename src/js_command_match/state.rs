#[derive(Debug, Clone, clap::Parser)]
pub struct StateArgs {
    account_id: String,
}

impl StateArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["account".to_owned()];
        args.push("view-account-summary".to_owned());
        args.push(self.account_id.to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args.push("now".to_owned());
        args
    }
}
