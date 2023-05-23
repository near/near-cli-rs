#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::ActionContext)]
#[interactive_clap(output_context = NetworkForTransactionArgsContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct NetworkForTransactionArgs {
    /// What is the name of the network?
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    transaction_signature_options: crate::transaction_signature_options::SignWith,
}

#[derive(Clone)]
pub struct NetworkForTransactionArgsContext {
    config: crate::config::Config,
    network_config: crate::config::NetworkConfig,
    prepopulated_transaction: crate::commands::PrepopulatedTransaction,
    on_before_signing_callback: crate::commands::OnBeforeSigningCallback,
    on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl NetworkForTransactionArgsContext {
    pub fn from_previous_context(
        previous_context: crate::commands::ActionContext,
        scope: &<NetworkForTransactionArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_connection = previous_context.config.network_connection.clone();
        let network_config = network_connection
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        let mut prepopulated_transaction =
            (previous_context.on_after_getting_network_callback)(&network_config)?;
        (previous_context.on_refine_prepopulated_transaction_callback)(
            &mut prepopulated_transaction,
            &network_config,
        )?;
        Ok(Self {
            config: previous_context.config,
            network_config,
            prepopulated_transaction,
            on_before_signing_callback: previous_context.on_before_signing_callback,
            on_before_sending_transaction_callback: previous_context
                .on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<NetworkForTransactionArgsContext> for crate::commands::TransactionContext {
    fn from(item: NetworkForTransactionArgsContext) -> Self {
        Self {
            config: item.config,
            network_config: item.network_config,
            prepopulated_transaction: item.prepopulated_transaction,
            on_before_signing_callback: item.on_before_signing_callback,
            on_before_sending_transaction_callback: item.on_before_sending_transaction_callback,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for NetworkForTransactionArgs {
    type FromCliContext = crate::commands::ActionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<
            <NetworkForTransactionArgs as interactive_clap::ToCli>::CliVariant,
        >,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut clap_variant = optional_clap_variant.unwrap_or_default();

        if clap_variant.network_name.is_none() {
            clap_variant.network_name = match Self::input_network_name(&context) {
                Ok(Some(network_name)) => Some(network_name),
                Ok(None) => return interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
            };
        }
        let network_name = clap_variant.network_name.clone().expect("Unexpected error");

        let new_context_scope =
            InteractiveClapContextScopeForNetworkForTransactionArgs { network_name };
        let new_context = match NetworkForTransactionArgsContext::from_previous_context(
            context,
            &new_context_scope,
        ) {
            Ok(new_context) => new_context,
            Err(err) => return interactive_clap::ResultFromCli::Err(Some(clap_variant), err),
        };

        if new_context.prepopulated_transaction.actions.is_empty() {
            return interactive_clap::ResultFromCli::Cancel(Some(clap_variant));
        }
        if new_context
            .network_config
            .meta_transaction_relayer_url
            .is_some()
        {
            eprintln!("\nUnsigned delegate action:\n");
        } else {
            eprintln!("\nUnsigned transaction:\n");
        }
        crate::common::print_unsigned_transaction(&new_context.prepopulated_transaction);
        eprintln!();

        match <crate::transaction_signature_options::SignWith as interactive_clap::FromCli>::from_cli(
                clap_variant.transaction_signature_options.take(),
                new_context.into(),
            ) {
                interactive_clap::ResultFromCli::Ok(cli_sign_with) | interactive_clap::ResultFromCli::Cancel(Some(cli_sign_with)) => {
                    clap_variant.transaction_signature_options = Some(cli_sign_with);
                    interactive_clap::ResultFromCli::Ok(clap_variant)
                }
                interactive_clap::ResultFromCli::Cancel(_) => interactive_clap::ResultFromCli::Cancel(Some(clap_variant)),
                interactive_clap::ResultFromCli::Back => interactive_clap::ResultFromCli::Back,
                interactive_clap::ResultFromCli::Err(optional_cli_sign_with, err) => {
                    clap_variant.transaction_signature_options = optional_cli_sign_with;
                    interactive_clap::ResultFromCli::Err(Some(clap_variant), err)
                }
            }
    }
}

impl NetworkForTransactionArgs {
    fn input_network_name(
        context: &crate::commands::ActionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&(context.config.clone(),))
    }

    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        let network_config = config.network_connection;
        network_config
            .get(self.network_name.as_str())
            .expect("Impossible to get network name!")
            .clone()
    }

    pub fn get_sign_option(&self) -> crate::transaction_signature_options::SignWith {
        self.transaction_signature_options.clone()
    }
}
