#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignAccessKeyFile {
    /// What is the location of the account access key file (path/to/access-key-file.json)?
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignAccessKeyFileContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignAccessKeyFileContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignAccessKeyFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Result<Self, color_eyre::eyre::Error> {
        let network_config = previous_context.network_config.clone();

        let data = std::fs::read_to_string(&scope.file_path).map_err(|err| {
            color_eyre::Report::msg(format!("Access key file not found! Error: {}", err))
        })?;
        let account_json: super::AccountKeyPair = serde_json::from_str(&data)
            .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {}", err)))?;

        let online_signer_access_key_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKey {
                        account_id: previous_context.transaction.signer_id.clone(),
                        public_key: account_json.public_key.clone(),
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
            public_key: account_json.public_key.clone(),
            block_hash: online_signer_access_key_response.block_hash,
            nonce: current_nonce + 1,
            ..previous_context.transaction.clone()
        };

        (previous_context.on_before_signing_callback)(&mut unsigned_transaction, &network_config)?;

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
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
        println!("Public key: {}", account_json.public_key);
        println!("Signature: {}", signature);

        Ok(Self {
            network_config: previous_context.network_config,
            signed_transaction,
            on_after_sending_transaction_callback: previous_context
                .on_after_sending_transaction_callback,
        })
    }
}

impl From<SignAccessKeyFileContext> for super::SubmitContext {
    fn from(item: SignAccessKeyFileContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction: item.signed_transaction.into(),
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignAccessKeyFile {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignAccessKeyFile as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let file_path: crate::types::path_buf::PathBuf = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.file_path)
        {
            Some(cli_file_path) => cli_file_path,
            None => Self::input_file_path(&context)?,
        };
        let new_context_scope = InteractiveClapContextScopeForSignAccessKeyFile {
            file_path: file_path.clone(),
        };
        let access_key_file_context =
            SignAccessKeyFileContext::from_previous_context(context.clone(), &new_context_scope)?;
        let new_context = super::SubmitContext::from(access_key_file_context.clone());

        let optional_submit = super::Submit::from_cli(
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit),
            new_context,
        )?;
        let submit = if let Some(submit) = optional_submit {
            submit
        } else {
            return Ok(None);
        };

        Ok(Some(Self { file_path, submit }))
    }
}
