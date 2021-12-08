use dialoguer::{theme::ColorfulTheme, Input, Select};
use near_primitives::borsh::BorshSerialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod sign_manually;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
pub mod sign_with_private_key;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSignTransaction {
    /// Provide arguments to sign a private key transaction
    SignPrivateKey(self::sign_with_private_key::CliSignPrivateKey),
    /// Provide arguments to sign a keychain transaction
    SignWithKeychain(self::sign_with_keychain::CliSignKeychain),
    #[cfg(feature = "ledger")]
    /// Connect your Ledger device and sign transaction with it
    SignWithLedger(self::sign_with_ledger::CliSignLedger),
    /// Provide arguments to sign a manually transaction
    SignManually(self::sign_manually::CliSignManually),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SignTransaction {
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with a plain-text private key"
    ))]
    SignPrivateKey(self::sign_with_private_key::SignPrivateKey),
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with keychain (located in ~/.near-credentials)"
    ))]
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with Ledger Nano S/X device"
    ))]
    SignWithLedger(self::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "No, I want to construct the transaction and sign it somewhere else"
    ))]
    SignManually(self::sign_manually::SignManually),
}

impl CliSignTransaction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            CliSignTransaction::SignPrivateKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("sign-private-key".to_owned());
                args
            }
            CliSignTransaction::SignWithKeychain(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("sign-with-keychain".to_owned());
                args
            }
            #[cfg(feature = "ledger")]
            CliSignTransaction::SignWithLedger(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("sign-with-ledger".to_owned());
                args
            }
            CliSignTransaction::SignManually(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("sign-manually".to_owned());
                args
            }
        }
    }
}

impl From<SignTransaction> for CliSignTransaction {
    fn from(sign_transaction: SignTransaction) -> Self {
        match sign_transaction {
            SignTransaction::SignPrivateKey(sign_with_private_key) => Self::SignPrivateKey(
                self::sign_with_private_key::CliSignPrivateKey::from(sign_with_private_key),
            ),
            SignTransaction::SignWithKeychain(sign_with_keychain) => Self::SignWithKeychain(
                self::sign_with_keychain::CliSignKeychain::from(sign_with_keychain),
            ),
            #[cfg(feature = "ledger")]
            SignTransaction::SignWithLedger(sign_with_ledger) => Self::SignWithLedger(
                self::sign_with_ledger::CliSignLedger::from(sign_with_ledger),
            ),
            SignTransaction::SignManually(sign_manually) => {
                Self::SignManually(self::sign_manually::CliSignManually::from(sign_manually))
            }
        }
    }
}

impl SignTransaction {
    pub fn from(
        item: CliSignTransaction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSignTransaction::SignPrivateKey(cli_private_key) => {
                let private_key = self::sign_with_private_key::SignPrivateKey::from(
                    cli_private_key,
                    connection_config,
                );
                Ok(SignTransaction::SignPrivateKey(private_key))
            }
            CliSignTransaction::SignWithKeychain(cli_key_chain) => {
                let key_chain = self::sign_with_keychain::SignKeychain::from(
                    cli_key_chain,
                    connection_config,
                    sender_account_id,
                )?;
                Ok(SignTransaction::SignWithKeychain(key_chain))
            }
            #[cfg(feature = "ledger")]
            CliSignTransaction::SignWithLedger(cli_ledger) => {
                let ledger =
                    self::sign_with_ledger::SignLedger::from(cli_ledger, connection_config)?;
                Ok(SignTransaction::SignWithLedger(ledger))
            }
            CliSignTransaction::SignManually(cli_manually) => {
                let manually =
                    self::sign_manually::SignManually::from(cli_manually, connection_config);
                Ok(SignTransaction::SignManually(manually))
            }
        }
    }
}

