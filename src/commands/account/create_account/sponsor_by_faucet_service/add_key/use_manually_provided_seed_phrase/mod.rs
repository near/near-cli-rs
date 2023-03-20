use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = AddAccessWithSeedPhraseActionContext)]
pub struct AddAccessWithSeedPhraseAction {
    /// Enter the seed-phrase for this account
    master_seed_phrase: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: super::super::network::Network,
}

#[derive(Clone)]
struct AddAccessWithSeedPhraseActionContext(super::super::SponsorServiceContext);

impl AddAccessWithSeedPhraseActionContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        scope: &<AddAccessWithSeedPhraseAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        // This is the HD path that is used in NEAR Wallet for plaintext seed phrase generation and, subsequently, for account recovery by a seed phrase.
        let near_wallet_seed_phrase_hd_path_default =
            slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            near_wallet_seed_phrase_hd_path_default,
            &scope.master_seed_phrase,
        )?;

        Ok(Self(super::super::SponsorServiceContext {
            config: previous_context.config,
            new_account_id: previous_context.new_account_id,
            public_key,
            on_after_getting_network_callback: std::sync::Arc::new(
                |_network_config, _storage_message| Ok(()),
            ),
            on_before_creating_account_callback: previous_context
                .on_before_creating_account_callback,
        }))
    }
}

impl From<AddAccessWithSeedPhraseActionContext> for super::super::SponsorServiceContext {
    fn from(item: AddAccessWithSeedPhraseActionContext) -> Self {
        item.0
    }
}
