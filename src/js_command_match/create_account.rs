#[derive(Debug, Clone, clap::Parser)]
pub struct CreateAccountArgs {
    account_id: String,
}

impl CreateAccountArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["account".to_owned()];
        args.push("create-account".to_owned());
        args.push("fund-myself".to_owned());
        args.push(self.account_id.to_owned());
        args.push("100 NEAR".to_owned());
        args.push("autogenerate-new-keypair".to_owned());
        args.push("save-to-keychain".to_owned());
        args.push("sign-as".to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args.push("sign-with-keychain".to_owned());
        args.push("send".to_owned());
        args
    }
}
