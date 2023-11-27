use color_eyre::eyre::{Context, ContextCompat};
use inquire::{CustomType, Select};

use near_primitives::transaction::{Action, DeployContractAction, TransferAction};

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
    #[interactive_clap(subcommand)]
    transaction_signature_options: crate::transaction_signature_options::SignWith,
}

#[derive(Clone)]
pub struct TransactionInfoContext {
    global_context: crate::GlobalContext,
    network_config: crate::config::NetworkConfig,
    prepopulated_transaction: crate::commands::PrepopulatedTransaction,
}

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionInfo as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let networks = previous_context.config.network_connection.clone();
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
            signer_id,
            receiver_id,
            actions: actions.clone(),
        };

        eprintln!("\nArchive transaction:\n");
        crate::common::print_unsigned_transaction(&prepopulated_transaction);
        eprintln!();

        if !need_edit_archive_transaction() {
            return Ok(Self {
                global_context: previous_context,
                network_config,
                prepopulated_transaction,
            });
        }

        let updated_receiver_id =
            crate::common::input_non_signer_account_id_from_used_account_list(
                &previous_context.config.credentials_home_dir,
                "Enter receiver account ID:",
            )?
            .wrap_err("Internal error: can not to get the receiver account ID.")?;

        let updated_signer_id = crate::common::input_signer_account_id_from_used_account_list(
            &previous_context.config.credentials_home_dir,
            "Enter signer account ID:",
        )?
        .wrap_err("Internal error: can not to get the signer account ID.")?;

        let mut updated_actions: Vec<Action> = vec![];
        for action in actions {
            match action {
                Action::CreateAccount(create_account_action) => {
                    updated_actions.push(Action::CreateAccount(create_account_action))
                }
                Action::DeployContract(_) => {
                    let file_path: crate::types::path_buf::PathBuf =
                        CustomType::new("What is a file location of the contract?").prompt()?;
                    let code = std::fs::read(&file_path.0).wrap_err_with(|| {
                        format!("Failed to open or read the file: {:?}.", &file_path.0,)
                    })?;
                    updated_actions.push(Action::DeployContract(DeployContractAction { code }))
                }
                Action::Transfer(_) => {
                    let deposit:near_token::NearToken  = CustomType::new("How many NEAR Tokens do you want to transfer? (example: 10NEAR or 0.5near or 10000yoctonear)").prompt()?;
                    updated_actions.push(Action::Transfer(TransferAction {
                        deposit: deposit.as_yoctonear(),
                    }))
                }
                _ => todo!(),
            }
        }

        Ok(Self {
            global_context: previous_context,
            network_config,
            prepopulated_transaction: crate::commands::PrepopulatedTransaction {
                receiver_id: updated_receiver_id.into(),
                signer_id: updated_signer_id.into(),
                actions: updated_actions,
            },
        })
    }
}

impl From<TransactionInfoContext> for crate::commands::TransactionContext {
    fn from(item: TransactionInfoContext) -> Self {
        Self {
            global_context: item.global_context,
            network_config: item.network_config,
            prepopulated_transaction: item.prepopulated_transaction,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl TransactionInfo {
    fn input_network_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}

fn need_edit_archive_transaction() -> bool {
    #[derive(strum_macros::Display, PartialEq)]
    enum ConfirmOptions {
        #[strum(to_string = "Yes, I want to edit the transaction.")]
        Yes,
        #[strum(to_string = "No, I don't want to edit the transaction.")]
        No,
    }
    let select_choose_input = Select::new(
        "Do you want to edit an archive transaction?",
        vec![ConfirmOptions::Yes, ConfirmOptions::No],
    )
    .prompt()
    .unwrap_or(ConfirmOptions::Yes);
    select_choose_input == ConfirmOptions::Yes
}
