#[derive(Debug, Clone, clap::Parser)]
pub struct LoginArgs {
    #[clap(long, aliases = ["wallet_url", "walletUrl"], default_value = "https://wallet.testnet.near.org")]
    wallet_url: String,
}

impl LoginArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        vec![
            "account".to_owned(),
            "import-account".to_owned(),
            "using-web-wallet".to_owned(),
            "network-config".to_owned(),
            network_config,
        ]
    }
}
