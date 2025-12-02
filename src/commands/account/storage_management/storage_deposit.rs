use inquire::Select;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ContractContext)]
#[interactive_clap(output_context = DepositArgsContext)]
pub struct DepositArgs {
    #[interactive_clap(skip_default_input_arg)]
    /// Which account ID do you want to add a deposit to?
    receiver_account_id: crate::types::account_id::AccountId,
    /// Enter the amount to deposit into the storage (example: 10NEAR or 0.5near or 10000yoctonear):
    deposit: crate::types::near_token::NearToken,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: SignerAccountId,
}

#[derive(Clone)]
pub struct DepositArgsContext {
    global_context: crate::GlobalContext,
    get_contract_account_id: super::GetContractAccountId,
    receiver_account_id: near_primitives::types::AccountId,
    deposit: crate::types::near_token::NearToken,
}

impl DepositArgsContext {
    pub fn from_previous_context(
        previous_context: super::ContractContext,
        scope: &<DepositArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            get_contract_account_id: previous_context.get_contract_account_id,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            deposit: scope.deposit,
        })
    }
}

impl DepositArgs {
    fn input_receiver_account_id(
        context: &super::ContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let receiver_account_id = if let Some(account_id) =
                crate::common::input_signer_account_id_from_used_account_list(
                    &context.global_context.config.credentials_home_dir,
                    "Which account ID do you want to add a deposit to?",
                )? {
                account_id
            } else {
                return Ok(None);
            };

            if context.global_context.offline {
                return Ok(Some(receiver_account_id));
            }

            if !crate::common::is_account_exist(
                &context.global_context.config.network_connection,
                receiver_account_id.clone().into(),
            )? {
                eprintln!(
                    "\nThe account <{receiver_account_id}> does not exist on [{}] networks.",
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
                    "Do you want to enter another receiver account id?",
                    vec![ConfirmOptions::Yes, ConfirmOptions::No],
                )
                .prompt()?;
                if let ConfirmOptions::No = select_choose_input {
                    return Ok(Some(receiver_account_id));
                }
            } else {
                return Ok(Some(receiver_account_id));
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DepositArgsContext)]
#[interactive_clap(output_context = SignerAccountIdContext)]
pub struct SignerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SignerAccountIdContext(crate::commands::ActionContext);

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: DepositArgsContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id: near_primitives::types::AccountId =
                    scope.signer_account_id.clone().into();
                let receiver_account_id = previous_context.receiver_account_id.clone();
                let get_contract_account_id = previous_context.get_contract_account_id.clone();
                let deposit = previous_context.deposit;

                move |network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: get_contract_account_id(network_config)?,
                        actions: vec![near_primitives::transaction::Action::FunctionCall(
                            Box::new(near_primitives::transaction::FunctionCallAction {
                                method_name: "storage_deposit".to_string(),
                                args: serde_json::to_vec(&serde_json::json!({
                                    "account_id": &receiver_account_id
                                }))?,
                                gas:  near_primitives::gas::Gas::from_teragas(50),
                                deposit: deposit.into(),
                            }),
                        )],
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer_account_id: near_primitives::types::AccountId = scope.signer_account_id.clone().into();
            let receiver_account_id = previous_context.receiver_account_id.clone();

            move |outcome_view, network_config| {
                let contract_account_id = (previous_context.get_contract_account_id)(network_config)?;

                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "\n{}",
                        crate::common::indent_payload(&format!(
                            "<{signer_account_id}> has successfully added a deposit of {deposit} to <{receiver_account_id}> on contract <{contract_account_id}>.",
                            deposit = previous_context.deposit,
                        ))
                    );
                }
                Ok(())
            }
        });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![
                scope.signer_account_id.clone().into(),
                previous_context.receiver_account_id,
            ],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        item.0
    }
}

impl SignerAccountId {
    fn input_signer_account_id(
        context: &DepositArgsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the signer account ID?",
        )
    }
}
