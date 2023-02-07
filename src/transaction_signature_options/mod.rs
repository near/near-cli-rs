use std::collections::HashMap;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use either::Either;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
#[cfg(target_os = "macos")]
pub mod sign_with_macos_keychain;
pub mod sign_with_private_key;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
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
}

pub fn input_signer_public_key() -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
    Ok(Input::new()
        .with_prompt("Enter sender (signer) public key")
        .interact_text()?)
}

pub fn input_signer_private_key() -> color_eyre::eyre::Result<crate::types::secret_key::SecretKey> {
    Ok(Input::new()
        .with_prompt("Enter sender (signer) private (secret) key")
        .interact_text()?)
}

pub async fn sign_with(
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
    prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    config: crate::config::Config,
) -> color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
    match network_config.get_sign_option() {
        #[cfg(target_os = "macos")]
        SignWith::SignWithMacosKeychain(sign_macos_keychain) => {
            sign_macos_keychain
                .process(
                    prepopulated_unsigned_transaction,
                    network_config.get_network_config(config),
                )
                .await
        }
        SignWith::SignWithKeychain(sign_keychain) => {
            sign_keychain
                .process(
                    prepopulated_unsigned_transaction,
                    network_config.get_network_config(config.clone()),
                    config.credentials_home_dir,
                )
                .await
        }
        #[cfg(feature = "ledger")]
        SignWith::SignWithLedger(sign_ledger) => {
            sign_ledger
                .process(
                    prepopulated_unsigned_transaction,
                    network_config.get_network_config(config),
                )
                .await
        }
        SignWith::SignWithPlaintextPrivateKey(sign_private_key) => {
            sign_private_key
                .process(
                    prepopulated_unsigned_transaction,
                    network_config.get_network_config(config),
                )
                .await
        }
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

#[derive(Debug, EnumDiscriminants, Clone, clap::Parser, interactive_clap::ToCliArgs)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Submit {
    #[strum_discriminants(strum(message = "send      - Send the transaction to the network"))]
    Send,
    #[strum_discriminants(strum(message = "send via relay     - Send to a relayer"))]
    SendViaRelay,
    #[strum_discriminants(strum(
        message = "display   - Print only base64 encoded transaction for JSON RPC input and exit"
    ))]
    Display,
}

impl interactive_clap::ToCli for Submit {
    type CliVariant = Submit;
}

impl Submit {
    pub fn choose_submit() -> Self {
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

    pub async fn process(
        &self,
        network_config: crate::config::NetworkConfig,
        signed_transaction: near_primitives::transaction::SignedTransaction,
        serialize_to_base64: String,
    ) -> Either<color_eyre::eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>>, reqwest::Result<T>> {
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
            Submit::SendViaRelay => {
                // TODO relayer type and info validation
                let relayer = Ok(Input::new()
                    .with_prompt("Enter relayer endpoint (ie http://relayer.near.org:3030/relay)")
                    .interact_text()?
                );
                // create signed delegate action and send to relayer
                // fill in params from https://github.com/near/nearcore/pull/7497/files#diff-90dfa190ec8dff070747d21fd42e25f6022268a7d008ae1e00c0dd5ada2e5bd2R247
                let max_block_height = signed_transaction.transaction.block_hash + 100;  // TODO is 100 blocks appropriate? - also get current block height instead of hash
                let delegate_action = near_primitives_01::transaction::DelegateAction(
                    signed_transaction.transaction.signer_id,
                    signed_transaction.transaction.receiver_id,
                    signed_transaction.transaction.actions,
                    signed_transaction.transaction.nonce,
                    max_block_height,
                    signed_transaction.transaction.public_key
                );
                let signed_delegate_action = near_primitives_01::transaction::SignedDelegateAction(
                    delegate_action,
                    signed_transaction.signature
                );
                // send signed_delegate_action to relayer via a POST request
                println!("Sending transaction to relayer ...");
                let client = reqwest::Client::new();
                let mut payload = HashMap::new();
                payload.insert("signed_delegate_action", signed_delegate_action);
                let relayer_response = client.post(relayer)
                    .json(&payload)  // serialize signed_delegate_action to json
                    .send()
                    .await?;
                Ok(Some(relayer_response))
            }
            Submit::Display => {
                println!("\nSerialize_to_base64:\n{}", &serialize_to_base64);
                Ok(None)
            }
        }
    }
}
