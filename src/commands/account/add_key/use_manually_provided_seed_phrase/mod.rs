use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = AddAccessWithSeedPhraseActionContext)]
pub struct AddAccessWithSeedPhraseAction {
    /// Enter the seed-phrase:
    master_seed_phrase: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct AddAccessWithSeedPhraseActionContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    public_key: near_crypto::PublicKey,
}

impl AddAccessWithSeedPhraseActionContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        scope: &<AddAccessWithSeedPhraseAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let seed_phrase_hd_path_default = slip10::BIP32Path::from_str("m/44'/397'/0'").unwrap();
        let public_key = crate::common::get_public_key_from_seed_phrase(
            seed_phrase_hd_path_default,
            &scope.master_seed_phrase,
        )?;
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            public_key,
        })
    }
}

impl From<AddAccessWithSeedPhraseActionContext> for crate::commands::ActionContext {
    fn from(item: AddAccessWithSeedPhraseActionContext) -> Self {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.signer_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: signer_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::AddKey(
                            near_primitives::transaction::AddKeyAction {
                                public_key: item.public_key.clone(),
                                access_key: near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: item.permission.clone(),
                                },
                            },
                        )],
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.signer_account_id],
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
