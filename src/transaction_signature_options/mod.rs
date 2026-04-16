use base64::Engine as _;
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
pub mod sign_with_mpc;
pub mod sign_with_private_key;
pub mod sign_with_seed_phrase;
pub mod submit_dao_proposal;

#[cfg(test)]
mod hash_compat_tests;

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
        message = "sign-with-mpc                    - Sign and send the transaction with MPC"
    ))]
    /// Sign and send the transaction with MPC
    SignWithMpc(crate::transaction_signature_options::sign_with_mpc::SignMpc),
    #[strum_discriminants(strum(
        message = "sign-later                       - Prepare an unsigned transaction to sign it later"
    ))]
    /// Prepare unsigned transaction to sign it later
    SignLater(self::sign_later::SignLater),
    #[strum_discriminants(strum(
        message = "submit-as-dao-proposal           - Convert current transaction to DAO proposal"
    ))]
    /// Prepare transaction as dao proposal
    SubmitAsDaoProposal(self::submit_dao_proposal::DaoProposal),
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
    pub public_key: near_kit::PublicKey,
    pub private_key: near_kit::SecretKey,
}

pub type OnBeforeSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &SignedTransactionOrSignedDelegateAction,
        &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<String>,
>;

pub type OnAfterSendingTransactionCallback = std::sync::Arc<
    dyn Fn(
        &near_kit::FinalExecutionOutcome,
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

#[derive(Debug, Clone)]
pub enum SignedTransactionOrSignedDelegateAction {
    SignedTransaction(near_kit::SignedTransaction),
    SignedDelegateAction(near_kit::SignedDelegateAction),
}

impl serde::Serialize for SignedTransactionOrSignedDelegateAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        match self {
            Self::SignedTransaction(tx) => {
                map.serialize_entry("SignedTransaction", &tx.to_base64())?;
            }
            Self::SignedDelegateAction(da) => {
                let b64 = base64::engine::general_purpose::STANDARD.encode(
                    &borsh::to_vec(da).expect("borsh serialization should not fail"),
                );
                map.serialize_entry("SignedDelegateAction", &b64)?;
            }
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for SignedTransactionOrSignedDelegateAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let map: std::collections::HashMap<String, String> =
            serde::Deserialize::deserialize(deserializer)?;
        if let Some(b64) = map.get("SignedTransaction") {
            let tx = near_kit::SignedTransaction::from_base64(b64)
                .map_err(serde::de::Error::custom)?;
            Ok(Self::SignedTransaction(tx))
        } else if let Some(b64) = map.get("SignedDelegateAction") {
            let bytes = base64::engine::general_purpose::STANDARD.decode(b64)
                .map_err(serde::de::Error::custom)?;
            let da: near_kit::SignedDelegateAction =
                borsh::from_slice(&bytes).map_err(serde::de::Error::custom)?;
            Ok(Self::SignedDelegateAction(da))
        } else {
            Err(serde::de::Error::custom(
                "expected SignedTransaction or SignedDelegateAction key",
            ))
        }
    }
}

impl From<near_kit::SignedTransaction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(signed_transaction: near_kit::SignedTransaction) -> Self {
        Self::SignedTransaction(signed_transaction)
    }
}

impl From<near_kit::SignedDelegateAction>
    for SignedTransactionOrSignedDelegateAction
{
    fn from(
        signed_delegate_action: near_kit::SignedDelegateAction,
    ) -> Self {
        Self::SignedDelegateAction(signed_delegate_action)
    }
}

pub fn get_signed_delegate_action(
    unsigned_transaction: near_kit::Transaction,
    public_key: &near_kit::PublicKey,
    private_key: near_kit::SecretKey,
    max_block_height: u64,
) -> near_kit::SignedDelegateAction {
    let delegate_action = near_kit::DelegateAction {
        sender_id: unsigned_transaction.signer_id.clone(),
        receiver_id: unsigned_transaction.receiver_id.clone(),
        actions: unsigned_transaction
            .actions
            .into_iter()
            .map(near_kit::NonDelegateAction::try_from)
            .collect::<Result<_, _>>()
            .expect("Internal error: can not convert the action to non delegate action (delegate action can not be delegated again)."),
        nonce: unsigned_transaction.nonce,
        max_block_height,
        public_key: unsigned_transaction.public_key.clone(),
    };

    let hash = delegate_action.get_hash();
    let signature = private_key.sign(hash.as_bytes());

    tracing::info!(
        parent: &tracing::Span::none(),
        "Your delegating action was signed successfully.{}",
        crate::common::indent_payload(&format!(
            "\nNote that the signed transaction is valid until block {max_block_height}. You can change the validity of a transaction by setting a flag in the command: --meta-transaction-valid-for 2000\nPublic key: {public_key}\nSignature:  {signature}\n"
        ))
    );

    delegate_action.sign(signature)
}
