use color_eyre::eyre::Context;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::GenerateKeypairContext)]
#[interactive_clap(output_context = SaveKeypairToLegacyKeychainContext)]
pub struct SaveKeypairToLegacyKeychain {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SaveKeypairToLegacyKeychainContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    key_pair_properties: crate::common::KeyPairProperties,
    public_key: near_crypto::PublicKey,
}

impl SaveKeypairToLegacyKeychainContext {
    pub fn from_previous_context(
        previous_context: super::GenerateKeypairContext,
        _scope: &<SaveKeypairToLegacyKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            key_pair_properties: previous_context.key_pair_properties,
            public_key: previous_context.public_key,
        })
    }
}

impl From<SaveKeypairToLegacyKeychainContext> for crate::commands::ActionContext {
    fn from(item: SaveKeypairToLegacyKeychainContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.signer_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: signer_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::AddKey(Box::new(
                            near_primitives::transaction::AddKeyAction {
                                public_key: item.public_key.clone(),
                                access_key: near_primitives::account::AccessKey {
                                    nonce: 0,
                                    permission: item.permission.clone(),
                                },
                            },
                        ))],
                    })
                }
            });

        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
            std::sync::Arc::new({
                let credentials_home_dir = item.global_context.config.credentials_home_dir.clone();

                move |transaction, network_config| {
                    let account_id = match transaction {
                        crate::transaction_signature_options::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                            signed_transaction,
                        ) => signed_transaction.transaction.signer_id().clone(),
                        crate::transaction_signature_options::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                            signed_delegate_action,
                        ) => signed_delegate_action.delegate_action.sender_id.clone()
                    };
                    let key_pair_properties_buf = serde_json::to_string(&item.key_pair_properties)?;
                    crate::common::save_access_key_to_legacy_keychain(
                        network_config.clone(),
                        credentials_home_dir.clone(),
                        &key_pair_properties_buf,
                        &item.key_pair_properties.public_key_str,
                        account_id.as_ref(),
                    )
                    .wrap_err_with(|| {
                        format!(
                            "Failed to save a file with access key: {}",
                            &item.key_pair_properties.public_key_str
                        )
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.signer_account_id],
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
