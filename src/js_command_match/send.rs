#[derive(Debug, Clone, clap::Parser)]
pub struct SendArgs {
    sender_account_id: String,
    receiver_account_id: String,
    amount: String,
}

impl SendArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        let network_config = std::env::var("NEAR_ENV").unwrap_or_else(|_| "testnet".to_owned());
        vec![
            "tokens".to_owned(),
            self.sender_account_id.to_owned(),
            "send-near".to_owned(),
            self.receiver_account_id.to_owned(),
            format!("{} NEAR", self.amount),
            "network-config".to_owned(),
            network_config,
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
