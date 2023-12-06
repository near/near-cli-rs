use color_eyre::eyre::ContextCompat;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SignedTransactionContext)]
#[interactive_clap(output_context = NetworkContext)]
pub struct Network {
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    pub submit: Submit,
}

#[derive(Debug, Clone)]
pub struct NetworkContext {
    signed_transaction: near_primitives::transaction::SignedTransaction,
    network_config: crate::config::NetworkConfig,
}

impl NetworkContext {
    pub fn from_previous_context(
        previous_context: super::SignedTransactionContext,
        scope: &<Network as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context
            .config
            .network_connection
            .get(&scope.network_name)
            .wrap_err("Failed to get network config!")?
            .clone();

        Ok(Self {
            signed_transaction: previous_context.signed_transaction,
            network_config,
        })
    }
}

impl Network {
    fn input_network_name(
        context: &super::SignedTransactionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(
            &context.config,
            &[context.signed_transaction.transaction.receiver_id.clone()],
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = NetworkContext)]
#[interactive_clap(output_context = SubmitContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to proceed?
pub enum Submit {
    #[strum_discriminants(strum(message = "send - Send the transaction to the network"))]
    Send,
}

#[derive(Debug, Clone)]
pub struct SubmitContext;

impl SubmitContext {
    pub fn from_previous_context(
        previous_context: NetworkContext,
        _scope: &<Submit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> crate::CliResult {
        eprintln!("Transaction sent ...");
        let transaction_info = loop {
            let transaction_info_result = previous_context
                .network_config
                .json_rpc_client()
                .blocking_call(
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                    signed_transaction: previous_context.signed_transaction.clone(),
                },
            );
            match transaction_info_result {
                Ok(response) => {
                    break response;
                }
                Err(err) => match crate::common::rpc_transaction_error(err) {
                    Ok(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(report) => return Err(color_eyre::Report::msg(report)),
                },
            };
        };
        crate::common::print_transaction_status(&transaction_info, &previous_context.network_config)
    }
}
