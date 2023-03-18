use near_primitives::borsh::BorshSerialize;
use serde::Deserialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod sign_with_access_key_file;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
#[cfg(target_os = "macos")]
pub mod sign_with_macos_keychain;
pub mod sign_with_private_key;
pub mod sign_with_seed_phrase;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a tool for signing the transaction
pub enum SignWith {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "sign-with-macos-keychain         - Sign the transaction with a key saved in macOS keychain"
    ))]
    /// Sign the transaction with a key saved in macOS keychain
    SignWithMacosKeychain(self::sign_with_macos_keychain::SignMacosKeychain),
    #[strum_discriminants(strum(
        message = "sign-with-keychain               - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)"
    ))]
    /// Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "sign-with-ledger                 - Sign the transaction with Ledger Nano device"
    ))]
    /// Sign the transaction with Ledger Nano device
    SignWithLedger(self::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "sign-with-plaintext-private-key  - Sign the transaction with a plaintext private key"
    ))]
    /// Sign the transaction with a plaintext private key
    SignWithPlaintextPrivateKey(self::sign_with_private_key::SignPrivateKey),
    #[strum_discriminants(strum(
        message = "sign-with-access-key-file        - Sign the transaction using the account access key file (access-key-file.json)"
    ))]
    /// Sign the transaction using the account access key file (access-key-file.json)
    SignWithAccessKeyFile(self::sign_with_access_key_file::SignAccessKeyFile),
    #[strum_discriminants(strum(
        message = "sign-with-seed-phrase            - Sign the transaction using the seed phrase"
    ))]
    /// Sign the transaction using the seed phrase
    SignWithSeedPhrase(self::sign_with_seed_phrase::SignSeedPhrase),
}

// from_cli ...
//         println!("\nUnsigned transaction:\n");
//         crate::common::print_unsigned_transaction(new_context.transaction.clone().into());
//         println!();

pub async fn sign_with(
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
    _prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    _config: crate::config::Config,
) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
    match network_config.get_sign_option() {
        #[cfg(target_os = "macos")]
        SignWith::SignWithMacosKeychain(_) => Ok(None),
        SignWith::SignWithKeychain(_) => Ok(None),
        #[cfg(feature = "ledger")]
        SignWith::SignWithLedger(_) => Ok(None),
        SignWith::SignWithPlaintextPrivateKey(_) => Ok(None),
        SignWith::SignWithAccessKeyFile(_) => Ok(None),
        SignWith::SignWithSeedPhrase(_) => Ok(None),
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

impl interactive_clap::FromCli for Submit {
    type FromCliContext = SubmitContext;
    type FromCliError = color_eyre::eyre::Error;
    fn from_cli(
        mut optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    >
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let mut storage_message = String::new();

        if optional_clap_variant.is_none() {
            match Self::choose_variant(context.clone()) {
                interactive_clap::ResultFromCli::Ok(cli_submit) => {
                    optional_clap_variant = Some(cli_submit)
                }
                interactive_clap::ResultFromCli::Back => {
                    return interactive_clap::ResultFromCli::Back
                }
                interactive_clap::ResultFromCli::Cancel(optional_cli_submit) => {
                    return interactive_clap::ResultFromCli::Cancel(optional_cli_submit)
                }
                interactive_clap::ResultFromCli::Err(optional_cli_submit, err) => {
                    return interactive_clap::ResultFromCli::Err(optional_cli_submit, err)
                }
            }
        }

        match optional_clap_variant {
            Some(CliSubmit::Send) => {
                match (context.on_before_sending_transaction_callback)(
                    &context.signed_transaction,
                    &context.network_config,
                    &mut storage_message,
                ) {
                    Ok(_) => (),
                    Err(report) => {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        )
                    }
                };

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
                            Err(report) => {
                                return interactive_clap::ResultFromCli::Err(
                                    optional_clap_variant,
                                    color_eyre::Report::msg(report),
                                )
                            }
                        },
                    };
                };
                match crate::common::print_transaction_status(
                    &transaction_info,
                    &context.network_config,
                ) {
                    Ok(_) => (),
                    Err(report) => {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        )
                    }
                };
                match (context.on_after_sending_transaction_callback)(
                    &transaction_info,
                    &context.network_config,
                ) {
                    Ok(_) => (),
                    Err(report) => {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        )
                    }
                };
                println!("{storage_message}");
                interactive_clap::ResultFromCli::Ok(CliSubmit::Send)
            }
            Some(CliSubmit::Display) => {
                match (context.on_before_sending_transaction_callback)(
                    &context.signed_transaction,
                    &context.network_config,
                    &mut storage_message,
                ) {
                    Ok(_) => (),
                    Err(report) => {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        )
                    }
                };
                let base64_transaction = near_primitives::serialize::to_base64(
                    context
                        .signed_transaction
                        .try_to_vec()
                        .expect("Transaction is not expected to fail on serialization"),
                );
                println!("\nSerialize_to_base64:\n{}", &base64_transaction);

                println!("{storage_message}");
                interactive_clap::ResultFromCli::Ok(CliSubmit::Display)
            }
            None => unreachable!("Unexpected error"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AccountKeyPair {
    pub public_key: near_crypto::PublicKey,
    pub private_key: near_crypto::SecretKey,
}

pub type OnBeforeSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &near_primitives::transaction::SignedTransaction,
        &crate::config::NetworkConfig,
        &mut String,
    ) -> crate::CliResult,
>;

pub type OnAfterSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &near_primitives::views::FinalExecutionOutcomeView,
        &crate::config::NetworkConfig,
    ) -> crate::CliResult,
>;

#[derive(Clone)]
pub struct SubmitContext {
    pub network_config: crate::config::NetworkConfig,
    pub signed_transaction: near_primitives::transaction::SignedTransaction,
    pub on_before_sending_transaction_callback: OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback: OnAfterSendingTransactionCallback,
}
