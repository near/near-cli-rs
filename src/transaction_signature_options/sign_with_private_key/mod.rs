#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = super::SubmitContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignPrivateKey {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub signer_private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    pub block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    pub submit: super::Submit,
}

#[derive(Clone)]
pub struct SignPrivateKeyContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignPrivateKeyContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignPrivateKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Result<Self, color_eyre::eyre::Error> {
        let network_config = previous_context.network_config.clone();
        let signer_secret_key: near_crypto::SecretKey = scope.signer_private_key.clone().into();
        let online_signer_access_key_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKey {
                        account_id: previous_context.transaction.signer_id.clone(),
                        public_key: scope.signer_public_key.clone().into(),
                    },
                },
            ))
            .map_err(|err| {
                println!("\nYour transaction was not successfully signed.\n");
                color_eyre::Report::msg(format!(
                    "Failed to fetch public key information for nonce: {:?}",
                    err
                ))
            })?;
        let current_nonce =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(
                online_signer_access_key,
            ) = online_signer_access_key_response.kind
            {
                online_signer_access_key.nonce
            } else {
                return Err(color_eyre::Report::msg("Error current_nonce".to_string()));
            };

        let mut unsigned_transaction = near_primitives::transaction::Transaction {
            public_key: scope.signer_public_key.clone().into(),
            block_hash: online_signer_access_key_response.block_hash,
            nonce: current_nonce + 1,
            ..previous_context.transaction.clone()
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        //XXX print unsigned transaction

        //XXX do you want to sign transaction?

        let signature = signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        for action in signed_transaction.transaction.actions.iter() {
            if let near_primitives::transaction::Action::FunctionCall(_) = action {
                println!("\nSigned transaction:\n");
                crate::common::print_transaction(signed_transaction.transaction.clone());
            }
        }

        println!("\nYour transaction was signed successfully.");
        println!("Public key: {}", scope.signer_public_key);
        println!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            signed_transaction,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<SignPrivateKeyContext> for super::SubmitContext {
    fn from(item: SignPrivateKeyContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction: item.signed_transaction,
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignPrivateKey {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignPrivateKey as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let signer_public_key: crate::types::public_key::PublicKey = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.signer_public_key.clone())
        {
            Some(cli_public_key) => cli_public_key,
            None => super::input_signer_public_key()?,
        };
        let signer_private_key: crate::types::secret_key::SecretKey = match optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.signer_private_key.clone())
        {
            Some(signer_private_key) => signer_private_key,
            None => super::input_signer_private_key()?,
        };
        let nonce: Option<u64> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.nonce);
        let block_hash: Option<String> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.block_hash.clone());

        let new_context_scope = InteractiveClapContextScopeForSignPrivateKey {
            signer_public_key: signer_public_key.clone(),
            signer_private_key: signer_private_key.clone(),
            nonce,
            block_hash: block_hash.clone(),
        };
        let private_key_context =
            SignPrivateKeyContext::from_previous_context(context.clone(), &new_context_scope)?;
        let new_context = super::SubmitContext::from(private_key_context);

        let optional_submit = super::Submit::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit),
            new_context,
        )?;
        let submit = if let Some(submit) = optional_submit {
            submit
        } else {
            return Ok(None);
        };
        Ok(Some(Self {
            signer_public_key,
            signer_private_key,
            nonce,
            block_hash,
            submit,
        }))
    }
}
