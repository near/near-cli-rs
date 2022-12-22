#[derive(Debug, Clone, clap::Parser)]
pub struct DeleteArgs {
    account_id: String,
    beneficiary_id: String,
}

impl DeleteArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["account".to_owned()];
        args.push("delete-account".to_owned());
        args.push(self.account_id.to_owned());
        args.push("beneficiary".to_owned());
        args.push(self.beneficiary_id.to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args.push("sign-with-keychain".to_owned());
        args.push("send".to_owned());
        args
    }
}
