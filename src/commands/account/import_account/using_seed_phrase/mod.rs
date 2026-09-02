#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = LoginFromSeedPhraseContext)]
pub struct LoginFromSeedPhrase {
    /// Enter the seed-phrase for this account:
    master_seed_phrase: String,

    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter BIP32 path for this account:
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,

    #[interactive_clap(subargs)]
    details: super::ImportAccountDetails,
}

#[derive(Debug, Clone)]
pub struct LoginFromSeedPhraseContext {
    global_context: crate::GlobalContext,
    key_store_property: super::key_store_prop::RecoverableKey,
}

impl LoginFromSeedPhraseContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<LoginFromSeedPhrase as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let master_seed_phrase = bip39::Mnemonic::parse(scope.master_seed_phrase.clone())?;
        let key_store_property = super::key_store_prop::RecoverableKey::derive(
            master_seed_phrase,
            scope.seed_phrase_hd_path.clone().into(),
        )?;

        Ok(Self {
            global_context: previous_context,
            key_store_property,
        })
    }
}

impl From<LoginFromSeedPhraseContext> for super::ImportAccountDetailsContext {
    fn from(item: LoginFromSeedPhraseContext) -> Self {
        let key_store_property =
            super::key_store_prop::KeyStorePropertyType::Recoverable(item.key_store_property);
        let on_after_getting_network_callback: super::network::OnAfterGettingNetworkConfigCallback =
            std::sync::Arc::new(move |_network_config| Ok(()));

        Self {
            global_context: item.global_context,
            key_store_property,
            on_after_getting_network_callback,
        }
    }
}

impl LoginFromSeedPhrase {
    pub fn input_seed_phrase_hd_path(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::slip10::BIP32Path>> {
        crate::transaction_signature_options::sign_with_seed_phrase::input_seed_phrase_hd_path()
    }
}
