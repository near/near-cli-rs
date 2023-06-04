#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ConstructTransactionContext)]
#[interactive_clap(output_context = SkipActionContext)]
pub struct SkipAction {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SkipActionContext(super::ConstructTransactionContext);

impl SkipActionContext {
    pub fn from_previous_context(
        previous_context: super::ConstructTransactionContext,
        _scope: &<SkipAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(previous_context))
    }
}

impl From<SkipActionContext> for crate::commands::ActionContext {
    fn from(item: SkipActionContext) -> Self {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.0.signer_account_id.clone(),
                    receiver_id: item.0.receiver_account_id.clone(),
                    actions: item.0.actions.clone(),
                })
            });
        Self {
            global_context: item.0.global_context,
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
