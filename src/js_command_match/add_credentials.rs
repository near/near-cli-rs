const DEFAULT_SEED_PHRASE_PATH: &str = "m/44'/397'/0'";

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct AddCredentialsArgs {
    account_id: String,
    #[clap(long, aliases = ["seedPhrase"], default_value = None, conflicts_with = "secret_key")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = ["secretKey"], default_value = None)]
    secret_key: Option<String>,
    #[clap(long, aliases = ["networkId"], default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl AddCredentialsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config.to_owned());

        let mut command = vec![
            "account".to_owned(),
            "import-account".to_owned(),
        ];

        if self.seed_phrase.is_some() {
            command.push("using-seed-phrase".to_owned());
            command.push(self.seed_phrase.to_owned().expect("You must provide valid seedPhrase"));
            command.push("--seed-phrase-hd-path".to_owned());
            command.push(DEFAULT_SEED_PHRASE_PATH.to_owned());
        } else {
            command.push("using-private-key".to_owned());
            command.push(self.secret_key.to_owned().expect("You must provide valid secretKey or seedPhrase"));
        }
        
        command.push("network-config".to_owned());
        command.push(network_id);

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_credentials_using_seed_phrase() {
        let seed_phrase = "seed_phrase_placeholder".to_string();
        let add_credentials_args = AddCredentialsArgs {
            account_id: "bob.testnet".to_string(),
            seed_phrase: Some(seed_phrase.clone()),
            secret_key: None,
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = AddCredentialsArgs::to_cli_args(&add_credentials_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("account import-account using-seed-phrase {} --seed-phrase-hd-path {} network-config testnet", seed_phrase, DEFAULT_SEED_PHRASE_PATH.to_string())
        )
    }

    #[test]
    fn add_credentials_using_secret_key() {
        let secret_key = "secret_key_placeholder".to_string();
        let add_credentials_args = AddCredentialsArgs {
            account_id: "bob.testnet".to_string(),
            seed_phrase: None,
            secret_key: Some(secret_key.clone()),
            network_id: None,
            _unknown_args: [].to_vec(),
        };
        let result = AddCredentialsArgs::to_cli_args(&add_credentials_args, "testnet".to_string());
        assert_eq!(
            result.join(" "),
            format!("account import-account using-private-key {} network-config testnet", secret_key)
        )
    }
}