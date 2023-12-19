use color_eyre::eyre::{Context, ContextCompat};
use interactive_clap::ToCliArgs;

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct TransactionInfo {
    /// Enter the hash of the transaction you want to use as a template:
    transaction_hash: crate::types::crypto_hash::CryptoHash,
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
}

#[derive(Clone)]
pub struct TransactionInfoContext;

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionInfo as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        use super::construct_transaction::{add_action_1, skip_action, CliConstructTransaction};
        use super::{CliTransactionActions, CliTransactionCommands};

        let networks = previous_context.config.network_connection;
        let network_config = networks
            .get(&scope.network_name)
            .wrap_err("Failed to get network config!")?
            .clone();
        let query_view_transaction_status = network_config
            .json_rpc_client()
            .blocking_call(
                near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
                    transaction_info:
                        near_jsonrpc_client::methods::tx::TransactionInfo::TransactionId {
                            hash: scope.transaction_hash.into(),
                            account_id: "near".parse::<near_primitives::types::AccountId>()?,
                        },
                },
            )
            .wrap_err_with(|| {
                format!(
                    "Failed to fetch query for view transaction on network <{}>",
                    network_config.network_name
                )
            })?;

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

        eprintln!(
            "\nTransaction {}:\n",
            query_view_transaction_status.transaction.hash
        );
        crate::common::print_unsigned_transaction(&prepopulated_transaction);
        eprintln!();

        if prepopulated_transaction.actions.len() == 1 {
            if let near_primitives::transaction::Action::Delegate(signed_delegate_action) =
                &prepopulated_transaction.actions[0]
            {
                prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                    signer_id: signed_delegate_action.delegate_action.sender_id.clone(),
                    receiver_id: signed_delegate_action.delegate_action.receiver_id.clone(),
                    actions: signed_delegate_action.delegate_action.get_actions(),
                };
            }
        }

        let cmd = crate::commands::CliTopLevelCommand::Transaction(CliTransactionCommands {
            transaction_actions: Some(CliTransactionActions::ConstructTransaction(
                CliConstructTransaction {
                    sender_account_id: Some(prepopulated_transaction.signer_id.into()),
                    receiver_account_id: Some(prepopulated_transaction.receiver_id.into()),
                    next_actions: None,
                },
            )),
        });
        let mut cmd_cli_args = cmd.to_cli_args();

        for transaction_action in prepopulated_transaction.actions {
            let next_actions =
                add_action_1::CliNextAction::AddAction(add_action_1::add_action::CliAddAction {
                    action: action_transformation(transaction_action)?,
                });
            cmd_cli_args.extend(next_actions.to_cli_args());
        }

        let skip_action = add_action_1::CliNextAction::Skip(skip_action::CliSkipAction {
            network_config: Some(
                skip_action::ClapNamedArgNetworkForTransactionArgsForSkipAction::NetworkConfig(
                    crate::network_for_transaction::CliNetworkForTransactionArgs {
                        network_name: Some(scope.network_name.clone()),
                        transaction_signature_options: None,
                    },
                ),
            ),
        });
        cmd_cli_args.extend(skip_action.to_cli_args());

        let near_cli_exec_path = crate::common::get_near_exec_path();
        eprintln!("Here is your console command to run archive transaction. You can to edit it or re-run:");
        eprintln!(
            "{}\n",
            shell_words::join(std::iter::once(near_cli_exec_path).chain(cmd_cli_args))
        );

        Ok(Self)
    }
}

impl TransactionInfo {
    fn input_network_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}

fn action_transformation(
    archival_action: near_primitives::transaction::Action,
) -> color_eyre::eyre::Result<
    Option<super::construct_transaction::add_action_1::add_action::CliActionSubcommand>,
> {
    use near_primitives::transaction::{self, Action};

    use super::construct_transaction::add_action_1::add_action;

    match archival_action {
        Action::CreateAccount(_) => {
            Ok(Some(add_action::CliActionSubcommand::CreateAccount(
                add_action::create_account::CliCreateAccountAction {
                    next_action: None
                }
            )))
        }
        Action::DeleteAccount(transaction::DeleteAccountAction { beneficiary_id }) => {
            Ok(Some(add_action::CliActionSubcommand::DeleteAccount(
                add_action::delete_account::CliDeleteAccountAction {
                    beneficiary_id: Some(beneficiary_id.into()),
                    next_action: None
                }
            )))
        }
        Action::AddKey(transaction::AddKeyAction { public_key, access_key }) => {
            Ok(Some(add_action::CliActionSubcommand::AddKey(
                add_action::add_key::CliAddKeyAction {
                    permission: get_access_key_permission(public_key, access_key.permission)?
                }
            )))
        }
        Action::DeleteKey(transaction::DeleteKeyAction { public_key }) => {
            Ok(Some(add_action::CliActionSubcommand::DeleteKey(
                add_action::delete_key::CliDeleteKeyAction {
                    public_key: Some(public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Transfer(transaction::TransferAction { deposit }) => {
            Ok(Some(add_action::CliActionSubcommand::Transfer(
                add_action::transfer::CliTransferAction {
                    amount_in_near: Some(crate::types::near_token::NearToken::from_yoctonear(deposit)),
                    next_action: None
                }
            )))
        }
        Action::DeployContract(transaction::DeployContractAction { code }) => {
            std::fs::write(
                "reconstruct-transaction-deploy-code.wasm",
                code
            )
            .wrap_err("Failed to write the deploy command code to file: 'reconstruct-transaction-deploy-code.wasm' in the current folder")?;
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
        Action::FunctionCall(transaction::FunctionCallAction { method_name, args, gas, deposit }) => {
            Ok(Some(add_action::CliActionSubcommand::FunctionCall(
                add_action::call_function::CliFunctionCallAction {
                    function_name: Some(method_name),
                    function_args_type: Some(crate::commands::contract::call_function::call_function_args_type::FunctionArgsType::TextArgs),
                    function_args: Some(String::from_utf8(args)?),
                    prepaid_gas: Some(add_action::call_function::ClapNamedArgPrepaidGasForFunctionCallAction::PrepaidGas(
                        add_action::call_function::CliPrepaidGas {
                            gas: Some(near_gas::NearGas::from_gas(gas)),
                            attached_deposit: Some(add_action::call_function::ClapNamedArgDepositForPrepaidGas::AttachedDeposit(
                                add_action::call_function::CliDeposit {
                                    deposit: Some(crate::types::near_token::NearToken::from_yoctonear(deposit)),
                                    next_action: None
                                }
                            ))
                        }
                    ))
                }
            )))
        }
        Action::Stake(transaction::StakeAction { stake, public_key }) => {
            Ok(Some(add_action::CliActionSubcommand::Stake(
                add_action::stake::CliStakeAction {
                    stake_amount: Some(crate::types::near_token::NearToken::from_yoctonear(stake)),
                    public_key: Some(public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Delegate(_) => {
            panic!("Internal error: Delegate action should have been handled before calling action_transformation.");
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
                    allowance: allowance.map(crate::types::near_token::NearToken::from_yoctonear),
                    receiver_account_id: Some(receiver_id.parse()?),
                    method_names: Some(crate::types::vec_string::VecString(method_names)),
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
