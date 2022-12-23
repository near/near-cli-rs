#[derive(Debug, Clone, clap::Parser)]
pub struct StateArgs {
    account_id: String,
}

impl StateArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        vec![
            "account".to_owned(),
            "view-account-summary".to_owned(),
            self.account_id.to_owned(),
            "network-config".to_owned(),
            "testnet".to_owned(),
            "now".to_owned(),
        ]
    }
}
