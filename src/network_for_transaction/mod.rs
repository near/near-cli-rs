#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
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

impl interactive_clap::FromCli for NetworkForTransactionArgs {
    type FromCliContext = crate::commands::TransactionContext;
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
                signer_id: context.signer_account_id.clone().into(),
                public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
                nonce: 0,
                receiver_id: context.receiver_account_id.clone().into(),
                block_hash: Default::default(),
                actions: context.actions.clone(),
            });
        println!("\nUnsigned transaction:\n");
        crate::common::print_unsigned_transaction(prepopulated_unsigned_transaction.clone().into());
        println!();
        let optional_transaction_signature_options =
            crate::transaction_signature_options::SignWith::from_cli(
                optional_clap_variant
                    .and_then(|clap_variant| clap_variant.transaction_signature_options),
                context,
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
        context: &crate::commands::TransactionContext,
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

    pub fn get_prepopulated_unsigned_transaction(
        &self,
    ) -> near_primitives::transaction::Transaction {
        self.prepopulated_unsigned_transaction.clone().into()
    }
}
