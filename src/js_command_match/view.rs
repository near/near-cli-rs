#[derive(Debug, Clone, clap::Parser)]
pub struct ViewArgs {
    contract_account_id: String,
    method_name: String,
    args: String,
}

impl ViewArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "contract".to_owned(),
            "call-function".to_owned(),
            "as-read-only".to_owned(),
            self.contract_account_id.to_owned(),
            self.method_name.to_owned(),
            "json-args".to_owned(),
            self.args.to_owned(),
            "network-config".to_owned(),
            network_config,
            "now".to_owned(),
        ]
    }
}
