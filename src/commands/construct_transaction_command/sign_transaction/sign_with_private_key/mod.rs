use near_primitives::borsh::BorshSerialize;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignPrivateKey {
    #[interactive_clap(long)]
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    pub signer_private_key: crate::types::secret_key::SecretKey,
    #[interactive_clap(long)]
    pub nonce: Option<u64>,
    #[interactive_clap(long)]
    pub block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(subcommand)]
    pub submit: Option<super::Submit>,
}

impl SignPrivateKey {
    pub fn from_cli(
        optional_clap_variant: Option<<SignPrivateKey as interactive_clap::ToCli>::CliVariant>,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let signer_public_key: crate::types::public_key::PublicKey = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.signer_public_key)
        {
            Some(cli_public_key) => cli_public_key,
            None => super::input_signer_public_key()?,
        };
        let signer_private_key: crate::types::secret_key::SecretKey = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.signer_private_key)
        {
            Some(signer_private_key) => signer_private_key,
            None => super::input_signer_private_key()?,
        };
        let submit: Option<super::Submit> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.submit)
        {
            Some(submit) => Some(submit),
            None => None,
        };

        match connection_config {
            Some(_) => Ok(Self {
                signer_public_key,
                signer_private_key,
                nonce: None,
                block_hash: None,
                submit,
            }),
            None => {
                let nonce: u64 = match optional_clap_variant
                    .clone()
                    .and_then(|clap_variant| clap_variant.nonce)
                {
                    Some(cli_nonce) => cli_nonce,
                    None => super::input_access_key_nonce(&signer_public_key.to_string())?,
                };
                let block_hash = match optional_clap_variant
                    .clone()
                    .and_then(|clap_variant| clap_variant.block_hash)
                {
                    Some(cli_block_hash) => cli_block_hash,
                    None => super::input_block_hash()?,
                };
                let public_key_origin: near_crypto::PublicKey =
                    near_crypto::SecretKey::public_key(&signer_private_key.clone().into());
                if &signer_public_key.0 == &public_key_origin {
                    Ok(Self {
                        signer_public_key,
                        signer_private_key,
                        nonce: Some(nonce),
                        block_hash: Some(block_hash),
                        submit,
                    })
                } else {
                    println!("\nError: The key pair does not match. Re-enter the keys.\n");
                    let signer_public_key: crate::types::public_key::PublicKey =
                        super::input_signer_public_key()?;
                    let signer_secret_key: crate::types::secret_key::SecretKey =
                        super::input_signer_private_key()?;
                    Self::from_cli(
                        Some(CliSignPrivateKey {
                            signer_public_key: Some(signer_public_key),
                            signer_private_key: Some(signer_secret_key),
                            nonce: Some(nonce),
                            block_hash: Some(block_hash),
                            submit: None,
                        }),
                        context,
                    )
                }
            }
        }
    }
}

impl SignPrivateKey {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let public_key: near_crypto::PublicKey = self.signer_public_key.0.clone();
        let signer_secret_key: near_crypto::SecretKey = self.signer_private_key.clone().into();
        let nonce: u64 = self.nonce.unwrap_or_default().clone();
        let block_hash: near_primitives::hash::CryptoHash =
            self.clone().block_hash.unwrap_or_default().0;
        let submit: Option<super::Submit> = self.submit.clone();
        match connection_config.clone() {
            None => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    nonce,
                    block_hash,
                    ..prepopulated_unsigned_transaction
                };
                let signature =
                    signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!("\nSigned transaction:\n");
                crate::common::print_transaction(signed_transaction.transaction.clone());
                println!("Your transaction was signed successfully.");
                match submit {
                    Some(submit) => submit.process_offline(serialize_to_base64),
                    None => {
                        let submit = super::Submit::choose_submit(connection_config.clone());
                        submit.process_offline(serialize_to_base64)
                    }
                }
            }
            Some(network_connection_config) => {
                let online_signer_access_key_response =
                    near_jsonrpc_client::JsonRpcClient::connect(
                        &network_connection_config.rpc_url().as_str(),
                    )
                    .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                        block_reference: near_primitives::types::Finality::Final.into(),
                        request: near_primitives::views::QueryRequest::ViewAccessKey {
                            account_id: prepopulated_unsigned_transaction.signer_id.clone(),
                            public_key: public_key.clone(),
                        },
                    })
                    .await
                    .map_err(|err| {
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
                        return Err(color_eyre::Report::msg(format!("Error current_nonce")));
                    };
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    block_hash: online_signer_access_key_response.block_hash,
                    nonce: current_nonce + 1,
                    ..prepopulated_unsigned_transaction
                };
                let signature =
                    signer_secret_key.sign(unsigned_transaction.get_hash_and_size().0.as_ref());
                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!("\nSigned transaction:\n");
                crate::common::print_transaction(signed_transaction.transaction.clone());
                println!("Your transaction was signed successfully.");
                match submit {
                    None => {
                        let submit = super::Submit::choose_submit(connection_config);
                        submit
                            .process_online(
                                network_connection_config,
                                signed_transaction,
                                serialize_to_base64,
                            )
                            .await
                    }
                    Some(submit) => {
                        submit
                            .process_online(
                                network_connection_config,
                                signed_transaction,
                                serialize_to_base64,
                            )
                            .await
                    }
                }
            }
        }
    }
}