impl SignTransaction {
    pub fn choose_sign_option(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = SignTransactionDiscriminants::iter().collect::<Vec<_>>();
        let sign_options = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_sign_options = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to sign the transaction?")
            .items(&sign_options)
            .default(0)
            .interact()
            .unwrap();
        let cli_sign_option = match variants[select_sign_options] {
            SignTransactionDiscriminants::SignPrivateKey => {
                CliSignTransaction::SignPrivateKey(Default::default())
            }
            SignTransactionDiscriminants::SignWithKeychain => {
                CliSignTransaction::SignWithKeychain(Default::default())
            }
            #[cfg(feature = "ledger")]
            SignTransactionDiscriminants::SignWithLedger => {
                CliSignTransaction::SignWithLedger(Default::default())
            }
            SignTransactionDiscriminants::SignManually => {
                CliSignTransaction::SignManually(Default::default())
            }
        };
        Self::from(cli_sign_option, connection_config, sender_account_id)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            SignTransaction::SignPrivateKey(keys) => {
                keys.process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignWithKeychain(chain) => {
                chain
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            #[cfg(feature = "ledger")]
            SignTransaction::SignWithLedger(ledger) => {
                ledger
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
            SignTransaction::SignManually(args_manually) => {
                args_manually
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

fn input_signer_public_key() -> near_crypto::PublicKey {
    Input::new()
        .with_prompt("Enter sender (signer) public key")
        .interact_text()
        .unwrap()
}

fn input_signer_private_key() -> near_crypto::SecretKey {
    Input::new()
        .with_prompt("Enter sender (signer) private (secret) key")
        .interact_text()
        .unwrap()
}

fn input_access_key_nonce(public_key: &str) -> u64 {
    println!("Your public key: `{}`", public_key);
    Input::new()
        .with_prompt(
            "Enter transaction nonce for this public key (query the access key information with \
            `./near-cli view nonce \
                network testnet \
                account 'volodymyr.testnet' \
                public-key ed25519:...` incremented by 1)",
        )
        .interact_text()
        .unwrap()
}

fn input_block_hash() -> near_primitives::hash::CryptoHash {
    let input_block_hash: crate::common::BlockHashAsBase58 = Input::new()
        .with_prompt(
            "Enter recent block hash (query information about the hash of the last block with \
            `./near-cli view recent-block-hash network testnet`)",
        )
        .interact_text()
        .unwrap();
    input_block_hash.inner
}

#[derive(Debug, EnumDiscriminants, Clone, clap::Clap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Submit {
    #[strum_discriminants(strum(message = "I want to send the transaction to the network"))]
    Send,
    #[strum_discriminants(strum(
        message = "I only want to print base64-encoded transaction for JSON RPC input and exit"
    ))]
    Display,
}

impl Submit {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Send => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("send".to_owned());
                args
            }
            Self::Display => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("display".to_owned());
                args
            }
        }
    }

    pub fn choose_submit(connection_config: Option<crate::common::ConnectionConfig>) -> Self {
        if connection_config.is_none() {
            return Submit::Display;
        }
        println!();

        let variants = SubmitDiscriminants::iter().collect::<Vec<_>>();
        let submits = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_submit = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("How would you like to proceed")
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
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        println!("Serialize_to_base64:\n{}", &serialize_to_base64);
        Ok(None)
    }

    pub async fn process_online(
        self,
        network_connection_config: crate::common::ConnectionConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            Submit::Send => {
                println!("Transaction sent ...");
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
                            match &err.data {
                                Some(serde_json::Value::String(data)) => {
                                    if data.contains("Timeout") {
                                        println!("Timeout error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
                                        continue;
                                    } else {
                                        println!("Error transaction: {}", data);
                                    }
                                }
                                Some(serde_json::Value::Object(err_data)) => {
                                    if let Some(tx_execution_error) = err_data
                                        .get("TxExecutionError")
                                        .and_then(|tx_execution_error_json| {
                                            serde_json::from_value(tx_execution_error_json.clone())
                                                .ok()
                                        })
                                    {
                                        crate::common::print_transaction_error(tx_execution_error);
                                    } else {
                                        println!("Unexpected response: {:#?}", err);
                                    }
                                }
                                _ => println!("Unexpected response: {:#?}", err),
                            }
                            return Ok(None);
                        }
                    };
                };
                Ok(Some(transaction_info))
            }
            Submit::Display => {
                println!("\nSerialize_to_base64:\n{}", &serialize_to_base64);
                Ok(None)
            }
        }
    }
}
