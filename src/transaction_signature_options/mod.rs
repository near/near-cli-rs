use serde::Deserialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::JsonRpcClientExt;

pub mod sign_later;
pub mod sign_with_access_key_file;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
pub mod sign_with_legacy_keychain;
pub mod sign_with_private_key;
pub mod sign_with_seed_phrase;

pub const META_TRANSACTION_VALID_FOR_DEFAULT: u64 = 1000;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a tool for signing the transaction:
pub enum SignWith {
    #[strum_discriminants(strum(
        message = "sign-with-keychain               - Sign the transaction with a key saved in the keychain (backwards compatible with the old near CLI)"
    ))]
    /// Sign the transaction with a key saved in keychain
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "sign-with-legacy-keychain        - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)"
    ))]
    /// Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)
    SignWithLegacyKeychain(self::sign_with_legacy_keychain::SignLegacyKeychain),
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
    #[strum_discriminants(strum(
        message = "sign-later                       - Prepare an unsigned transaction to sign it later"
    ))]
    /// Prepare unsigned transaction to sign it later
    SignLater(self::sign_later::Display),
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SubmitContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(skip_default_from_cli)]
/// How would you like to proceed?
pub enum Submit {
    #[strum_discriminants(strum(message = "send      - Send the transaction to the network"))]
    /// Send the transaction to the network
    Send,
    #[strum_discriminants(strum(
        message = "display   - Print the signed transaction to terminal (if you want to send it later)"
    ))]
    /// Print the signed transaction to terminal (if you want to send it later)
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
                result => return result,
            }
        }

        match optional_clap_variant {
            Some(CliSubmit::Send) => match context.signed_transaction_or_signed_delegate_action {
                SignedTransactionOrSignedDelegateAction::SignedTransaction(signed_transaction) => {
                    if let Err(report) = (context.on_before_sending_transaction_callback)(
                        &signed_transaction,
                        &context.network_config,
                        &mut storage_message,
                    ) {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        );
                    };

                    eprintln!("Transaction sent ...");
                    let transaction_info = loop {
                        let transaction_info_result = context.network_config.json_rpc_client()
                        .blocking_call(
                            near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest{
                                signed_transaction: signed_transaction.clone()
                            }
                        );
                        match transaction_info_result {
                            Ok(response) => {
                                break response;
                            }
                            Err(err) => match crate::common::rpc_transaction_error(err) {
                                Ok(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
                                Err(report) => {
                                    return interactive_clap::ResultFromCli::Err(
                                        optional_clap_variant,
                                        color_eyre::Report::msg(report),
                                    )
                                }
                            },
                        };
                    };
                    if let Err(report) = crate::common::print_transaction_status(
                        &transaction_info,
                        &context.network_config,
                    ) {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        );
                    };
                    if let Err(report) = (context.on_after_sending_transaction_callback)(
                        &transaction_info,
                        &context.network_config,
                    ) {
                        return interactive_clap::ResultFromCli::Err(
                            optional_clap_variant,
                            color_eyre::Report::msg(report),
                        );
                    };
                    eprintln!("{storage_message}");
                    interactive_clap::ResultFromCli::Ok(CliSubmit::Send)
                }
                SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                    signed_delegate_action,
                ) => {
                    let client = reqwest::blocking::Client::new();
                    let json_payload = serde_json::json!({
                        "signed_delegate_action": crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
                            signed_delegate_action
                        ).to_string()
                    });
                    match client
                        .post(
                            context
                                .network_config
                                .meta_transaction_relayer_url
                                .expect("Internal error: Meta-transaction relayer URL must be Some() at this point"),
                        )
                        .json(&json_payload)
                        .send()
                    {
                        Ok(relayer_response) => {
                            if relayer_response.status().is_success() {
                                let response_text = match relayer_response.text() {
                                    Ok(text) => text,
                                    Err(report) => {
                                        return interactive_clap::ResultFromCli::Err(
                                            optional_clap_variant,
                                            color_eyre::Report::msg(report),
                                        )
                                    }
                                };
                                println!("Relayer Response text: {}", response_text);
                            } else {
                                println!(
                                    "Request failed with status code: {}",
                                    relayer_response.status()
                                );
                            }
                        }
                        Err(report) => {
                            return interactive_clap::ResultFromCli::Err(
                                optional_clap_variant,
                                color_eyre::Report::msg(report),
                            )
                        }
                    }
                    eprintln!("{storage_message}");
                    interactive_clap::ResultFromCli::Ok(CliSubmit::Send)
                }
            },
            Some(CliSubmit::Display) => {
                match context.signed_transaction_or_signed_delegate_action {
                    SignedTransactionOrSignedDelegateAction::SignedTransaction(
                        signed_transaction,
                    ) => {
                        if let Err(report) = (context.on_before_sending_transaction_callback)(
                            &signed_transaction,
                            &context.network_config,
                            &mut storage_message,
                        ) {
                            return interactive_clap::ResultFromCli::Err(
                                optional_clap_variant,
                                color_eyre::Report::msg(report),
                            );
                        };
                        eprintln!(
                            "\nSigned transaction (serialized as base64):\n{}\n",
                            crate::types::signed_transaction::SignedTransactionAsBase64::from(
                                signed_transaction
                            )
                        );
                        eprintln!(
                            "This base64-encoded signed transaction is ready to be sent to the network. You can call RPC server directly, or use a helper command on near CLI:\n$ {} transaction send-signed-transaction\n",
                            crate::common::get_near_exec_path()
                        );
                        eprintln!("{storage_message}");
                        interactive_clap::ResultFromCli::Ok(CliSubmit::Display)
                    }
                    SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                        signed_delegate_action,
                    ) => {
                        eprintln!(
                            "\nSigned delegate action (serialized as base64):\n{}\n",
                            crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
                                signed_delegate_action
                            )
                        );
                        eprintln!(
                            "This base64-encoded signed delegate action is ready to be sent to the meta-transaction relayer. There is a helper command on near CLI that can do that:\n$ {} transaction send-meta-transaction\n",
                            crate::common::get_near_exec_path()
                        );
                        eprintln!("{storage_message}");
                        interactive_clap::ResultFromCli::Ok(CliSubmit::Display)
                    }
                }
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
    pub global_context: crate::GlobalContext,
    pub signed_transaction_or_signed_delegate_action: SignedTransactionOrSignedDelegateAction,
    pub on_before_sending_transaction_callback: OnBeforeSendingTransactionCallback,
    pub on_after_sending_transaction_callback: OnAfterSendingTransactionCallback,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SignedTransactionOrSignedDelegateAction {
    SignedTransaction(near_primitives::transaction::SignedTransaction),
    SignedDelegateAction(near_primitives::delegate_action::SignedDelegateAction),
}

