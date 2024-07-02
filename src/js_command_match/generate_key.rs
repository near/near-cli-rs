use crate::js_command_match::constants::{
    SAVE_IMPLICIT_ALIASES,
    SEED_PHRASE_ALIASES,
    USE_LEDGER_ALIASES,
    LEDGER_PATH_ALIASES,
    NETWORK_ID_ALIASES,
    DEFAULT_SEED_PHRASE_PATH
};

#[derive(Debug, Clone, clap::Parser)]
/// This is a legacy `generate-key` command. Once you run it with the specified arguments, new syntax command will be suggested.
pub struct GenerateKeyArgs {
    account_id: Option<String>,
    #[clap(long, aliases = SAVE_IMPLICIT_ALIASES, default_value_t = false)]
    save_implicit: bool,
    #[clap(long, aliases = SEED_PHRASE_ALIASES, default_value = None, conflicts_with = "use_ledger")]
    seed_phrase: Option<String>,
    #[clap(long, aliases = USE_LEDGER_ALIASES, default_value_t = false, conflicts_with = "seed_phrase")]
    use_ledger: bool,
    #[clap(long, aliases = LEDGER_PATH_ALIASES, default_missing_value = Some(DEFAULT_SEED_PHRASE_PATH), num_args=0..=1)]
    ledger_path: Option<String>,
    #[clap(long, aliases = NETWORK_ID_ALIASES, default_value = None)]
    network_id: Option<String>,
    #[clap(allow_hyphen_values = true, num_args = 0..)]
    _unknown_args: Vec<String>,
}

impl GenerateKeyArgs {
    pub fn to_cli_args(&self, network_config: String) -> color_eyre::eyre::Result<Vec<String>> {
        let network_id = self.network_id.clone().unwrap_or(network_config);
        
        let mut command = vec![
            "account".to_string(),
            "create-account".to_string(),
            "fund-later".to_string(),
        ];
          
        let config = crate::config::Config::get_config_toml()?;
        let mut generation_method = "use-auto-generation".to_string();

        if self.use_ledger {
            generation_method = "use-ledger".to_string();
        }

        let folder_path;

        if let Some(account_id) = self.account_id.as_deref() {
            folder_path = shellexpand::tilde(
                format!(
                    "{}/{}/{}",
                    config.credentials_home_dir.to_string_lossy(),
                    network_id,
                    account_id
            )
            .as_str()).as_ref().parse()?;
        } else {
            folder_path = shellexpand::tilde(format!("{}/implicit", config.credentials_home_dir.to_string_lossy()).as_str()).as_ref().parse()?;
        }

        if let Some(seed_phrase) = self.seed_phrase.as_deref() {
            command.push("use-seed-phrase".to_string());
            command.push(seed_phrase.to_owned());
            command.push("--seed-phrase-hd-path".to_string());
            command.push("m/44'/397'/0'".to_string());
            command.push("save-to-folder".to_string());
            command.push(folder_path);

            return Ok(command);
        }
        
        command.push(generation_method);
        command.push("save-to-folder".to_string());
        command.push(folder_path);

        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
 
    #[test]
    fn generate_key_implicit_testnet() {
        let network_id = "testnet";

        let config = crate::config::Config::get_config_toml();
        let folder_path = format!("{}/implicit", config.unwrap().credentials_home_dir.to_string_lossy());

        for i in 0..NETWORK_ID_ALIASES.len() {
            let network_id_parameter_alias = &format!("--{}", &NETWORK_ID_ALIASES[i]);

            for j in 0..SAVE_IMPLICIT_ALIASES.len() {
                let save_implicit_parameter_alias = &format!("--{}", &SAVE_IMPLICIT_ALIASES[j]);
                let generate_key_args = GenerateKeyArgs::parse_from(&[
                    "near",
                    save_implicit_parameter_alias,
                    network_id_parameter_alias,
                    network_id
                ]);
                let result = GenerateKeyArgs::to_cli_args(&generate_key_args, "testnet".to_string());
                assert_eq!(
                    result.unwrap().join(" "),
                    format!(
                        "account create-account fund-later use-auto-generation save-to-folder {}",
                        folder_path,
                    )
                );
            }
            

        }
    }

    #[test]
    fn generate_key_with_ledger_testnet() {
        let account_id = "new-account.testnet";
        let seed_phrase = "crisp clump stay mean dynamic become fashion mail bike disorder chronic sight";
        let network_id = "testnet";

        let config = crate::config::Config::get_config_toml();
        let folder_path = format!(
          "{}/{}/{}",
          config.unwrap().credentials_home_dir.to_string_lossy(),
          network_id,
          account_id
        );

        for i in 0..SEED_PHRASE_ALIASES.len() {
            let seed_phrase_parameter_alias = &format!("--{}", &SEED_PHRASE_ALIASES[i]);
            let generate_key_args = GenerateKeyArgs::parse_from(&[
                "near",
                account_id,
                seed_phrase_parameter_alias,
                seed_phrase
            ]);
            let result = GenerateKeyArgs::to_cli_args(&generate_key_args, "testnet".to_string());
            assert_eq!(
                result.unwrap().join(" "),
                format!(
                    "account create-account fund-later use-seed-phrase {} --seed-phrase-hd-path m/44'/397'/0' save-to-folder {}",
                    seed_phrase,
                    folder_path,
                )
            );
        }
    }
}