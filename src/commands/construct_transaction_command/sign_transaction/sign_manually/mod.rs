use near_primitives::borsh::BorshSerialize;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignManually {
    #[interactive_clap(long)]
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    block_hash: Option<crate::types::crypto_hash::CryptoHash>,
}

impl interactive_clap::ToCli for crate::types::public_key::PublicKey {
    type CliVariant = crate::types::public_key::PublicKey;
}

impl interactive_clap::ToCli for crate::types::crypto_hash::CryptoHash {
    type CliVariant = crate::types::crypto_hash::CryptoHash;
}

impl SignManually {
    pub fn from_cli(
        optional_clap_variant: Option<CliSignManually>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let signer_public_key: crate::types::public_key::PublicKey = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.signer_public_key)
        {
            Some(cli_public_key) => cli_public_key,
            None => super::input_signer_public_key()?,
        };
        match connection_config {
            Some(_) => Ok(Self {
                signer_public_key,
                nonce: None,
                block_hash: None,
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
                Ok(Self {
                    signer_public_key,
                    nonce: Some(nonce),
                    block_hash: Some(block_hash),
                })
            }
        }
    }
}

impl SignManually {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let public_key: near_crypto::PublicKey = self.signer_public_key.0.clone();

        let unsigned_transaction = match network_connection_config {
            None => near_primitives::transaction::Transaction {
                public_key,
                nonce: self.nonce.unwrap_or_default().clone(),
                block_hash: self.block_hash.unwrap_or_default().0.clone(),
                ..prepopulated_unsigned_transaction
            },
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
                near_primitives::transaction::Transaction {
                    public_key,
                    block_hash: online_signer_access_key_response.block_hash,
                    nonce: current_nonce + 1,
                    ..prepopulated_unsigned_transaction
                }
            }
        };

        println!();
        println!("Unsigned transaction:");
        crate::common::print_transaction(unsigned_transaction.clone());
        let serialize_to_base64 = near_primitives::serialize::to_base64(
            unsigned_transaction
                .try_to_vec()
                .expect("Transaction is not expected to fail on serialization"),
        );
        println!("\nSerialize_to_base64:\n{}", &serialize_to_base64);
        Ok(None)
    }
}
