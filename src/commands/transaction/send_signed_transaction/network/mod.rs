use color_eyre::eyre::ContextCompat;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

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
            &[context.signed_transaction.transaction.receiver_id().clone()],
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
    #[tracing::instrument(name = "Sending transaction ...", skip_all)]
    pub fn from_previous_context(
        previous_context: NetworkContext,
        _scope: &<Submit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> crate::CliResult {
        let transaction_info =
            crate::transaction_signature_options::send::sending_signed_transaction(
                &previous_context.network_config,
                &previous_context.signed_transaction,
            )?;

        crate::common::print_transaction_status(&transaction_info, &previous_context.network_config)
    }
}
