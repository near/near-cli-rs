#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::GenerateKeypairContext)]
#[interactive_clap(output_context = SaveKeypairToKeychainContext)]
pub struct SaveKeypairToKeychain {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SaveKeypairToKeychainContext(super::GenerateKeypairContext);

impl SaveKeypairToKeychainContext {
    pub fn from_previous_context(
        previous_context: super::GenerateKeypairContext,
        _scope: &<SaveKeypairToKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(previous_context))
    }
}

impl From<SaveKeypairToKeychainContext> for crate::commands::ActionContext {
    fn from(item: SaveKeypairToKeychainContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.0.signer_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: signer_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::AddKey(Box::new(
                            near_primitives::transaction::AddKeyAction {
                                public_key: item.0.public_key.clone(),
                                access_key: near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: item.0.permission.clone(),
                                },
                            },
                        ))],
                    })
                }
            });
        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
            std::sync::Arc::new(
                move |transaction, network_config| {
                    let account_id = match transaction {
                        crate::transaction_signature_options::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                            signed_transaction,
                        ) => signed_transaction.transaction.signer_id().clone(),
                        crate::transaction_signature_options::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                            signed_delegate_action,
                        ) => signed_delegate_action.delegate_action.sender_id.clone()
                    };
                    crate::common::save_access_key_to_keychain(
                        network_config.clone(),
                        &serde_json::to_string(&item.0.key_pair_properties)?,
                        &item.0.key_pair_properties.public_key_str,
                        account_id.as_ref(),
                    )
                },
            );

        Self {
            global_context: item.0.global_context,
            interacting_with_account_ids: vec![item.0.signer_account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
