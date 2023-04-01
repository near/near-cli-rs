#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteKeyCommandContext)]
pub struct DeleteKeyCommand {
    /// Which account should You delete the access key for?
    owner_account_id: crate::types::account_id::AccountId,
    /// Enter the public key You wish to delete
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct DeleteKeyCommandContext {
    config: crate::config::Config,
    owner_account_id: crate::types::account_id::AccountId,
    public_key: crate::types::public_key::PublicKey,
}

impl DeleteKeyCommandContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteKeyCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            owner_account_id: scope.owner_account_id.clone(),
            public_key: scope.public_key.clone(),
        })
    }
}

impl From<DeleteKeyCommandContext> for crate::commands::ActionContext {
    fn from(item: DeleteKeyCommandContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.owner_account_id.clone().into(),
            receiver_account_id: item.owner_account_id.into(),
            actions: vec![near_primitives::transaction::Action::DeleteKey(
                near_primitives::transaction::DeleteKeyAction {
                    public_key: item.public_key.into(),
                },
            )],
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_after_getting_network_callback: std::sync::Arc::new(
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
