#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct LoginArgs {
    #[clap(long, aliases = ["wallet_url", "walletUrl"], default_value = "https://wallet.testnet.near.org")]
    wallet_url: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl LoginArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let command = vec![
            "account".to_owned(),
            "import-account".to_owned(),
            "using-web-wallet".to_owned(),
            "network-config".to_owned(),
            network_id,
        ];

        command
    }
}
