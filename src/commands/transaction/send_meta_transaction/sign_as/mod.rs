use inquire::Select;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendMetaTransactionContext)]
#[interactive_clap(output_context = RelayerAccountIdContext)]
pub struct RelayerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the relayer account ID?
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
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id: near_primitives::types::AccountId =
                    scope.relayer_account_id.clone().into();
                let signed_delegate_action = previous_context.signed_delegate_action.clone();

                move |_network_config| {
                    let actions = vec![signed_delegate_action.clone().into()];

                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: signed_delegate_action.delegate_action.sender_id.clone(),
                        actions,
                    })
                }
            });

        let on_before_signing_callback: crate::commands::OnBeforeSigningCallback =
            std::sync::Arc::new({
                move |prepopulated_unsigned_transaction, _network_config| {
                    *prepopulated_unsigned_transaction.actions_mut() =
                        vec![near_primitives::transaction::Action::Delegate(Box::new(
                            previous_context.signed_delegate_action.clone(),
                        ))];
                    Ok(())
                }
            });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![scope.relayer_account_id.clone().into()],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
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
            let relayer_account_id = if let Some(account_id) =
                crate::common::input_signer_account_id_from_used_account_list(
                    &context.global_context.config.credentials_home_dir,
                    "What is the relayer account ID?",
                )? {
                account_id
            } else {
                return Ok(None);
            };

            if context.global_context.offline {
                return Ok(Some(relayer_account_id));
            }

            if !crate::common::is_account_exist(
                &context.global_context.config.network_connection,
                relayer_account_id.clone().into(),
            ) {
                eprintln!(
                    "\nThe account <{relayer_account_id}> does not exist on [{}] networks.",
                    context.global_context.config.network_names().join(", ")
                );
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
