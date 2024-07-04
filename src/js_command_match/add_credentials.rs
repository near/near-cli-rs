use crate::js_command_match::constants::{
    DEFAULT_SEED_PHRASE_PATH, NETWORK_ID_ALIASES, SECRET_KEY_ALIASES, SEED_PHRASE_ALIASES,
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `legacy` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct AddCredentialsArgs {
    account_id: String,
    #[clap(long, aliases = SEED_PHRASE_ALIASES, default_value = None, conflicts_with = "secret_key")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = SECRET_KEY_ALIASES, default_value = None)]
    secret_key: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value=None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl AddCredentialsArgs {
    pub fn to_cli_args(&self, network_config: String) -> Vec<String> {
        let network_id = self.network_id.clone().unwrap_or(network_config);

        let mut command = vec!["account".to_string(), "import-account".to_string()];

        if self.seed_phrase.is_some() {
            command.push("using-seed-phrase".to_string());
            command.push(
                self.seed_phrase
                    .to_owned()
                    .expect("You must provide valid seedPhrase"),
            );
            command.push("--seed-phrase-hd-path".to_string());
            command.push(DEFAULT_SEED_PHRASE_PATH.to_string());
        } else {
            command.push("using-private-key".to_string());
            command.push(
                self.secret_key
                    .to_owned()
                    .expect("You must provide valid secretKey or seedPhrase"),
            );
        }

        command.push("network-config".to_string());
        command.push(network_id);

        command
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn add_credentials_using_seed_phrase_testnet() {
        let seed_phrase = "seed_phrase_placeholder";

        for i in 0..SEED_PHRASE_ALIASES.len() {
            let seed_phrase_parameter_alias = &format!("--{}", &SEED_PHRASE_ALIASES[i]);
            let add_credentials_args = AddCredentialsArgs::parse_from(&[
                "near",
                "bob.testnet",
                seed_phrase_parameter_alias,
                seed_phrase,
            ]);
            let result =
                AddCredentialsArgs::to_cli_args(&add_credentials_args, "testnet".to_string());
            assert_eq!(
              result.join(" "),
              format!("account import-account using-seed-phrase {} --seed-phrase-hd-path {} network-config testnet", seed_phrase, DEFAULT_SEED_PHRASE_PATH)
          )
        }
    }

    #[test]
    fn add_credentials_using_seed_phrase_mainnet() {
        let seed_phrase = "seed_phrase_placeholder";

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);
            let seed_phrase_parameter_alias = &format!("--{}", &SEED_PHRASE_ALIASES[0]);
            let add_credentials_args = AddCredentialsArgs::parse_from(&[
                "near",
                "bob.testnet",
                seed_phrase_parameter_alias,
                seed_phrase,
                network_id_parameter_alias,
                "mainnet",
            ]);
            let result =
                AddCredentialsArgs::to_cli_args(&add_credentials_args, "testnet".to_string());
            assert_eq!(
              result.join(" "),
              format!("account import-account using-seed-phrase {} --seed-phrase-hd-path {} network-config mainnet", seed_phrase, DEFAULT_SEED_PHRASE_PATH)
          )
        }
    }

    #[test]
    fn add_credentials_using_secret_key_testnet() {
        let secret_key = "secret_key_placeholder";

        for i in 0..SECRET_KEY_ALIASES.len() {
            let secret_key_parameter_alias = &format!("--{}", &SECRET_KEY_ALIASES[i]);
            let add_credentials_args = AddCredentialsArgs::parse_from(&[
                "near",
                "bob.testnet",
                secret_key_parameter_alias,
                secret_key,
            ]);
            let result =
                AddCredentialsArgs::to_cli_args(&add_credentials_args, "testnet".to_string());
            assert_eq!(
                result.join(" "),
                format!(
                    "account import-account using-private-key {} network-config testnet",
                    secret_key
                )
            )
        }
    }
}
