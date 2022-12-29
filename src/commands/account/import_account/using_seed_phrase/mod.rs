use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct LoginFromSeedPhrase {
    /// Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

impl LoginFromSeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::slip10::BIP32Path> {
        Ok(crate::types::slip10::BIP32Path::from_str(
            &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default)")
                .with_initial_value("m/44'/397'/0'")
                .prompt()
                .unwrap(),
        )
        .unwrap())
    }

    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            self.seed_phrase_hd_path.clone(),
            self.master_seed_phrase.clone(),
        )?;
        let key_pair_properties_buf = serde_json::to_string(&key_pair_properties).unwrap();
        let error_message = "\nIt is currently not possible to verify the account access key.\nYou may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n";
        super::login(
            network_config,
            config.credentials_home_dir,
            &key_pair_properties_buf,
            &key_pair_properties.public_key_str,
            error_message,
        )
        .await
    }
}
