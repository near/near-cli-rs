#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::ActionContext)]
#[interactive_clap(output_context = crate::commands::TransactionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct NetworkForTransactionArgs {
    /// What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(subcommand)]
    transaction_signature_options: crate::transaction_signature_options::SignWith,
}

#[derive(Clone)]
pub struct NetworkForTransactionArgsContext {
    config: crate::config::Config,
    network_config: crate::config::NetworkConfig,
    prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    on_before_signing_callback: crate::commands::OnBeforeSigningCallback,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl NetworkForTransactionArgsContext {
    pub fn from_previous_context(
        previous_context: crate::commands::ActionContext,
        scope: &<NetworkForTransactionArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: previous_context.signer_account_id.clone(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: previous_context.receiver_account_id.clone(),
            block_hash: Default::default(),
            actions: previous_context.actions.clone(),
        };
        let networks = previous_context.config.networks.clone();
        let network_config = networks
            .get(&scope.network_name)
            .expect("Failed to get network config!")
            .clone();
        Ok(Self {
            config: previous_context.config,
            network_config,
            prepopulated_unsigned_transaction,
            on_before_signing_callback: previous_context.on_before_signing_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<NetworkForTransactionArgsContext> for crate::commands::TransactionContext {
    fn from(previous_context: NetworkForTransactionArgsContext) -> Self {
        Self {
            config: previous_context.config,
            network_config: previous_context.network_config,
            transaction: previous_context.prepopulated_unsigned_transaction,
            on_before_signing_callback: previous_context.on_before_signing_callback,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
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
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let network_name = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.network_name.clone())
        {
            Some(network_name) => network_name,
            None => NetworkForTransactionArgs::input_network_name(&context)?,
        };

        let new_context_scope = InteractiveClapContextScopeForNetworkForTransactionArgs {
            network_name: network_name.clone(),
        };
        let mut new_context = NetworkForTransactionArgsContext::from_previous_context(
            context.clone(),
            &new_context_scope,
        )?;

        (context.on_after_getting_network_callback)(
            &mut new_context.prepopulated_unsigned_transaction,
            &new_context.network_config,
        )?;
        if new_context
            .prepopulated_unsigned_transaction
            .actions
            .is_empty()
        {
            return Err(crate::common::CliError::ExitOk.into());
        }

        println!("\nUnsigned transaction:\n"); // XXX remove!
        crate::common::print_unsigned_transaction(
            new_context.prepopulated_unsigned_transaction.clone().into(),
        );
        println!();

        let optional_transaction_signature_options =
            crate::transaction_signature_options::SignWith::from_cli(
                optional_clap_variant
                    .and_then(|clap_variant| clap_variant.transaction_signature_options),
                new_context.into(),
            )?;
        let transaction_signature_options =
            if let Some(transaction_signature_options) = optional_transaction_signature_options {
                transaction_signature_options
            } else {
                return Ok(None);
            };

        Ok(Some(Self {
            network_name,
            transaction_signature_options,
        }))
    }
}

impl NetworkForTransactionArgs {
    fn input_network_name(
        context: &crate::commands::ActionContext,
    ) -> color_eyre::eyre::Result<String> {
        crate::common::input_network_name(&(context.config.clone(),))
    }

    pub fn get_network_config(
        &self,
        config: crate::config::Config,
    ) -> crate::config::NetworkConfig {
        let network_config = config.networks;
        network_config
            .get(self.network_name.as_str())
            .expect("Impossible to get network name!")
            .clone()
    }

    pub fn get_sign_option(&self) -> crate::transaction_signature_options::SignWith {
        self.transaction_signature_options.clone()
    }
}
