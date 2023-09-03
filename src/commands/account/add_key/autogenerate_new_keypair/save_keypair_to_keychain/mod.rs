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
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.0.signer_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: signer_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::AddKey(
                            near_primitives::transaction::AddKeyAction {
                                public_key: item.0.public_key.clone(),
                                access_key: near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: item.0.permission.clone(),
                                },
                            },
                        )],
                    })
                }
            });
        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
            std::sync::Arc::new(
                move |signed_transaction, network_config, storage_message| {
                    *storage_message = crate::common::save_access_key_to_keychain(
                        network_config.clone(),
                        &serde_json::to_string(&item.0.key_pair_properties)?,
                        &item.0.key_pair_properties.public_key_str,
                        &signed_transaction.transaction.signer_id,
                    )?;
                    Ok(())
                },
            );

        Self {
            global_context: item.0.global_context,
            interacting_with_account_ids: vec![item.0.signer_account_id],
            on_after_getting_network_callback,
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
