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
        let key_pair_properties = crate::common::get_key_pair_properties_from_seed_phrase(
            scope.seed_phrase_hd_path.clone(),
            scope.master_seed_phrase.clone(),
        )?;
        let key_pair_properties_buf = serde_json::to_string(&key_pair_properties).unwrap();

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let config = previous_context.config.clone();

                move |network_config| {
                    super::login(
                        network_config.clone(),
                        config.credentials_home_dir.clone(),
                        &key_pair_properties_buf,
                        &key_pair_properties.public_key_str,
                        &format!(
                            "\nIt is currently not possible to verify the account access key on network <{}>.\nYou may have entered an incorrect account_id.\nYou have the option to reconfirm your account or save your access key information.\n ",
                            network_config.network_name
                        ),
                    )
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: Vec::new(),
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
        crate::transaction_signature_options::sign_with_seed_phrase::input_seed_phrase_hd_path()
    }
}
