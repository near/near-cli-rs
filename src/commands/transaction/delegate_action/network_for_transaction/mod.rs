#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::sign_as::RelayerAccountIdContext)]
#[interactive_clap(output_context = NetworkForTransactionArgsContext)]
// #[interactive_clap(skip_default_from_cli)]
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

// impl interactive_clap::FromCli for NetworkForTransactionArgs {
//     type FromCliContext = super::sign_as::RelayerAccountIdContext;
//     type FromCliError = color_eyre::eyre::Error;

//     fn from_cli(
//         optional_clap_variant: Option<
//             <NetworkForTransactionArgs as interactive_clap::ToCli>::CliVariant,
//         >,
//         context: Self::FromCliContext,
//     ) -> interactive_clap::ResultFromCli<
//         <Self as interactive_clap::ToCli>::CliVariant,
//         Self::FromCliError,
//     >
//     where
//         Self: Sized + interactive_clap::ToCli,
//     {
//         let mut clap_variant = optional_clap_variant.unwrap_or_default();

//         if clap_variant.network_name.is_none() {
//             clap_variant.network_name = match Self::input_network_name(&context) {
//                 Ok(Some(network_name)) => Some(network_name),
//                 Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
//                 Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
//             };
//         }
//         let network_name = clap_variant.network_name.clone().expect("Unexpected error");

//         let new_context_scope =
//             InteractiveClapContextScopeForNetworkForTransactionArgs { network_name };
//         let new_context = match NetworkForTransactionArgsContext::from_previous_context(
//             context,
//             &new_context_scope,
//         ) {
//             Ok(new_context) => new_context,
//             Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
//         };

//         // eprintln!("\nUnsigned transaction:\n");
//         // crate::common::print_unsigned_transaction(&new_context.prepopulated_transaction);
//         // eprintln!();

//         match <super::transaction_signature_options::SignWith as interactive_clap::FromCli>::from_cli(
//                 clap_variant.transaction_signature_options.take(),
//                 new_context.into(),
//             ) {
//                 interactive_clap::ResultFromCli::Ok(cli_sign_with) | interactive_clap::ResultFromCli::Cancel(Some(cli_sign_with)) => {
//                     clap_variant.transaction_signature_options = Some(cli_sign_with);
//                     interactive_clap::ResultFromCli::Ok(clap_variant)
//                 }
//                 interactive_clap::ResultFromCli::Cancel(_) => interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
//                 interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
//                 interactive_clap::ResultFromCli::Err(optional_cli_sign_with, err) => {
//                     clap_variant.transaction_signature_options = optional_cli_sign_with;
//                     interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
//                 }
//             }
//     }
// }

impl NetworkForTransactionArgs {
    fn input_network_name(
        context: &super::sign_as::RelayerAccountIdContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&(context.config.clone(),))
    }
}
