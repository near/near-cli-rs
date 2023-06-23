use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromSeedPhraseContext)]
pub struct LoginFromSeedPhrase {
    /// Enter the seed-phrase for this account:
    master_seed_phrase: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct LoginFromSeedPhraseContext(crate::network::NetworkContext);

impl LoginFromSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<LoginFromSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let config = previous_context.config.clone();
        let seed_phrase_hd_path = scope.seed_phrase_hd_path.clone();
        let master_seed_phrase = scope.master_seed_phrase.clone();
        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            seed_phrase_hd_path,
            master_seed_phrase,
        )?;
        let key_pair_properties_buf = serde_json::to_string(&key_pair_properties).unwrap();
        let error_message = "\nIt is currently not possible to verify the account access key.\nYou may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n";

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    super::login(
                        network_config.clone(),
                        config.credentials_home_dir.clone(),
                        &key_pair_properties_buf,
                        &key_pair_properties.public_key_str,
                        error_message,
                    )
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<LoginFromSeedPhraseContext> for crate::network::NetworkContext {
    fn from(item: LoginFromSeedPhraseContext) -> Self {
        item.0
    }
}

impl LoginFromSeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        Ok(Some(
            crate::types::slip10::BIP32Path::from_str(
                &Text::new("Enter seed phrase HD Path (if you not sure leave blank for default):")
                    .with_initial_value("m/44'/397'/0'")
                    .prompt()
                    .unwrap(),
            )
            .unwrap(),
        ))
    }
}
