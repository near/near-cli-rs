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
    /// Enter the hash of the transaction you need to view:
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
                    action: action_transformation(archival_action)
                }
            );
            cmd_cli_args.extend(next_actions.to_cli_args());
        }

        let skip_action = crate::commands::transaction::construct_transaction::add_action_2::CliNextAction::Skip(
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
) -> Option<crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand>{
    match archival_action {
        Action::Transfer(TransferAction { deposit }) => {
            Some(crate::commands::transaction::construct_transaction::add_action_1::add_action::CliActionSubcommand::Transfer(
                crate::commands::transaction::construct_transaction::add_action_1::add_action::transfer::CliTransferAction{
                    amount_in_near: Some(near_token::NearToken::from_yoctonear(deposit)),
                    next_action: None
                }
            ))
        }
        _ => todo!()
    }
}
