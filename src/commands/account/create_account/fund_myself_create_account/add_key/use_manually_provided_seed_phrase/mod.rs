use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = AddAccessWithSeedPhraseActionContext)]
pub struct AddAccessWithSeedPhraseAction {
    /// Enter the seed-phrase for this sub-account:
    master_seed_phrase: String,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

#[derive(Clone)]
struct AddAccessWithSeedPhraseActionContext(super::super::AccountPropertiesContext);

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
        let account_properties = super::super::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self(super::super::AccountPropertiesContext {
            config: previous_context.config,
            account_properties,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
        }))
    }
}

impl From<AddAccessWithSeedPhraseActionContext> for super::super::AccountPropertiesContext {
    fn from(item: AddAccessWithSeedPhraseActionContext) -> Self {
        item.0
    }
}
