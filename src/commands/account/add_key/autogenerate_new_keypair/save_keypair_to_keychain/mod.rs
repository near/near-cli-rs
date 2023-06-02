use color_eyre::eyre::Context;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::GenerateKeypairContext)]
#[interactive_clap(output_context = SaveKeypairToKeychainContext)]
pub struct SaveKeypairToKeychain {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SaveKeypairToKeychainContext {
    config: crate::config::Config,
    offline: bool,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    key_pair_properties: crate::common::KeyPairProperties,
    public_key: near_crypto::PublicKey,
}

impl SaveKeypairToKeychainContext {
    pub fn from_previous_context(
        previous_context: super::GenerateKeypairContext,
        _scope: &<SaveKeypairToKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            offline: previous_context.offline,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            key_pair_properties: previous_context.key_pair_properties,
            public_key: previous_context.public_key,
        })
    }
}

impl From<SaveKeypairToKeychainContext> for crate::commands::ActionContext {
    fn from(item: SaveKeypairToKeychainContext) -> Self {
        let credentials_home_dir = item.config.credentials_home_dir.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.signer_account_id.clone(),
                    receiver_id: item.signer_account_id.clone(),
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
            });

        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
            std::sync::Arc::new(
                move |signed_transaction, network_config, storage_message| {
                    let key_pair_properties_buf = serde_json::to_string(&item.key_pair_properties)?;
                    *storage_message = crate::common::save_access_key_to_keychain(
                        network_config.clone(),
                        credentials_home_dir.clone(),
                        &key_pair_properties_buf,
                        &item.key_pair_properties.public_key_str,
                        &signed_transaction.transaction.signer_id,
                    )
                    .wrap_err_with(|| {
                        format!(
                            "Failed to save a file with access key: {}",
                            &item.key_pair_properties.public_key_str
                        )
                    })?;
                    Ok(())
                },
            );

        Self {
            config: item.config,
            offline: item.offline,
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
