use dialoguer::{theme::ColorfulTheme, Input, Select};
use near_primitives::borsh::BorshSerialize;
use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

/// Sign constructed transaction with Ledger
#[derive(Debug, Default, clap::Clap)]
pub struct CliSignLedger {
    #[clap(long)]
    seed_phrase_hd_path: Option<slip10::BIP32Path>,
    #[clap(subcommand)]
    submit: Option<Submit>,
}

#[derive(Debug)]
pub struct SignLedger {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub submit: Option<Submit>,
}

impl From<CliSignLedger> for SignLedger {
    fn from(item: CliSignLedger) -> Self {
        let seed_phrase_hd_path = match item.seed_phrase_hd_path {
            Some(hd_path) => hd_path,
            None => SignLedger::input_seed_phrase_hd_path(),
        };
        let submit: Option<Submit> = item.submit;

        Self {
            seed_phrase_hd_path,
            submit,
        }
    }
}

impl SignLedger {
    fn rpc_client(self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub fn input_seed_phrase_hd_path() -> slip10::BIP32Path {
        let input: String = Input::new()
            .with_prompt("Enter seed phrase HD Path (if you not sure leave blank for default)")
            .default("44'/397'/0'/0'/1'".into())
            .interact_text()
            .unwrap();
        slip10::BIP32Path::from_str(input.as_str()).unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let seed_phrase_hd_path = self.seed_phrase_hd_path.clone();

        println!(
            "Please allow getting the PublicKey on Ledger device (HD Path: {})",
            seed_phrase_hd_path
        );
        let public_key = match near_ledger::get_public_key(seed_phrase_hd_path.clone()).await {
            Ok(public_key) => near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
                public_key.to_bytes(),
            )),
            Err(near_ledger_error) => {
                return Err(color_eyre::Report::msg(format!(
                    "An error occurred while trying to get PublicKey from Ledger device: {:?}",
                    near_ledger_error
                )));
            }
        };

        let submit: Option<Submit> = self.submit.clone();
        match network_connection_config {
            None => {
                let unsigned_transaction = near_primitives::transaction::Transaction {
                    public_key,
                    ..prepopulated_unsigned_transaction
                };
                println!(
                    "{:#?}\n Confirm transaction signing on your Ledger device (HD Path: {})",
                    unsigned_transaction, seed_phrase_hd_path,
                );
                let signature = match near_ledger::sign_transaction(
                    unsigned_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                    self.seed_phrase_hd_path,
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
                println!(
                    "\n\n---  Signed transaction:   ---\n    {:#?}",
                    &signed_transaction
                );
                match submit {
                    Some(submit) => submit.process_offline(signed_transaction, serialize_to_base64),
                    None => {
                        let submit = Submit::choose_submit();
                        submit.process_offline(signed_transaction, serialize_to_base64)
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

                println!(
                    "{:#?}\n Confirm transaction signing on your Ledger device (HD Path: {})",
                    unsigned_transaction, seed_phrase_hd_path,
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
                        println!("LEDGER ERROR {:?}", near_ledger_error);
                        color_eyre::Report::msg(format!(
                            "Transaction is not expected to fail on serialization: {:?}",
                            near_ledger_error
                        ));
                        return Ok(());
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
                println!(
                    "\n\n---  Signed transaction:   ---\n    {:#?}",
                    &signed_transaction
                );
                match submit {
                    None => {
                        let submit = Submit::choose_submit();
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

#[derive(Debug, EnumDiscriminants, Clone, clap::Clap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Submit {
    #[strum_discriminants(strum(
        message = "Do you want send the transaction to the server (it's works only for online mode)"
    ))]
    Send,
    #[strum_discriminants(strum(message = "Do you want show the transaction on display?"))]
    Display,
}

impl Submit {
    pub fn choose_submit() -> Self {
        println!();
        let variants = SubmitDiscriminants::iter().collect::<Vec<_>>();
        let submits = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_submit = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action that you want to add to the action:")
            .items(&submits)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_submit] {
            SubmitDiscriminants::Send => Submit::Send,
            SubmitDiscriminants::Display => Submit::Display,
        }
    }

    pub fn process_offline(
        self,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> crate::CliResult {
        println!("\n\n\n===========  DISPLAY  ==========");
        println!(
            "\n\n---  Signed transaction:   ---\n    {:#?}",
            &signed_transaction
        );
        println!(
            "\n\n---  serialize_to_base64:   --- \n   {:#?}",
            &serialize_to_base64
        );
        Ok(())
    }

    pub async fn process_online(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> crate::CliResult {
        match self {
            Submit::Send => {
                println!("\n\n\n========= SENT =========");
                println!(
                    "\n\n---  Signed transaction:   ---\n    {:#?}",
                    &signed_transaction
                );
                println!(
                    "\n\n---  serialize_to_base64:   --- \n   {:#?}",
                    &serialize_to_base64
                );
                let json_rcp_client =
                    near_jsonrpc_client::new_client(network_connection_config.rpc_url().as_str());
                let transaction_info = loop {
                    let transaction_info_result = json_rcp_client
                        .broadcast_tx_commit(near_primitives::serialize::to_base64(
                            signed_transaction
                                .try_to_vec()
                                .expect("Transaction is not expected to fail on serialization"),
                        ))
                        .await;
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(err) => {
                            if let Some(serde_json::Value::String(data)) = &err.data {
                                if data.contains("Timeout") {
                                    println!("Error transaction: {:?}", err);
                                    continue;
                                }
                            }
                            return Err(color_eyre::Report::msg(format!(
                                "Error transaction: {:?}",
                                err
                            )));
                        }
                    };
                };
                println!("\n\n---  Success:  ---\n {:#?}", &transaction_info);
            }
            Submit::Display => {
                println!("\n\n\n===========  DISPLAY  ==========");
                println!(
                    "\n\n---  Signed transaction:   ---\n {:#?}",
                    &signed_transaction
                );
                println!(
                    "\n\n---  serialize_to_base64:   --- \n {:#?}",
                    &serialize_to_base64
                );
            }
        }
        Ok(())
    }
}
