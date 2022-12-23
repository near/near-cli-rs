#[derive(Debug, Clone, clap::Parser)]
pub struct ViewArgs {
    contract_account_id: String,
    method_name: String,
    args: String,
}

impl ViewArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec!["contract".to_owned()];
        args.push("call-function".to_owned());
        args.push("as-read-only".to_owned());
        args.push(self.contract_account_id.to_owned());
        args.push(self.method_name.to_owned());
        args.push("json-args".to_owned());
        args.push(self.args.to_owned());
        args.push("network-config".to_owned());
        args.push("mainnet".to_owned());
        args.push("now".to_owned());
        args
    }
}
