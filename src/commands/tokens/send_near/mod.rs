use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = SendNearCommandContext)]
pub struct SendNearCommand {
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount to transfer:
    amount_in_near: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SendNearCommandContext {
    config: crate::config::Config,
    offline: bool,
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    amount_in_near: crate::common::NearBalance,
}

impl SendNearCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendNearCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            offline: previous_context.offline,
            signer_account_id: previous_context.owner_account_id,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            amount_in_near: scope.amount_in_near.clone(),
        })
    }
}

impl From<SendNearCommandContext> for crate::commands::ActionContext {
    fn from(item: SendNearCommandContext) -> Self {
        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: item.signer_account_id.clone(),
                    receiver_id: item.receiver_account_id.clone(),
                    actions: vec![near_primitives::transaction::Action::Transfer(
                        near_primitives::transaction::TransferAction {
                            deposit: item.amount_in_near.to_yoctonear(),
                        },
                    )],
                })
            });
        Self {
            config: item.config,
            offline: item.offline,
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

impl SendNearCommand {
    fn input_amount_in_near(
        _context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        let input_amount =
            CustomType::new("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)").prompt()?;
        Ok(Some(input_amount))
    }
}
