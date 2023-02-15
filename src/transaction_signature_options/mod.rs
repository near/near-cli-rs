use inquire::{CustomType, Select};
use near_primitives::borsh::BorshSerialize;
use serde::Deserialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

// pub mod sign_with_access_key_file;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
// pub mod sign_with_ledger;
// #[cfg(target_os = "macos")]
// pub mod sign_with_macos_keychain;
pub mod sign_with_private_key;
// pub mod sign_with_seed_phrase;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a tool for signing the transaction
pub enum SignWith {
    // #[cfg(target_os = "macos")]
    // #[strum_discriminants(strum(
    //     message = "sign-with-macos-keychain         - Sign the transaction with a key saved in macOS keychain"
    // ))]
    // /// Sign the transaction with a key saved in macOS keychain
    // SignWithMacosKeychain(self::sign_with_macos_keychain::SignMacosKeychain),
    #[strum_discriminants(strum(
        message = "sign-with-keychain               - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)"
    ))]
    /// Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    // #[cfg(feature = "ledger")]
    // #[strum_discriminants(strum(
    //     message = "sign-with-ledger                 - Sign the transaction with Ledger Nano device"
    // ))]
    // /// Sign the transaction with Ledger Nano device
    // SignWithLedger(self::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "sign-with-plaintext-private-key  - Sign the transaction with a plaintext private key"
    ))]
    /// Sign the transaction with a plaintext private key
    SignWithPlaintextPrivateKey(self::sign_with_private_key::SignPrivateKey),
    // #[strum_discriminants(strum(
    //     message = "sign-with-access-key-file        - Sign the transaction using the account access key file (access-key-file.json)"
    // ))]
    // /// Sign the transaction using the account access key file (access-key-file.json)
    // SignWithAccessKeyFile(self::sign_with_access_key_file::SignAccessKeyFile),
    // #[strum_discriminants(strum(
    //     message = "sign-with-seed-phrase            - Sign the transaction using the seed phrase"
    // ))]
    // /// Sign the transaction using the seed phrase
    // SignWithSeedPhrase(self::sign_with_seed_phrase::SignSeedPhrase),
}

// from_cli ...
//         println!("\nUnsigned transaction:\n");
//         crate::common::print_unsigned_transaction(new_context.transaction.clone().into());
//         println!();

pub fn input_signer_public_key() -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
    Ok(CustomType::new("Enter sender (signer) public key").prompt()?)
}

pub fn input_signer_private_key() -> color_eyre::eyre::Result<crate::types::secret_key::SecretKey> {
    Ok(CustomType::new("Enter sender (signer) private (secret) key").prompt()?)
}

pub async fn sign_with(
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
    prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    config: crate::config::Config,
) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
    match network_config.get_sign_option() {
        // #[cfg(target_os = "macos")]
        // SignWith::SignWithMacosKeychain(sign_macos_keychain) => {
        //     sign_macos_keychain
        //         .process(
        //             prepopulated_unsigned_transaction,
        //             network_config.get_network_config(config),
        //         )
        //         .await
        // }
        SignWith::SignWithKeychain(_) => Ok(None),
        // #[cfg(feature = "ledger")]
        // SignWith::SignWithLedger(sign_ledger) => {
        //     sign_ledger
        //         .process(
        //             prepopulated_unsigned_transaction,
        //             network_config.get_network_config(config),
        //         )
        //         .await
        // }
        SignWith::SignWithPlaintextPrivateKey(_) => Ok(None),
        // SignWith::SignWithAccessKeyFile(sign_access_key_file) => {
        //     sign_access_key_file
        //         .process(
        //             prepopulated_unsigned_transaction,
        //             network_config.get_network_config(config),
        //         )
        //         .await
        // }
        // SignWith::SignWithSeedPhrase(sign_seed_phrase) => {
        //     sign_seed_phrase
        //         .process(
        //             prepopulated_unsigned_transaction,
        //             network_config.get_network_config(config),
        //         )
        //         .await
        // }
    }
}
//-----------------------------------------------------------------------------------
//---- these functions are used for offline mode ----
// pub fn input_access_key_nonce(public_key: &str) -> color_eyre::eyre::Result<u64> {
//     println!("Your public key: `{}`", public_key);
//     Ok(Input::new()
//         .with_prompt(
//             "Enter transaction nonce for this public key (query the access key information with \
//             `./near-cli view nonce \
//                 network testnet \
//                 account 'volodymyr.testnet' \
//                 public-key ed25519:...` incremented by 1)",
//         )
//         .interact_text()?)
// }

