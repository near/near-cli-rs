#[derive(Debug, Clone, clap::Parser)]
pub struct KeysArgs {
    account_id: String,
}

impl KeysArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "account".to_owned(),
            "list-keys".to_owned(),
            self.account_id.to_owned(),
            "network-config".to_owned(),
            network_config,
            "now".to_owned(),
        ]
    }
}
