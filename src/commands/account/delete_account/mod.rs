use color_eyre::owo_colors::OwoColorize;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DeleteAccountContext)]
pub struct DeleteAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID to be deleted?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Enter the beneficiary ID to delete this account ID
    beneficiary: BeneficiaryAccount,
}

#[derive(Debug, Clone)]
pub struct DeleteAccountContext {
    pub global_context: crate::GlobalContext,
    pub account_id: near_primitives::types::AccountId,
}

impl DeleteAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DeleteAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl DeleteAccount {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What Account ID to be deleted?",
        )
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DeleteAccountContext)]
#[interactive_clap(output_context = BeneficiaryAccountContext)]
pub struct BeneficiaryAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// Specify a beneficiary:
    beneficiary_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct BeneficiaryAccountContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
    beneficiary_account_id: near_primitives::types::AccountId,
}

impl BeneficiaryAccountContext {
    pub fn from_previous_context(
        previous_context: DeleteAccountContext,
        scope: &<BeneficiaryAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let beneficiary_account_id: near_primitives::types::AccountId =
            scope.beneficiary_account_id.clone().into();

        if previous_context.account_id == beneficiary_account_id {
            return Err(color_eyre::eyre::eyre!(
                "Invalid beneficiary account ID.\nThe beneficiary account ID cannot be the same as the account ID being deleted."
            ));
        }

        Ok(Self {
            global_context: previous_context.global_context,
            account_id: previous_context.account_id,
            beneficiary_account_id,
        })
    }
}

impl From<BeneficiaryAccountContext> for crate::commands::ActionContext {
    fn from(item: BeneficiaryAccountContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let account_id = item.account_id.clone();
                let beneficiary_account_id = item.beneficiary_account_id.clone();

                move |network_config| {
                    validate_beneficiary_in_network(network_config, &beneficiary_account_id, item.global_context.offline)?;
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: account_id.clone(),
                        receiver_id: account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::DeleteAccount(
                            near_primitives::transaction::DeleteAccountAction {
                                beneficiary_id: beneficiary_account_id.clone(),
                            },
                        )],
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
            sign_as_delegate_action: false,
            on_sending_delegate_action_callback: None,
        }
    }
}

impl BeneficiaryAccount {
    pub fn input_beneficiary_account_id(
        context: &DeleteAccountContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        loop {
            let beneficiary_account_id = if let Some(account_id) =
                crate::common::input_non_signer_account_id_from_used_account_list(
                    &context.global_context.config.credentials_home_dir,
                    "What is the beneficiary account ID?",
                )? {
                account_id
            } else {
                return Ok(None);
            };

            if beneficiary_account_id.0 == context.account_id {
                tracing::warn!("{}", "You have selected a beneficiary account ID that will now be deleted. This will result in the loss of your funds. So make your choice again.".red());
                continue;
            }
            return Ok(Some(beneficiary_account_id));
        }
    }
}

pub fn validate_beneficiary_in_network(
    network_config: &crate::config::NetworkConfig,
    beneficiary_account_id: &near_primitives::types::AccountId,
    offline_mode: bool,
) -> crate::CliResult {
    if offline_mode {
        tracing::warn!(
            target: "near_teach_me",
            "{}{}",
            format!(
                "Skipping verification of account <{}> as a beneficiary in offline mode.",
                beneficiary_account_id,
            ).red(),
            crate::common::indent_payload(&format!("\n{}",
                "Make sure you specify an existing account as a beneficiary to avoid losing your funds.\nIt is currently possible to continue deleting an account offline.\nYou can sign and send the created transaction later.\n "
                .yellow()
            ))
        );
        return Ok(());
    }

    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::common::get_account_state(
            network_config,
            beneficiary_account_id,
            near_primitives::types::BlockReference::latest(),
        )) {
        Ok(_) => Ok(()),
        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount { .. },
            ),
        )) => Err(color_eyre::eyre::eyre!(
            "Account <{}> does not exist on the {}. Please specify an existing account as a beneficiary to avoid losing your funds.",
            &beneficiary_account_id,
            &network_config.network_name
        )),
        Err(err) => Err(err.into()),
    }
}