// pub fn input_block_hash() -> color_eyre::eyre::Result<crate::types::crypto_hash::CryptoHash> {
//     let input_block_hash: crate::common::BlockHashAsBase58 = Input::new()
//         .with_prompt(
//             "Enter recent block hash (query information about the hash of the last block with \
//             `./near-cli view recent-block-hash network testnet`)",
//         )
//         .interact_text()?;
//     Ok(crate::types::crypto_hash::CryptoHash(
//         input_block_hash.inner,
//     ))
// }
//-----------------------------------------------------------------------------------

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SubmitContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(skip_default_from_cli)]
/// How would you like to proceed
pub enum Submit {
    #[strum_discriminants(strum(message = "send      - Send the transaction to the network"))]
    Send,
    #[strum_discriminants(strum(
        message = "display   - Print only base64 encoded transaction for JSON RPC input and exit"
    ))]
    Display,
}

// impl interactive_clap::ToCli for Submit {
//     type CliVariant = Submit;
// }

impl interactive_clap::FromCli for Submit {
    type FromCliContext = SubmitContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Submit>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        match optional_clap_variant {
            Some(CliSubmit::Send) => {
                println!("Transaction sent ...");
                let transaction_info = loop {
                    let transaction_info_result = tokio::runtime::Runtime::new()
                    .unwrap()
                    .block_on(context.network_config.json_rpc_client()
                        .call(near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest{signed_transaction: context.signed_transaction.clone()})
                        )
                    ;
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(err) => match crate::common::rpc_transaction_error(err) {
                            Ok(_) => tokio::runtime::Runtime::new().unwrap().block_on(
                                tokio::time::sleep(std::time::Duration::from_millis(100)),
                            ),
                            Err(report) => return color_eyre::eyre::Result::Err(report),
                        },
                    };
                };

                crate::common::print_transaction_status(transaction_info, context.network_config)?;
                // Ok(Some(transaction_info))

                Ok(Some(Self::Send))
            }
            Some(CliSubmit::Display) => {
                let base64_transaction = near_primitives::serialize::to_base64(
                    context
                        .signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!("\nSerialize_to_base64:\n{}", &base64_transaction);
                Ok(Some(Self::Display))
                // Ok(None)
            }
            None => Self::choose_variant(context.clone()),
        }
    }
}

// impl std::fmt::Display for SubmitDiscriminants {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             Self::Send => write!(f, "send"),
//             Self::Display => write!(f, "display"),
//         }
//     }
// }

impl Submit {
    // pub fn choose_submit() -> Self {
    //     let variants = SubmitDiscriminants::iter().collect::<Vec<_>>();
    //     let select_submit = Select::new("How would you like to proceed", variants)
    //         .prompt()
    //         .unwrap_or(SubmitDiscriminants::Display);
    //     match select_submit {
    //         SubmitDiscriminants::Send => Submit::Send,
    //         SubmitDiscriminants::Display => Submit::Display,
    //     }
    // }

    pub async fn process(
        &self,
        network_config: crate::config::NetworkConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
        match self {
            Submit::Send => {
                println!("Transaction sent ...");
                let transaction_info = loop {
                    let transaction_info_result = network_config.json_rpc_client()
                        .call(near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest{signed_transaction: signed_transaction.clone()})
                        .await;
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(err) => match crate::common::rpc_transaction_error(err) {
                            Ok(_) => {
                                tokio::time::sleep(std::time::Duration::from_millis(100)).await
                            }
                            Err(report) => return color_eyre::eyre::Result::Err(report),
                        },
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

#[derive(Debug, Deserialize)]
pub struct AccountKeyPair {
    pub public_key: near_crypto::PublicKey,
    pub private_key: near_crypto::SecretKey,
}

#[derive(Debug, Clone)]
pub struct SubmitContext {
    pub network_config: crate::config::NetworkConfig,
    pub signed_transaction: near_primitives::transaction::SignedTransaction,
    // pub base64_transaction: String,
}
