#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
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
    pub submit: Option<super::Submit>,
}

impl SignMacosKeychain {
    pub fn from_cli(
        optional_clap_variant: Option<<SignMacosKeychain as interactive_clap::ToCli>::CliVariant>,
        _context: crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let submit: Option<super::Submit> =
            optional_clap_variant.and_then(|clap_variant| clap_variant.submit);
        Ok(Some(Self {
            nonce: None,
            block_hash: None,
            submit,
        }))
    }
}

impl SignMacosKeychain {
    pub async fn process(
        &self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_config: crate::config::NetworkConfig,
    ) -> crate::CliResult {
        let keychain =
            security_framework::os::macos::keychain::SecKeychain::default().map_err(|err| {
                color_eyre::Report::msg(format!("Failed to open keychain: {:?}", err))
            })?;
        let query_view_method_response = network_config
            .json_rpc_client()?
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                    account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch access key list for {}: {:?}",
                    prepopulated_unsigned_transaction.signer_id, err
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
            "near-testnet-{}",
            prepopulated_unsigned_transaction.signer_id.as_str()
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
                        &format!(
                            "{}:{}",
                            prepopulated_unsigned_transaction.signer_id, public_key
                        ),
                    )
                    .ok()?;
                Some(password)
            })
            .ok_or_else(|| {
                color_eyre::eyre::eyre!(format!(
                    "There are no access keys for {} account in the macOS keychain.",
                    prepopulated_unsigned_transaction.signer_id
                ))
            })?;
        let account_json: super::sign_with_keychain::AccountKeyPair =
            serde_json::from_slice(password.as_ref())
                .map_err(|err| color_eyre::Report::msg(format!("Error reading data: {:?}", err)))?;
        let sign_with_private_key = super::sign_with_private_key::SignPrivateKey {
            signer_public_key: crate::types::public_key::PublicKey(account_json.public_key),
            signer_private_key: crate::types::secret_key::SecretKey(account_json.private_key),
            nonce: self.nonce,
            block_hash: self.block_hash.clone(),
            submit: self.submit.clone(),
        };
        sign_with_private_key
            .process(prepopulated_unsigned_transaction, network_config)
            .await
    }
}
