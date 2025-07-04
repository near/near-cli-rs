use std::str::FromStr;

use color_eyre::eyre::{Context, ContextCompat};
use interactive_clap::ToCliArgs;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct TransactionInfo {
    /// Enter the hash of the transaction you want to use as a template:
    transaction_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct TransactionInfoContext(crate::network::NetworkContext);

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionInfo as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        use super::construct_transaction::{add_action_1, skip_action, CliConstructTransaction};
        use super::{CliTransactionActions, CliTransactionCommands};

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let tx_hash: near_primitives::hash::CryptoHash = scope.transaction_hash.into();

                move |network_config: &crate::config::NetworkConfig| {
                    let query_view_transaction_status = super::view_status::get_transaction_info(network_config, tx_hash)?
                        .final_execution_outcome
                        .wrap_err_with(|| {
                            format!(
                                "Failed to get the final execution outcome for the transaction {tx_hash}"
                            )
                        })?
                        .into_outcome();

                    let mut prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                        signer_id: query_view_transaction_status.transaction.signer_id,
                        receiver_id: query_view_transaction_status.transaction.receiver_id,
                        actions: query_view_transaction_status
                            .transaction
                            .actions
                            .into_iter()
                            .map(near_primitives::transaction::Action::try_from)
                            .collect::<Result<Vec<near_primitives::transaction::Action>, _>>()
                            .expect("Internal error: can not convert the action_view to action."),
                    };

                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "Transaction {}:{}",
                        query_view_transaction_status.transaction.hash,
                        crate::common::indent_payload(&crate::common::print_unsigned_transaction(
                            &prepopulated_transaction,
                        ))
                    );

                    if prepopulated_transaction.actions.len() == 1 {
                        if let near_primitives::transaction::Action::Delegate(
                            signed_delegate_action,
                        ) = &prepopulated_transaction.actions[0]
                        {
                            prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                                signer_id: signed_delegate_action.delegate_action.sender_id.clone(),
                                receiver_id: signed_delegate_action
                                    .delegate_action
                                    .receiver_id
                                    .clone(),
                                actions: signed_delegate_action.delegate_action.get_actions(),
                            };
                        }
                    }

                    let cmd =
                        crate::commands::CliTopLevelCommand::Transaction(CliTransactionCommands {
                            transaction_actions: Some(CliTransactionActions::ConstructTransaction(
                                CliConstructTransaction {
                                    sender_account_id: Some(
                                        prepopulated_transaction.signer_id.into(),
                                    ),
                                    receiver_account_id: Some(
                                        prepopulated_transaction.receiver_id.clone().into(),
                                    ),
                                    next_actions: None,
                                },
                            )),
                        });
                    let mut cmd_cli_args = cmd.to_cli_args();

                    for transaction_action in prepopulated_transaction.actions {
                        let next_actions = add_action_1::CliNextAction::AddAction(
                            add_action_1::add_action::CliAddAction {
                                action: action_transformation(
                                    transaction_action,
                                    prepopulated_transaction.receiver_id.clone(),
                                    network_config,
                                    near_primitives::types::BlockReference::BlockId(
                                        near_primitives::types::BlockId::Hash(
                                            query_view_transaction_status
                                                .transaction_outcome
                                                .block_hash,
                                        ),
                                    ),
                                )?,
                            },
                        );
                        cmd_cli_args.extend(next_actions.to_cli_args());
                    }

                    let skip_action = add_action_1::CliNextAction::Skip(skip_action::CliSkipAction {
                        network_config: Some(
                            skip_action::ClapNamedArgNetworkForTransactionArgsForSkipAction::NetworkConfig(
                                crate::network_for_transaction::CliNetworkForTransactionArgs {
                                    network_name: Some(network_config.network_name.clone()),
                                    transaction_signature_options: None,
                                },
                            ),
                        ),
                    });
                    cmd_cli_args.extend(skip_action.to_cli_args());

                    let near_cli_exec_path = crate::common::get_near_exec_path();
                    if let crate::Verbosity::Quiet = previous_context.verbosity {
                        println!(
                            "{}",
                            shell_words::join(
                                std::iter::once(near_cli_exec_path).chain(cmd_cli_args)
                            )
                        );
                    } else {
                        tracing::info!(
                            parent: &tracing::Span::none(),
                            "Here is your console command to run archive transaction. You can to edit it or re-run:\n{}",
                            crate::common::indent_payload(&shell_words::join(
                                std::iter::once(near_cli_exec_path).chain(cmd_cli_args)
                            ))
                        );
                    }
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![],
            on_after_getting_network_callback,
        }))
    }
}

