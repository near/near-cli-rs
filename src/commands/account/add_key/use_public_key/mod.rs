#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::access_key_type::AccessTypeContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
pub struct AddAccessKeyAction {
    /// Enter the public key for this account
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct AddAccessKeyActionContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    public_key: crate::types::public_key::PublicKey,
}

impl AddAccessKeyActionContext {
    pub fn from_previous_context(
        previous_context: super::access_key_type::AccessTypeContext,
        scope: &<AddAccessKeyAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            public_key: scope.public_key.clone(),
        })
    }
}

impl From<AddAccessKeyActionContext> for crate::commands::ActionContext {
    fn from(item: AddAccessKeyActionContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id.clone(),
            receiver_account_id: item.signer_account_id,
            actions: vec![near_primitives::transaction::Action::AddKey(
                near_primitives::transaction::AddKeyAction {
                    public_key: item.public_key.into(),
                    access_key: near_primitives::account::AccessKey {
                        nonce: 0,
                        permission: item.permission,
                    },
                },
            )],
            on_after_getting_network_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
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
