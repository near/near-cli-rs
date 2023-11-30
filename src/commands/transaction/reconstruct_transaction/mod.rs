use std::str::FromStr;

use color_eyre::eyre::Context;
use interactive_clap::ToCliArgs;

use near_primitives::transaction::{
    Action, AddKeyAction, DeleteAccountAction, DeleteKeyAction, DeployContractAction,
    FunctionCallAction, StakeAction, TransferAction,
};

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

        let cmd = crate::commands::CliTopLevelCommand::Transaction(
            crate::commands::transaction::CliTransactionCommands{
                transaction_actions: Some(crate::commands::transaction::CliTransactionActions::ConstructTransaction(
                    crate::commands::transaction::construct_transaction::CliConstructTransaction {
                        sender_account_id: Some(signer_id.into()),
                        receiver_account_id: Some(receiver_id.into()),
                        next_actions: None
                    }
                ))
            }
        );
        let mut cmd_cli_args = cmd.to_cli_args();

        for archival_action in archival_actions {
            let next_actions = crate::commands::transaction::construct_transaction::add_action_1::CliNextAction::AddAction(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::CliAddAction{
                    action: action_transformation(archival_action)?
                }
            );
            cmd_cli_args.extend(next_actions.to_cli_args());
        }

        let skip_action = crate::commands::transaction::construct_transaction::add_action_1::CliNextAction::Skip(
            crate::commands::transaction::construct_transaction::skip_action::CliSkipAction{
                network_config: Some(crate::commands::transaction::construct_transaction::skip_action::ClapNamedArgNetworkForTransactionArgsForSkipAction::NetworkConfig(
                    crate::network_for_transaction::CliNetworkForTransactionArgs{
                    network_name: Some(scope.network_name.clone()),
                    transaction_signature_options: None
                }
                ))
            }
        );
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
) -> color_eyre::eyre::Result<Option<crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand>>{
    match archival_action {
        Action::CreateAccount(_) => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::CreateAccount(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::create_account::CliCreateAccountAction{
                    next_action: None
                }
            )))
        }
        Action::DeleteAccount(DeleteAccountAction { beneficiary_id }) => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::DeleteAccount(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::delete_account::CliDeleteAccountAction{
                    beneficiary_id: Some(beneficiary_id.into()),
                    next_action: None
                }
            )))
        }
        Action::AddKey(AddKeyAction {
            public_key,
            access_key,
        }) => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::AddKey(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::CliAddKeyAction{
                    permission: get_access_key_permission(public_key, access_key.permission)?
                }
            )))
        }
        Action::DeleteKey(DeleteKeyAction { public_key }) => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::DeleteKey(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::delete_key::CliDeleteKeyAction{
                    public_key: Some(public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Transfer(TransferAction { deposit }) => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::Transfer(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::transfer::CliTransferAction{
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
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::DeployContract(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::deploy_contract::CliDeployContractAction{
                    use_file: Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::deploy_contract::ClapNamedArgContractFileForDeployContractAction::UseFile(
                        crate::commands::transaction::construct_transaction::add_action_1::add_action::deploy_contract::CliContractFile{
                            file_path: Some(crate::types::path_buf::PathBuf::from_str("./near-contract/my-contract.wasm")?),
                            initialize: Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                                crate::commands::transaction::construct_transaction::add_action_1::add_action::deploy_contract::initialize_mode::CliNoInitialize{
                                    next_action: None
                                }
                            ))
                        }
                    )),
                }
            )))
        }
        Action::FunctionCall(FunctionCallAction { method_name, args, gas, deposit }) => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::FunctionCall(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::call_function::CliFunctionCallAction{
                    function_name: Some(method_name),
                    function_args_type: Some(crate::commands::contract::call_function::call_function_args_type::FunctionArgsType::TextArgs),
                    function_args: Some(String::from_utf8(args)?),
                    prepaid_gas: Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::call_function::ClapNamedArgPrepaidGasForFunctionCallAction::PrepaidGas(
                        crate::commands::transaction::construct_transaction::add_action_1::add_action::call_function::CliPrepaidGas{
                            gas: Some(near_gas::NearGas::from_gas(gas)),
                            attached_deposit: Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::call_function::ClapNamedArgDepositForPrepaidGas::AttachedDeposit(
                                crate::commands::transaction::construct_transaction::add_action_1::add_action::call_function::CliDeposit{
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
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::Stake(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::stake::CliStakeAction{
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
) -> color_eyre::eyre::Result<Option<crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyPermission>>{
    match access_key_permission {
        near_primitives::account::AccessKeyPermission::FullAccess => {
            Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyPermission::GrantFullAccess(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::access_key_type::CliFullAccessType{
                    access_key_mode: Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyMode::UseManuallyProvidedPublicKey(
                        crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::use_public_key::CliAddAccessKeyAction{
                            public_key: Some(public_key.into()),
                            next_action: None
                        }
                    ))
                }
            )))
        }
        near_primitives::account::AccessKeyPermission::FunctionCall(
            near_primitives::account::FunctionCallPermission{allowance, receiver_id, method_names}
        ) => Ok(Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyPermission::GrantFunctionCallAccess(
            crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::access_key_type::CliFunctionCallType{
                allowance: Some(near_token::NearToken::from_yoctonear(allowance.expect("Internal error"))),
                receiver_account_id: Some(crate::types::account_id::AccountId::from_str(&receiver_id)?),
                method_names: Some( crate::types::vec_string::VecString(method_names)),
                access_key_mode: Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyMode::UseManuallyProvidedPublicKey(
                    crate::commands::transaction::construct_transaction::add_action_1::add_action::add_key::use_public_key::CliAddAccessKeyAction{
                        public_key: Some(public_key.into()),
                        next_action: None
                    }
                ))
            }
        )))
    }
}
