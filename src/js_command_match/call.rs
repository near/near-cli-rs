#[derive(Debug, Clone, clap::Parser)]
pub struct CallArgs {
    contract_account_id: String,
    method_name: String,
    args: String,
    #[clap(long = "masterAccount")]
    master_account: String,
}

impl CallArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["contract".to_owned()];
        args.push("call-function".to_owned());
        args.push("as-transaction".to_owned());
        args.push(self.contract_account_id.to_owned());
        args.push(self.method_name.to_owned());
        args.push("json-args".to_owned());
        args.push(self.args.to_owned());
        args.push("prepaid-gas".to_owned());
        args.push("30 TeraGas".to_owned());
        args.push("attached-deposit".to_owned());
        args.push("0 NEAR".to_owned());
        args.push("sign-as".to_owned());
        args.push(self.master_account.to_owned());
        args.push("network-config".to_owned());
        args.push("testnet".to_owned());
        args.push("sign-with-keychain".to_owned());
        args.push("send".to_owned());
        args
    }
}
