#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::ActionContext)]
#[interactive_clap(output_context = crate::commands::TransactionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct NetworkForTransactionArgs {
    ///What is the name of the network
    #[interactive_clap(skip_default_input_arg)]
    network_name: String,
    #[interactive_clap(skip)]
    prepopulated_unsigned_transaction: crate::types::transaction::Transaction,
    #[interactive_clap(subcommand)]
    transaction_signature_options: crate::transaction_signature_options::SignWith,
}

#[derive(Debug, Clone)]
pub struct NetworkForTransactionArgsContext {
    config: crate::config::Config,
    network_name: String,
    prepopulated_unsigned_transaction: crate::types::transaction::Transaction,
}

impl NetworkForTransactionArgsContext {
    pub fn from_previous_context(
        previous_context: crate::commands::ActionContext,
        scope: &<NetworkForTransactionArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            config: previous_context.config,
            network_name: scope.network_name.clone(),
            prepopulated_unsigned_transaction: scope.prepopulated_unsigned_transaction.clone(),
        }
    }
}

impl From<NetworkForTransactionArgsContext> for crate::commands::TransactionContext {
    fn from(item: NetworkForTransactionArgsContext) -> Self {
        let networks = item.config.networks.clone();
        let network_config = networks
            .get(&item.network_name)
            .expect("Failed to get network config!")
            .clone();
        Self {
            config: item.config,
            network_config,
            transaction: item.prepopulated_unsigned_transaction.into(),
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
        let prepopulated_unsigned_transaction =
            crate::types::transaction::Transaction(near_primitives::transaction::Transaction {
                signer_id: context.signer_account_id.clone(),
                public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
                nonce: 0,
                receiver_id: context.receiver_account_id.clone(),
                block_hash: Default::default(),
                actions: context.actions.clone(),
            });

        let new_context_scope = InteractiveClapContextScopeForNetworkForTransactionArgs {
            network_name: network_name.clone(),
            prepopulated_unsigned_transaction: prepopulated_unsigned_transaction.clone(),
        };
        let new_context = NetworkForTransactionArgsContext::from_previous_context(
            context.clone(),
            &new_context_scope,
        );
        // let new_context = crate::commands::TransactionContext::from(network_context);

        // println!("\nUnsigned transaction:\n");
        // crate::common::print_unsigned_transaction(new_context.transaction.clone().into());
        // println!();

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
            prepopulated_unsigned_transaction,
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