impl From<TransactionInfoContext> for crate::network::NetworkContext {
    fn from(item: TransactionInfoContext) -> Self {
        item.0
    }
}

fn action_transformation(
    archival_action: near_primitives::transaction::Action,
    receiver_id: near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<
    Option<super::construct_transaction::add_action_1::add_action::CliActionSubcommand>,
> {
    use near_primitives::transaction::Action;

    use super::construct_transaction::add_action_1::add_action;

    match archival_action {
        Action::CreateAccount(_) => {
            Ok(Some(add_action::CliActionSubcommand::CreateAccount(
                add_action::create_account::CliCreateAccountAction {
                    next_action: None
                }
            )))
        }
        Action::DeleteAccount(delete_account_action) => {
            Ok(Some(add_action::CliActionSubcommand::DeleteAccount(
                add_action::delete_account::CliDeleteAccountAction {
                    beneficiary_id: Some(delete_account_action.beneficiary_id.into()),
                    next_action: None
                }
            )))
        }
        Action::AddKey(add_key_action) => {
            Ok(Some(add_action::CliActionSubcommand::AddKey(
                add_action::add_key::CliAddKeyAction {
                    permission: get_access_key_permission(add_key_action.public_key, add_key_action.access_key.permission)?
                }
            )))
        }
        Action::DeleteKey(delete_key_action) => {
            Ok(Some(add_action::CliActionSubcommand::DeleteKey(
                add_action::delete_key::CliDeleteKeyAction {
                    public_key: Some(delete_key_action.public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Transfer(transfer_action) => {
            Ok(Some(add_action::CliActionSubcommand::Transfer(
                add_action::transfer::CliTransferAction {
                    amount_in_near: Some(crate::types::near_token::NearToken::from_yoctonear(transfer_action.deposit)),
                    next_action: None
                }
            )))
        }
        Action::DeployContract(deploy_contract_action) => {
            download_code(
                &receiver_id,
                network_config,
                block_reference,
                "reconstruct-transaction-deploy-code.wasm",
                &deploy_contract_action.code
            )?;
            Ok(Some(add_action::CliActionSubcommand::DeployContract(
                add_action::deploy_contract::CliDeployContractAction {
                    use_file: Some(add_action::deploy_contract::ClapNamedArgContractFileForDeployContractAction::UseFile(
                        add_action::deploy_contract::CliContractFile {
                            file_path: Some("reconstruct-transaction-deploy-code.wasm".parse()?),
                            initialize: Some(add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                                add_action::deploy_contract::initialize_mode::CliNoInitialize {
                                    next_action: None
                                }
                            ))
                        }
                    )),
                }
            )))
        }
        Action::FunctionCall(function_call_action) => {
            Ok(Some(add_action::CliActionSubcommand::FunctionCall(
                add_action::call_function::CliFunctionCallAction {
                    function_name: Some(function_call_action.method_name),
                    function_args_type: Some(crate::commands::contract::call_function::call_function_args_type::FunctionArgsType::TextArgs),
                    function_args: Some(String::from_utf8(function_call_action.args)?),
                    prepaid_gas: Some(add_action::call_function::ClapNamedArgPrepaidGasForFunctionCallAction::PrepaidGas(
                        add_action::call_function::CliPrepaidGas {
                            gas: Some(near_gas::NearGas::from_gas(function_call_action.gas)),
                            attached_deposit: Some(add_action::call_function::ClapNamedArgDepositForPrepaidGas::AttachedDeposit(
                                add_action::call_function::CliDeposit {
                                    deposit: Some(crate::types::near_token::NearToken::from_yoctonear(function_call_action.deposit)),
                                    next_action: None
                                }
                            ))
                        }
                    ))
                }
            )))
        }
        Action::Stake(stake_action) => {
                Ok(Some(add_action::CliActionSubcommand::Stake(
                add_action::stake::CliStakeAction {
                    stake_amount: Some(crate::types::near_token::NearToken::from_yoctonear(stake_action.stake)),
                    public_key: Some(stake_action.public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Delegate(_) => {
            panic!("Internal error: Delegate action should have been handled before calling action_transformation.");
        }
        Action::DeployGlobalContract(_) => {
            Err(color_eyre::eyre::eyre!("Reconstruction of Global Deploy transactions is not supported yet. This feature is being tracked at: https://github.com/near/nearcore/issues/13531"))
        }
        Action::UseGlobalContract(use_global_contract_action) => {
            let mode = match use_global_contract_action.contract_identifier {
                near_primitives::action::GlobalContractIdentifier::CodeHash(hash) => add_action::use_global_contract::CliUseGlobalActionMode::UseGlobalHash(
                    add_action::use_global_contract::CliUseHashAction {
                        hash: Some(crate::types::crypto_hash::CryptoHash(hash)),
                        initialize: Some(add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                            add_action::deploy_contract::initialize_mode::CliNoInitialize {
                                next_action: None
                            }
                        ))
                    }
                ),
                near_primitives::action::GlobalContractIdentifier::AccountId(account_id) => add_action::use_global_contract::CliUseGlobalActionMode::UseGlobalAccountId(
                    add_action::use_global_contract::CliUseAccountIdAction {
                        account_id: Some(crate::types::account_id::AccountId(account_id)),
                        initialize: Some(add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                            add_action::deploy_contract::initialize_mode::CliNoInitialize {
                                next_action: None
                            }
                        ))
                    }
                ),
            };

            Ok(Some(add_action::CliActionSubcommand::UseGlobalContract(
                add_action::use_global_contract::CliUseGlobalContractAction {
                    mode: Some(mode)
                }
            )))
        }
    }
}

fn get_access_key_permission(
    public_key: near_crypto::PublicKey,
    access_key_permission: near_primitives::account::AccessKeyPermission,
) -> color_eyre::eyre::Result<
    Option<super::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyPermission>,
> {
    use super::construct_transaction::add_action_1::add_action::add_key;

    match access_key_permission {
        near_primitives::account::AccessKeyPermission::FullAccess => {
            Ok(Some(add_key::CliAccessKeyPermission::GrantFullAccess(
                add_key::access_key_type::CliFullAccessType {
                    access_key_mode: Some(add_key::CliAccessKeyMode::UseManuallyProvidedPublicKey(
                        add_key::use_public_key::CliAddAccessKeyAction {
                            public_key: Some(public_key.into()),
                            next_action: None,
                        },
                    )),
                },
            )))
        }
        near_primitives::account::AccessKeyPermission::FunctionCall(
            near_primitives::account::FunctionCallPermission {
                allowance,
                receiver_id,
                method_names,
            },
        ) => Ok(Some(
            add_key::CliAccessKeyPermission::GrantFunctionCallAccess(
                add_key::access_key_type::CliFunctionCallType {
                    allowance: {
                        match allowance {
                            Some(yoctonear) => {
                                Some(crate::types::near_allowance::NearAllowance::from_yoctonear(
                                    yoctonear,
                                ))
                            }
                            None => Some(crate::types::near_allowance::NearAllowance::from_str(
                                "unlimited",
                            )?),
                        }
                    },
                    contract_account_id: Some(receiver_id.parse()?),
                    function_names: Some(crate::types::vec_string::VecString(method_names)),
                    access_key_mode: Some(add_key::CliAccessKeyMode::UseManuallyProvidedPublicKey(
                        add_key::use_public_key::CliAddAccessKeyAction {
                            public_key: Some(public_key.into()),
                            next_action: None,
                        },
                    )),
                },
            ),
        )),
    }
}

fn download_code(
    receiver_id: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
    file_name: &str,
    hash_to_match: &[u8],
) -> color_eyre::eyre::Result<()> {
    // Unfortunately, RPC doesn't return the code for the deployed contract. Only the hash.
    // So we need to fetch it from archive node.

    let code = crate::commands::contract::download_wasm::get_code(
                receiver_id,
                network_config,
                block_reference
            ).map_err(|e| {
                color_eyre::Report::msg(format!("Couldn't fetch the code. Please verify that you are using the archival node in the `network_connection.*.rpc_url` field of the `config.toml` file. You can see the list of RPC providers at https://docs.near.org/api/rpc/providers.\nError: {e}"))
            })?;

    let code_hash = near_primitives::hash::CryptoHash::hash_bytes(&code);
    tracing::info!(
        parent: &tracing::Span::none(),
        "The code for the account <{}> was downloaded successfully with hash <{}>",
        receiver_id,
        code_hash,
    );
    if code_hash.0 != hash_to_match {
        return Err(color_eyre::Report::msg("The code hash of the contract deploy action does not match the code that we retrieved from the archive node.".to_string()));
    }

    std::fs::write(file_name, code).wrap_err(format!(
        "Failed to write the deploy command code to file: '{file_name}' in the current folder"
    ))?;

    tracing::info!(
        parent: &tracing::Span::none(),
        "The file `{}` with contract code of `{}` was downloaded successfully",
        file_name,
        receiver_id,
    );

    Ok(())
}
