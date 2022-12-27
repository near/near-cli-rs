#[derive(Debug, Clone, clap::Parser)]
pub struct DeleteArgs {
    account_id: String,
    beneficiary_id: String,
}

impl DeleteArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "account".to_owned(),
            "delete-account".to_owned(),
            self.account_id.to_owned(),
            "beneficiary".to_owned(),
            self.beneficiary_id.to_owned(),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
