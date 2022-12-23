#[derive(Debug, Clone, clap::Parser)]
pub struct CreateAccountArgs {
    account_id: String,
}

impl CreateAccountArgs {
    pub fn to_cli_args(&self) -> Vec<String> {
        vec![
            "account".to_owned(),
            "create-account".to_owned(),
            "fund-myself".to_owned(),
            self.account_id.to_owned(),
            "100 NEAR".to_owned(),
            "autogenerate-new-keypair".to_owned(),
            "save-to-keychain".to_owned(),
            "sign-as".to_owned(),
            "network-config".to_owned(),
            "testnet".to_owned(),
            "sign-with-keychain".to_owned(),
            "send".to_owned(),
        ]
    }
}
