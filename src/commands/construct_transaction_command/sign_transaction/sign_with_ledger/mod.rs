use dialoguer::Input;
use interactive_clap::ToCli;
use near_primitives::borsh::BorshSerialize;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignLedger {
    #[interactive_clap(long)]
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    #[interactive_clap(skip)]
    pub signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    nonce: Option<u64>,
    #[interactive_clap(long)]
    block_hash: Option<crate::types::crypto_hash::CryptoHash>,
    #[interactive_clap(subcommand)]
    pub submit: Option<super::Submit>,
}

impl ToCli for crate::types::slip10::BIP32Path {
    type CliVariant = crate::types::slip10::BIP32Path;
}

impl SignLedger {
    pub fn from_cli(
        optional_clap_variant: Option<CliSignLedger>,
        context: crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<Self> {
        let connection_config = context.connection_config.clone();
        let seed_phrase_hd_path = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.seed_phrase_hd_path)
        {
            Some(hd_path) => hd_path,
            None => SignLedger::input_seed_phrase_hd_path(),
        };
        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = actix::System::new()
            .block_on(async {
                near_ledger::get_public_key(seed_phrase_hd_path.clone().into()).await
            })
            .map_err(|near_ledger_error| {
                color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                ))
            })?;
        let signer_public_key: crate::types::public_key::PublicKey =
            near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
                public_key.to_bytes(),
            ))
            .into();
        let submit: Option<super::Submit> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.submit)
        {
            Some(submit) => Some(submit),
            None => None,
        };
        match connection_config {
            Some(_) => Ok(Self {
                seed_phrase_hd_path,
                signer_public_key,
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
                Ok(Self {
                    seed_phrase_hd_path,
                    signer_public_key,
                    nonce: Some(nonce),
                    block_hash: Some(block_hash),
                    submit,
                })
            }
        }
    }
}

impl SignLedger {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub fn input_seed_phrase_hd_path() -> crate::types::slip10::BIP32Path {
        Input::new()
            .with_prompt("Enter seed phrase HD Path (if you not sure leave blank for default)")
            .with_initial_text("44'/397'/0'/0'/1'")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        let seed_phrase_hd_path: slip10::BIP32Path = self.seed_phrase_hd_path.clone().into();
        let public_key: near_crypto::PublicKey = self.signer_public_key.clone().into();
        let nonce = self.nonce.unwrap_or_default().clone();
        let block_hash: near_primitives::hash::CryptoHash =
            self.clone().block_hash.unwrap_or_default().into();
        let submit: Option<super::Submit> = self.submit.clone();
        match connection_config.clone() {
            None => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    nonce,
                    block_hash,
                    ..prepopulated_unsigned_transaction
                };
                println!("\nUnsigned transaction:\n");
                crate::common::print_transaction(unsigned_transaction.clone());
                println!(
                    "Confirm transaction signing on your Ledger device (HD Path: {})",
                    seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    seed_phrase_hd_path,
                )
                .await
                {
                    Ok(signature) => near_crypto::Signature::from_parts(
                        near_crypto::KeyType::ED25519,
                        &signature,
                    )
                    .expect("Signature is not expected to fail on deserialization"),
                    Err(near_ledger_error) => {
                        return Err(color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            near_ledger_error
                        )));
                    }
                };

                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
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
                println!("\nUnsigned transaction:\n");
                crate::common::print_transaction(unsigned_transaction.clone());
                println!(
                    "Confirm transaction signing on your Ledger device (HD Path: {})",
                    seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    seed_phrase_hd_path,
                )
                .await
                {
                    Ok(signature) => near_crypto::Signature::from_parts(
                        near_crypto::KeyType::ED25519,
                        &signature,
                    )
                    .expect("Signature is not expected to fail on deserialization"),
                    Err(near_ledger_error) => {
                        return Err(color_eyre::Report::msg(format!(
                            "Error occurred while signing the transaction: {:?}",
                            near_ledger_error
                        )));
                    }
                };

                let signed_transaction = near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_transaction,
                );
                let serialize_to_base64 = near_primitives::serialize::to_base64(
                    signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
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
