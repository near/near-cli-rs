#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ImportAccountCommandContext)]
#[interactive_clap(output_context = LoginFromSeedPhraseContext)]
pub struct LoginFromSeedPhrase {
    /// Enter the seed-phrase for this account:
    master_seed_phrase: String,

    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,

    #[interactive_clap(named_arg)]
    /// Select network:
    network: super::network::NetworkForImportAccount,
}

#[derive(Debug, Clone)]
pub struct LoginFromSeedPhraseContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
    key_store_property: super::key_store_prop::RecoverableKey,
}

impl LoginFromSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: super::ImportAccountCommandContext,
        scope: &<LoginFromSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let master_seed_phrase = bip39::Mnemonic::parse(scope.master_seed_phrase.clone())?;

        Ok(Self {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            key_store_property: super::key_store_prop::RecoverableKey::derive(
                master_seed_phrase,
                scope.seed_phrase_hd_path.clone().into(),
            )?,
        })
    }
}

impl From<LoginFromSeedPhraseContext> for super::network::NetworkForImportAccountContext {
    fn from(item: LoginFromSeedPhraseContext) -> Self {
        let get_key_store_property_after_getting_network_callback: super::network::GetKeyStorePropertyAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |_network_config| {
                    let key_store_property = item.key_store_property.clone();

                    Ok(super::key_store_prop::KeyStorePropertyType::Recoverable(key_store_property))
                }
            });

        Self {
            global_context: item.global_context,
            account_id: item.account_id,
            get_key_store_property_after_getting_network_callback,
        }
    }
}

impl LoginFromSeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &super::ImportAccountCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_seed_phrase::input_seed_phrase_hd_path()
    }
}
