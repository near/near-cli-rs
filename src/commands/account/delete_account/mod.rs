use color_eyre::owo_colors::OwoColorize;
use inquire::Select;

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
        let beneficiary_account_id = validate_beneficiary(
            &previous_context,
            scope.beneficiary_account_id.clone().into(),
        )?;
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

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: account_id.clone(),
                        receiver_id: account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::DeleteAccount(
                            near_primitives::transaction::DeleteAccountAction {
                                beneficiary_id: item.beneficiary_account_id.clone(),
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

            if context.global_context.offline {
                tracing::warn!("{}{}",
                    format!(
                        "Skipping verification of account <{}> as a beneficiary in offline mode.",
                        &beneficiary_account_id,
                    ).red(),
                    crate::common::indent_payload(&format!("\n{}",
                        "Make sure you specify an existing account as a beneficiary to avoid losing your funds.\nIt is currently possible to continue deleting an account offline.\nYou can sign and send the created transaction later.\n "
                        .yellow()
                    ))
                );
                if crate::common::ask_if_different_account_id_wanted()? {
                    continue;
                } else {
                    return Ok(Some(beneficiary_account_id));
                }
            }

            #[derive(derive_more::Display)]
            enum ConfirmOptions {
                #[display(
                    "Yes, I want to check if account <{account_id}> exists. (It is free of charge, and only requires Internet access)"
                )]
                Yes {
                    account_id: crate::types::account_id::AccountId,
                },
                #[display("No, I know this account exists and want to continue.")]
                No,
            }
            let select_choose_input =
                Select::new("Do you want to check the existence of the specified account so that you don't lose tokens?",
                    vec![ConfirmOptions::Yes{account_id: beneficiary_account_id.clone()}, ConfirmOptions::No],
                    )
                    .prompt()?;
            if let ConfirmOptions::Yes { account_id } = select_choose_input {
                let network_where_beneficiary_account_exist =
                    match crate::common::find_network_where_account_exist(
                        &context.global_context,
                        account_id.clone().into(),
                    ) {
                        Ok(network_config) => network_config,
                        Err(err) => {
                            tracing::warn!("{}{}", 
                                "Cannot verify beneficiary. Proceeding may result in total loss of NEAR tokens of the deleting account.".red(),
                                crate::common::indent_payload(&format!("\n{}{}",
                                    format!("{err}").red(),
                                    "\nIt is currently possible to continue deleting an account offline.\nYou can sign and send the created transaction later.\n "
                                    .yellow()
                                ))
                            );
                            return Ok(Some(account_id));
                        }
                    };

                if let Some(network_config) = network_where_beneficiary_account_exist {
                    if crate::common::is_receiver_on_wrong_network(
                        network_config.linkdrop_account_id.as_ref(),
                        &context.account_id,
                    ) {
                        tracing::warn!("{}", format!(
                                "Heads up! You will lose remaining NEAR tokens on the account you delete if you specify the account <{}> as the beneficiary as it does not exist on the same network as the account you are deleting.",
                                &beneficiary_account_id,
                            ).red());
                        continue;
                    } else {
                        return Ok(Some(beneficiary_account_id));
                    }
                } else {
                    tracing::warn!("{}", format!("The account <{}> does not exist on [{}] networks. So, you can delete this account without specifying an existing account as a beneficiary, but you will lose your funds.",
                        &beneficiary_account_id,
                        context.global_context.config.network_names().join(", ")).red()
                    );
                    continue;
                }
            } else {
                return Ok(Some(beneficiary_account_id));
            };
        }
    }
}

pub fn validate_beneficiary(
    context: &DeleteAccountContext,
    beneficiary_account_id: near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
    if context.account_id == beneficiary_account_id {
        return Err(color_eyre::eyre::eyre!(
            "Invalid beneficiary account ID.\nThe beneficiary account ID cannot be the same as the account ID being deleted."
        ));
    }

    if context.global_context.offline {
        tracing::warn!(
                target: "near_teach_me",
                "{}", format!(
                "Skipping verification of account <{}> as a beneficiary in offline mode. Make sure you specify an existing account as a beneficiary to avoid losing your funds.",
                &beneficiary_account_id,
            ).red());
        return Ok(beneficiary_account_id);
    }

    let network_where_account_exist = crate::common::find_network_where_account_exist(
        &context.global_context,
        beneficiary_account_id.clone(),
    )?;

    if let Some(network_config) = network_where_account_exist {
        if crate::common::is_receiver_on_wrong_network(
            network_config.linkdrop_account_id.as_ref(),
            &context.account_id,
        ) {
            Err(color_eyre::eyre::eyre!(
                "Invalid beneficiary account ID.\nThe beneficiary account ID cannot be on a different network than the account being deleted."
            ))
        } else {
            Ok(beneficiary_account_id)
        }
    } else {
        Err(color_eyre::eyre::eyre!(
            "Account <{}> does not exist on any of the networks. Please specify an existing account as a beneficiary to avoid losing your funds.",
            &beneficiary_account_id
        ))
    }
}
