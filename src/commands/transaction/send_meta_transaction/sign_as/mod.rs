use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendMetaTransactionContext)]
#[interactive_clap(output_context = RelayerAccountIdContext)]
pub struct RelayerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    relayer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct RelayerAccountIdContext(crate::commands::ActionContext);

impl RelayerAccountIdContext {
    pub fn from_previous_context(
        previous_context: super::SendMetaTransactionContext,
        scope: &<RelayerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let signer_id: near_primitives::types::AccountId = scope.relayer_account_id.clone().into();
        let signed_delegate_action = previous_context.signed_delegate_action.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |_network_config| {
                let actions = vec![previous_context.signed_delegate_action.clone().into()];

                Ok(crate::commands::PrepopulatedTransaction {
                    signer_id: signer_id.clone(),
                    receiver_id: previous_context
                        .signed_delegate_action
                        .delegate_action
                        .sender_id
                        .clone(),
                    actions,
                })
            });

        let on_before_signing_callback: crate::commands::OnBeforeSigningCallback =
            std::sync::Arc::new({
                move |prepopulated_unsigned_transaction, _network_config| {
                    prepopulated_unsigned_transaction.actions =
                        vec![near_primitives::transaction::Action::Delegate(
                            signed_delegate_action.clone(),
                        )];
                    Ok(())
                }
            });

        Ok(Self(crate::commands::ActionContext {
            config: previous_context.config,
            on_after_getting_network_callback,
            on_refine_prepopulated_transaction_callback: std::sync::Arc::new(
                |_prepared_transaction, _network_config| Ok(()),
            ),
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome, _network_config| Ok(()),
            ),
        }))
    }
}

impl From<RelayerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: RelayerAccountIdContext) -> Self {
        item.0
    }
}

impl RelayerAccountId {
    fn input_relayer_account_id(
        context: &super::SendMetaTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let relayer_account_id: crate::types::account_id::AccountId =
                CustomType::new("What is the relayer account ID?").prompt()?;
            if !crate::common::is_account_exist(
                &context.config.network_connection,
                relayer_account_id.clone().into(),
            ) {
                eprintln!("\nThe account <{relayer_account_id}> does not yet exist.");
                #[derive(strum_macros::Display)]
                enum ConfirmOptions {
                    #[strum(to_string = "Yes, I want to enter a new account name.")]
                    Yes,
                    #[strum(to_string = "No, I want to use this account name.")]
                    No,
                }
                let select_choose_input = Select::new(
                    "Do you want to enter another relayer account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(relayer_account_id));
                }
            } else {
                return Ok(Some(relayer_account_id));
            }
        }
    }
}
