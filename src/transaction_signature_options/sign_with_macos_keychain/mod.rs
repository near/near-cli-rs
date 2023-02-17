#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignMacosKeychain {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    block_hash: Option<String>,
    #[interactive_clap(subcommand)]
    submit: super::Submit,
}

#[derive(Clone)]
pub struct SignMacosKeychainContext {
    network_config: crate::config::NetworkConfig,
    signed_transaction: near_primitives::transaction::SignedTransaction,
    on_after_sending_transaction_callback:
        crate::transaction_signature_options::OnAfterSendingTransactionCallback,
}

impl SignMacosKeychainContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        _scope: &<SignMacosKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Result<Self, color_eyre::eyre::Error> {
        let network_config = previous_context.network_config.clone();

        let keychain =
            security_framework::os::macos::keychain::SecKeychain::default().map_err(|err| {
                color_eyre::Report::msg(format!("Failed to open keychain: {:?}", err))
            })?;

        let query_view_method_response = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(network_config.json_rpc_client().call(
                near_jsonrpc_client::methods::query::RpcQueryRequest {
                    block_reference: near_primitives::types::Finality::Final.into(),
                    request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                        account_id: previous_context.transaction.signer_id.clone(),
                    },
                },
            ))
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch access key list for {}: {:?}",
                    previous_context.transaction.signer_id, err
                ))
            })?;

        let access_key_list =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg("Error call result".to_string()));
            };
        let service_name = std::borrow::Cow::Owned(format!(
            "near-{}-{}",
            network_config.network_name,
            previous_context.transaction.signer_id.as_str()
        ));
        let password = access_key_list
            .keys
            .into_iter()
            .filter(|key| {
                matches!(
                    key.access_key.permission,
                    near_primitives::views::AccessKeyPermissionView::FullAccess
                )
            })
            .map(|key| key.public_key)
            .find_map(|public_key| {
                let (password, _) = keychain
                    .find_generic_password(
                        &service_name,
                        &format!("{}:{}", previous_context.transaction.signer_id, public_key),
                    )
                    .ok()?;
                Some(password)
            })
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(format!(
                    "There are no access keys for {} account in the macOS keychain.",
                    previous_context.transaction.signer_id
                ))
            })?;

        let account_json: super::AccountKeyPair = serde_json::from_slice(password.as_ref())
            .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {:?}", err)))?;

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

        //XXX print unsigned transaction

        //XXX do you want to sign transaction?

        let signature = account_json
            .private_key
            .sign(unsigned_transaction.get_hash_and_size().0.as_ref());
        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            signature.clone(),
            unsigned_transaction,
        );

        (previous_context.on_after_signing_callback)(&signed_transaction)?;

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

impl From<SignMacosKeychainContext> for super::SubmitContext {
    fn from(item: SignMacosKeychainContext) -> Self {
        Self {
            network_config: item.network_config,
            signed_transaction: item.signed_transaction.into(),
            on_after_sending_transaction_callback: item.on_after_sending_transaction_callback,
        }
    }
}

impl interactive_clap::FromCli for SignMacosKeychain {
    type FromCliContext = crate::commands::TransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<SignMacosKeychain as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let nonce: Option<u64> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.nonce);
        let block_hash: Option<String> = optional_clap_variant
            .as_ref()
            .and_then(|clap_variant| clap_variant.block_hash.clone());

        let new_context_scope = InteractiveClapContextScopeForSignMacosKeychain {
            nonce,
            block_hash: block_hash.clone(),
        };
        let keychain_context =
            SignMacosKeychainContext::from_previous_context(context.clone(), &new_context_scope)?;
        let new_context = super::SubmitContext::from(keychain_context.clone());

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
            nonce,
            block_hash,
            submit,
        }))
    }
}
