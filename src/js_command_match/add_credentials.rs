#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct AddCredentialsArgs {
    account_id: String,
    #[clap(long, aliases = ["network_id", "networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(long, aliases = ["seed_phrase", "seedPhrase"], default_value = None, conflicts_with = "secret_key")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = ["secret_key", "secretKey"], default_value = None)]
    secret_key: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl AddCredentialsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        if self.seed_phrase.is_some() {
            vec![
                "account".to_owned(),
                "import-account".to_owned(),
                "using-seed-phrase".to_owned(),
                self.seed_phrase.to_owned().expect("You must provide valid seedPhrase"),
                "--seed-phrase-hd-path".to_owned(),
                "m/44'/397'/0'".to_owned(),
                "network-config".to_owned(),
                self.network_id.clone().unwrap_or(network_config.to_owned()),
            ]
        } else {
            vec![
                "account".to_owned(),
                "import-account".to_owned(),
                "using-private-key".to_owned(),
                self.secret_key.to_owned().expect("You must provide valid secretKey or seedPhrase"),
                "network-config".to_owned(),
                self.network_id.clone().unwrap_or(network_config.to_owned()),
            ]
        }
    }
}
