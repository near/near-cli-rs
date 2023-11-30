use std::str::FromStr;

use color_eyre::eyre::Context;
use interactive_clap::ToCliArgs;

use near_primitives::{
    account::{AccessKeyPermission, FunctionCallPermission},
    transaction::{
        Action, AddKeyAction, DeleteAccountAction, DeleteKeyAction, DeployContractAction,
        FunctionCallAction, StakeAction, TransferAction,
    },
};

use crate::common::JsonRpcClientExt;

use crate::commands::transaction::{
    construct_transaction::{
        add_action_1::{
            add_action::{
                add_key::CliAddKeyAction,
                add_key::{
                    access_key_type::{CliFullAccessType, CliFunctionCallType},
                    use_public_key::CliAddAccessKeyAction,
                    CliAccessKeyMode, CliAccessKeyPermission,
                },
                call_function::{
                    ClapNamedArgDepositForPrepaidGas, ClapNamedArgPrepaidGasForFunctionCallAction,
                    CliDeposit, CliFunctionCallAction, CliPrepaidGas,
                },
                create_account::CliCreateAccountAction,
                delete_account::CliDeleteAccountAction,
                delete_key::CliDeleteKeyAction,
                deploy_contract::{
                    initialize_mode::{CliInitializeMode, CliNoInitialize},
                    ClapNamedArgContractFileForDeployContractAction, CliContractFile,
                    CliDeployContractAction,
                },
                stake::CliStakeAction,
                transfer::CliTransferAction,
                CliActionSubcommand, CliAddAction,
            },
            CliNextAction,
        },
        skip_action::{ClapNamedArgNetworkForTransactionArgsForSkipAction, CliSkipAction},
        CliConstructTransaction,
    },
    CliTransactionActions, CliTransactionCommands,
};

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
        let networks = previous_context.config.network_connection;
        let network_config = networks
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        let query_view_transaction_status = network_config
            .json_rpc_client()
            .blocking_call(near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::RpcTransactionStatusRequest {
                transaction_info: near_jsonrpc_client::methods::EXPERIMENTAL_tx_status::TransactionInfo::TransactionId {
                    hash: scope.transaction_hash.into(),
                    account_id: "near".parse::<near_primitives::types::AccountId>()?

                }
            })
            .wrap_err("Failed to fetch query for view transaction")?;

        let signer_id = query_view_transaction_status
            .final_outcome
            .transaction
            .signer_id;
        let receiver_id = query_view_transaction_status
            .final_outcome
            .transaction
            .receiver_id;
        let archival_actions: Vec<Action> = query_view_transaction_status
            .final_outcome
            .transaction
            .actions
            .into_iter()
            .map(near_primitives::transaction::Action::try_from)
            .collect::<Result<_, _>>()
            .expect("Internal error: can not convert the action_view to action.");
        let prepopulated_transaction = crate::commands::PrepopulatedTransaction {
            signer_id: signer_id.clone(),
            receiver_id: receiver_id.clone(),
            actions: archival_actions.clone(),
        };

        eprintln!("\nArchive transaction:\n");
        crate::common::print_unsigned_transaction(&prepopulated_transaction);
        eprintln!();

        let cmd = crate::commands::CliTopLevelCommand::Transaction(CliTransactionCommands {
            transaction_actions: Some(CliTransactionActions::ConstructTransaction(
                CliConstructTransaction {
                    sender_account_id: Some(signer_id.into()),
                    receiver_account_id: Some(receiver_id.into()),
                    next_actions: None,
                },
            )),
        });
        let mut cmd_cli_args = cmd.to_cli_args();

        for archival_action in archival_actions {
            let next_actions = CliNextAction::AddAction(CliAddAction {
                action: action_transformation(archival_action)?,
            });
            cmd_cli_args.extend(next_actions.to_cli_args());
        }

        let skip_action = CliNextAction::Skip(CliSkipAction {
            network_config: Some(
                ClapNamedArgNetworkForTransactionArgsForSkipAction::NetworkConfig(
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
    archival_action: Action,
) -> color_eyre::eyre::Result<Option<CliActionSubcommand>> {
    match archival_action {
        Action::CreateAccount(_) => {
            Ok(Some(CliActionSubcommand::CreateAccount(
                CliCreateAccountAction{
                    next_action: None
                }
            )))
        }
        Action::DeleteAccount(DeleteAccountAction { beneficiary_id }) => {
            Ok(Some(CliActionSubcommand::DeleteAccount(
                CliDeleteAccountAction{
                    beneficiary_id: Some(beneficiary_id.into()),
                    next_action: None
                }
            )))
        }
        Action::AddKey(AddKeyAction {
            public_key,
            access_key,
        }) => {
            Ok(Some(CliActionSubcommand::AddKey(
                CliAddKeyAction{
                    permission: get_access_key_permission(public_key, access_key.permission)?
                }
            )))
        }
        Action::DeleteKey(DeleteKeyAction { public_key }) => {
            Ok(Some(CliActionSubcommand::DeleteKey(
                CliDeleteKeyAction{
                    public_key: Some(public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Transfer(TransferAction { deposit }) => {
            Ok(Some(CliActionSubcommand::Transfer(
                CliTransferAction{
                    amount_in_near: Some(near_token::NearToken::from_yoctonear(deposit)),
                    next_action: None
                }
            )))
        }
        Action::DeployContract(DeployContractAction { code }) => {
            std::fs::create_dir_all("near-contract")?;
            std::fs::write(
                "./near-contract/my-contract.wasm",
                code
            )
            .wrap_err("Failed to write to file: '/near-contract/my-contract.wasm'")?;
            Ok(Some(CliActionSubcommand::DeployContract(
                CliDeployContractAction{
                    use_file: Some(ClapNamedArgContractFileForDeployContractAction::UseFile(
                        CliContractFile{
                            file_path: Some(crate::types::path_buf::PathBuf::from_str("./near-contract/my-contract.wasm")?),
                            initialize: Some(CliInitializeMode::WithoutInitCall(
                                CliNoInitialize{
                                    next_action: None
                                }
                            ))
                        }
                    )),
                }
            )))
        }
        Action::FunctionCall(FunctionCallAction { method_name, args, gas, deposit }) => {
            Ok(Some(CliActionSubcommand::FunctionCall(
                CliFunctionCallAction{
                    function_name: Some(method_name),
                    function_args_type: Some(crate::commands::contract::call_function::call_function_args_type::FunctionArgsType::TextArgs),
                    function_args: Some(String::from_utf8(args)?),
                    prepaid_gas: Some(ClapNamedArgPrepaidGasForFunctionCallAction::PrepaidGas(
                        CliPrepaidGas{
                            gas: Some(near_gas::NearGas::from_gas(gas)),
                            attached_deposit: Some(ClapNamedArgDepositForPrepaidGas::AttachedDeposit(
                                CliDeposit{
                                    deposit: Some(near_token::NearToken::from_yoctonear(deposit)),
                                    next_action: None
                                }
                            ))
                        }
                    ))
                }
            )))
        }
        Action::Stake(StakeAction { stake, public_key }) => {
            Ok(Some(CliActionSubcommand::Stake(
                CliStakeAction{
                    stake_amount: Some(near_token::NearToken::from_yoctonear(stake)),
                    public_key: Some(public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Delegate(_) => {
            color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "SignedDelegateAction not implemented"
            ))
        }
    }
}

fn get_access_key_permission(
    public_key: near_crypto::PublicKey,
    access_key_permission: near_primitives::account::AccessKeyPermission,
) -> color_eyre::eyre::Result<Option<CliAccessKeyPermission>> {
    match access_key_permission {
        AccessKeyPermission::FullAccess => Ok(Some(CliAccessKeyPermission::GrantFullAccess(
            CliFullAccessType {
                access_key_mode: Some(CliAccessKeyMode::UseManuallyProvidedPublicKey(
                    CliAddAccessKeyAction {
                        public_key: Some(public_key.into()),
                        next_action: None,
                    },
                )),
            },
        ))),
        AccessKeyPermission::FunctionCall(FunctionCallPermission {
            allowance,
            receiver_id,
            method_names,
        }) => Ok(Some(CliAccessKeyPermission::GrantFunctionCallAccess(
            CliFunctionCallType {
                allowance: Some(near_token::NearToken::from_yoctonear(
                    allowance.expect("Internal error"),
                )),
                receiver_account_id: Some(crate::types::account_id::AccountId::from_str(
                    &receiver_id,
                )?),
                method_names: Some(crate::types::vec_string::VecString(method_names)),
                access_key_mode: Some(CliAccessKeyMode::UseManuallyProvidedPublicKey(
                    CliAddAccessKeyAction {
                        public_key: Some(public_key.into()),
                        next_action: None,
                    },
                )),
            },
        ))),
    }
}
