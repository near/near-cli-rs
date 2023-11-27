use color_eyre::eyre::Context;

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
        let actions: Vec<Action> = query_view_transaction_status
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
            actions: actions.clone(),
        };

        eprintln!("\nArchive transaction:\n");
        crate::common::print_unsigned_transaction(&prepopulated_transaction);
        eprintln!();

        let mut cli_args_suffix = "skip network-config testnet sign-with-legacy-keychain send"
            .split(' ')
            .map(String::from)
            .collect::<Vec<String>>();
        let mut cli_args = "near transaction construct-transaction"
            .split(' ')
            .map(String::from)
            .collect::<Vec<String>>();
        cli_args.push(signer_id.to_string());
        cli_args.push(receiver_id.to_string());

        for action in actions {
            match action {
                Action::CreateAccount(_) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("create-account".to_string());
                }
                Action::DeleteAccount(DeleteAccountAction { beneficiary_id }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("delete-account".to_string());
                    cli_args.push("--beneficiary-id".to_string());
                    cli_args.push(beneficiary_id.to_string());
                }
                Action::AddKey(AddKeyAction {
                    public_key,
                    access_key,
                }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("add-key".to_string());
                    match access_key.permission {
                        near_primitives::account::AccessKeyPermission::FullAccess => {
                            cli_args.push("grant-full-access".to_string())
                        }
                        near_primitives::account::AccessKeyPermission::FunctionCall(_) => todo!(),
                    }
                    cli_args.push("use-manually-provided-public-key".to_string());
                    cli_args.push(public_key.to_string());
                }
                Action::DeleteKey(DeleteKeyAction { public_key }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("delete-key".to_string());
                    cli_args.push(public_key.to_string());
                }
                Action::Transfer(TransferAction { deposit }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("transfer".to_string());
                    cli_args.push(format!(
                        "'{}'",
                        near_token::NearToken::from_yoctonear(deposit)
                    ));
                }
                Action::DeployContract(DeployContractAction { .. }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("deploy-contract".to_string());
                    cli_args.push("use-file".to_string());
                    cli_args.push("/near/my_project.wasm".to_string());
                    cli_args.push("without-init-call".to_string());
                }
                Action::FunctionCall(FunctionCallAction {
                    method_name,
                    args: _,
                    gas,
                    deposit,
                }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("function-call".to_string());
                    cli_args.push(method_name);
                    cli_args.push("json-args".to_string());
                    cli_args.push("'{}'".to_string());
                    cli_args.push("prepaid-gas".to_string());
                    cli_args.push(format!(
                        "'{} Tgas'",
                        near_gas::NearGas::from_gas(gas).as_tgas()
                    ));
                    cli_args.push("attached-deposit".to_string());
                    cli_args.push(format!(
                        "'{}'",
                        near_token::NearToken::from_yoctonear(deposit)
                    ));
                }
                Action::Stake(StakeAction { stake, public_key }) => {
                    cli_args.push("add-action".to_string());
                    cli_args.push("stake".to_string());
                    cli_args.push(format!(
                        "'{}'",
                        near_token::NearToken::from_yoctonear(stake)
                    ));
                    cli_args.push(public_key.to_string());
                }
                Action::Delegate(_) => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                        "SignedDelegateAction not implemented"
                    ));
                }
            }
        }
        cli_args.append(&mut cli_args_suffix);

        println!("Here is your console command to run archive transaction. You can to edit it or re-run:\n");
        println!("{}\n", cli_args.join(" "));

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
