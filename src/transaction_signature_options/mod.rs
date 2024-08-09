use serde::Deserialize;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod display;
pub mod save_to_file;
pub mod send;
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
        message = "sign-with-keychain               - Sign the transaction with a key saved in the secure keychain"
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
    SignLater(self::sign_later::SignLater),
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SubmitContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to proceed?
pub enum Submit {
    #[strum_discriminants(strum(
        message = "send             - Send the transaction to the network"
    ))]
    /// Send the transaction to the network
    Send(self::send::Send),
    #[strum_discriminants(strum(
        message = "save-to-file     - Save the signed transaction to file (if you want to send it later)"
    ))]
    /// Save the signed transaction to file (if you want to send it later)
    SaveToFile(self::save_to_file::SaveToFile),
    #[strum_discriminants(strum(
        message = "display          - Print the signed transaction to terminal (if you want to send it later)"
    ))]
    /// Print the signed transaction to terminal (if you want to send it later)
    Display(self::display::Display),
}

#[derive(Debug, Deserialize)]
pub struct AccountKeyPair {
    pub public_key: near_crypto::PublicKey,
    pub private_key: near_crypto::SecretKey,
}

pub type OnBeforeSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &SignedTransactionOrSignedDelegateAction,
        &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<String>,
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
    SignedDelegateAction(near_primitives::action::delegate::SignedDelegateAction),
}

impl From<near_primitives::transaction::SignedTransaction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(signed_transaction: near_primitives::transaction::SignedTransaction) -> Self {
        Self::SignedTransaction(signed_transaction)
    }
}

impl From<near_primitives::action::delegate::SignedDelegateAction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(
        signed_delegate_action: near_primitives::action::delegate::SignedDelegateAction,
    ) -> Self {
        Self::SignedDelegateAction(signed_delegate_action)
    }
}

pub fn get_signed_delegate_action(
    unsigned_transaction: near_primitives::transaction::Transaction,
    public_key: &near_crypto::PublicKey,
    private_key: near_crypto::SecretKey,
    max_block_height: u64,
) -> near_primitives::action::delegate::SignedDelegateAction {
    use near_primitives::signable_message::{SignableMessage, SignableMessageType};

    let mut delegate_action = near_primitives::action::delegate::DelegateAction {
        sender_id: unsigned_transaction.signer_id().clone(),
        receiver_id: unsigned_transaction.receiver_id().clone(),
        actions: vec![],
        nonce: unsigned_transaction.nonce(),
        max_block_height,
        public_key: unsigned_transaction.public_key().clone(),
    };

    delegate_action.actions = unsigned_transaction
        .take_actions()
        .into_iter()
        .map(near_primitives::action::delegate::NonDelegateAction::try_from)
        .collect::<Result<_, _>>()
        .expect("Internal error: can not convert the action to non delegate action (delegate action can not be delegated again).");

    // create a new signature here signing the delegate action + discriminant
    let signable = SignableMessage::new(&delegate_action, SignableMessageType::DelegateAction);
    let signer = near_crypto::InMemorySigner::from_secret_key(
        delegate_action.sender_id.clone(),
        private_key,
    );
    let signature = signable.sign(&near_crypto::Signer::InMemory(signer));

    eprintln!("\nYour delegating action was signed successfully.");
    eprintln!("Note that the signed transaction is valid until block {max_block_height}. You can change the validity of a transaction by setting a flag in the command: --meta-transaction-valid-for 2000");
    eprintln!("Public key: {}", public_key);
    eprintln!("Signature: {}", signature);

    near_primitives::action::delegate::SignedDelegateAction {
        delegate_action,
        signature,
    }
}
