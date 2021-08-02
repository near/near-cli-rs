use near_primitives::borsh::BorshSerialize;

/// подписание сформированной транзакции с помощью личных ключей
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSignPrivateKey {
    #[clap(long)]
    signer_public_key: Option<near_crypto::PublicKey>,
    #[clap(long)]
    signer_private_key: Option<near_crypto::SecretKey>,
    #[clap(long)]
    nonce: Option<u64>,
    #[clap(long)]
    block_hash: Option<near_primitives::hash::CryptoHash>,
    #[clap(subcommand)]
    submit: Option<super::Submit>,
}

#[derive(Debug, Clone)]
pub struct SignPrivateKey {
    pub signer_public_key: near_crypto::PublicKey,
    pub signer_private_key: near_crypto::SecretKey,
    pub nonce: Option<u64>,
    pub block_hash: Option<near_primitives::hash::CryptoHash>,
    pub submit: Option<super::Submit>,
}

impl CliSignPrivateKey {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .submit
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(block_hash) = &self.block_hash {
            args.push_front(block_hash.to_string());
            args.push_front("--block-hash".to_owned())
        }
        if let Some(nonce) = &self.nonce {
            args.push_front(nonce.to_string());
            args.push_front("--nonce".to_owned())
        }
        if let Some(signer_secret_key) = &self.signer_private_key {
            args.push_front(signer_secret_key.to_string());
            args.push_front("--signer-private-key".to_owned())
        }
        if let Some(signer_public_key) = &self.signer_public_key {
            args.push_front(signer_public_key.to_string());
            args.push_front("--signer-public-key".to_owned())
        }
        args
    }
}

impl From<SignPrivateKey> for CliSignPrivateKey {
    fn from(sign_private_key: SignPrivateKey) -> Self {
        Self {
            signer_public_key: Some(sign_private_key.signer_public_key),
            signer_private_key: Some(sign_private_key.signer_private_key),
            nonce: sign_private_key.nonce,
            block_hash: sign_private_key.block_hash,
            submit: sign_private_key.submit,
        }
    }
}

impl SignPrivateKey {
    pub fn from(
        item: CliSignPrivateKey,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> Self {
        let signer_public_key: near_crypto::PublicKey = match item.signer_public_key {
            Some(cli_public_key) => cli_public_key,
            None => super::input_signer_public_key(),
        };
        let signer_private_key: near_crypto::SecretKey = match item.signer_private_key {
            Some(signer_private_key) => signer_private_key,
            None => super::input_signer_private_key(),
        };
        let submit: Option<super::Submit> = item.submit;
        match connection_config {
            Some(_) => Self {
                signer_public_key,
                signer_private_key,
                nonce: None,
                block_hash: None,
                submit,
            },
            None => {
                let nonce: u64 = match item.nonce {
                    Some(cli_nonce) => cli_nonce,
                    None => super::input_access_key_nonce(&signer_public_key.to_string()),
                };
                let block_hash = match item.block_hash {
                    Some(cli_block_hash) => cli_block_hash,
                    None => super::input_block_hash(),
                };
                let public_key_origin: near_crypto::PublicKey =
                    near_crypto::SecretKey::public_key(&signer_private_key);
                if &signer_public_key == &public_key_origin {
                    Self {
                        signer_public_key,
                        signer_private_key,
                        nonce: Some(nonce),
                        block_hash: Some(block_hash),
                        submit,
                    }
                } else {
                    println!("\nError: The key pair does not match. Re-enter the keys.\n");
                    let signer_public_key: near_crypto::PublicKey =
                        super::input_signer_public_key();
                    let signer_secret_key: near_crypto::SecretKey =
                        super::input_signer_private_key();
                    Self::from(
                        CliSignPrivateKey {
                            signer_public_key: Some(signer_public_key),
                            signer_private_key: Some(signer_secret_key),
                            nonce: Some(nonce),
                            block_hash: Some(block_hash),
                            submit: None,
                        },
                        connection_config,
                    )
                }
            }
        }
    }
}

impl SignPrivateKey {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let public_key: near_crypto::PublicKey = self.signer_public_key.clone();
        let signer_secret_key: near_crypto::SecretKey = self.signer_private_key.clone();
        let nonce: u64 = self.nonce.unwrap_or_default().clone();
        let block_hash: near_primitives::hash::CryptoHash =
            self.block_hash.unwrap_or_default().clone();
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
                let online_signer_access_key_response = self
                    .rpc_client(network_connection_config.rpc_url().as_str())
                    .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
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
