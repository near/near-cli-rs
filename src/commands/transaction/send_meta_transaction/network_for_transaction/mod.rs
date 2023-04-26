#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::sign_as::RelayerAccountIdContext)]
#[interactive_clap(output_context = NetworkForTransactionArgsContext)]
pub struct NetworkForTransactionArgs {
    /// What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    transaction_signature_options: super::transaction_signature_options::SignWith,
}

#[derive(Clone)]
pub struct NetworkForTransactionArgsContext {
    pub config: crate::config::Config,
    pub transaction_hash: String,
    pub relayer_account_id: near_primitives::types::AccountId,
    pub network_config: crate::config::NetworkConfig,
}

impl NetworkForTransactionArgsContext {
    pub fn from_previous_context(
        previous_context: super::sign_as::RelayerAccountIdContext,
        scope: &<NetworkForTransactionArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_connection = previous_context.config.network_connection.clone();
        let network_config = network_connection
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        Ok(Self {
            config: previous_context.config,
            transaction_hash: previous_context.transaction_hash,
            relayer_account_id: previous_context.relayer_account_id,
            network_config,
        })
    }
}

impl NetworkForTransactionArgs {
    fn input_network_name(
        context: &super::sign_as::RelayerAccountIdContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&(context.config.clone(),))
    }
}
