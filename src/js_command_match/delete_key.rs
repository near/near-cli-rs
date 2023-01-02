#[derive(Debug, Clone, clap::Parser)]
pub struct DeleteKeyArgs {
    account_id: String,
    access_key: String,
}

impl DeleteKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "account".to_owned(),
            "delete-key".to_owned(),
            self.account_id.to_owned(),
            self.access_key.to_owned(),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