impl From<near_primitives::transaction::SignedTransaction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(signed_transaction: near_primitives::transaction::SignedTransaction) -> Self {
        Self::SignedTransaction(signed_transaction)
    }
}

impl From<near_primitives::delegate_action::SignedDelegateAction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(
        signed_delegate_action: near_primitives::delegate_action::SignedDelegateAction,
    ) -> Self {
        Self::SignedDelegateAction(signed_delegate_action)
    }
}

pub fn get_signed_delegate_action(
    unsigned_transaction: near_primitives::transaction::Transaction,
    public_key: &near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
    max_block_height: u64,
) -> near_primitives::delegate_action::SignedDelegateAction {
    use near_primitives::signable_message::{SignableMessage, SignableMessageType};

    let actions = unsigned_transaction
        .actions
        .into_iter()
        .map(near_primitives::delegate_action::NonDelegateAction::try_from)
        .collect::<Result<_, _>>()
        .expect("Internal error: can not convert the action to non delegate action (delegate action can not be delegated again).");
    let delegate_action = near_primitives::delegate_action::DelegateAction {
        sender_id: unsigned_transaction.signer_id.clone(),
        receiver_id: unsigned_transaction.receiver_id,
        actions,
        nonce: unsigned_transaction.nonce,
        max_block_height,
        public_key: unsigned_transaction.public_key,
    };

    // create a new signature here signing the delegate action + discriminant
    let signable = SignableMessage::new(&delegate_action, SignableMessageType::DelegateAction);
    let signer =
        near_crypto::InMemorySigner::from_secret_key(unsigned_transaction.signer_id, private_key);
    let signature = signable.sign(&signer);

    eprintln!("\nYour delegating action was signed successfully.");
    eprintln!("Note that the signed transaction is valid until block {max_block_height}. You can change the validity of a transaction by setting a flag in the command: --meta-transaction-valid-for 2000");
    eprintln!("Public key: {}", public_key);
    eprintln!("Signature: {}", signature);

    near_primitives::delegate_action::SignedDelegateAction {
        delegate_action,
        signature,
    }
}
