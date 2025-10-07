use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a tool for signing transaction to send to MPC:
pub enum MpcSignWith {
    #[strum_discriminants(strum(
        message = "sign-with-keychain               - Sign the transaction with a key saved in the secure keychain"
    ))]
    /// Sign the transaction with a key saved in keychain
    SignWithKeychain(crate::transaction_signature_options::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "sign-with-legacy-keychain        - Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)"
    ))]
    /// Sign the transaction with a key saved in legacy keychain (compatible with the old near CLI)
    SignWithLegacyKeychain(
        crate::transaction_signature_options::sign_with_legacy_keychain::SignLegacyKeychain,
    ),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "sign-with-ledger                 - Sign the transaction with Ledger Nano device"
    ))]
    /// Sign the transaction with Ledger Nano device
    SignWithLedger(crate::transaction_signature_options::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "sign-with-plaintext-private-key  - Sign the transaction with a plaintext private key"
    ))]
    /// Sign the transaction with a plaintext private key
    SignWithPlaintextPrivateKey(
        crate::transaction_signature_options::sign_with_private_key::SignPrivateKey,
    ),
    #[strum_discriminants(strum(
        message = "sign-with-access-key-file        - Sign the transaction using the account access key file (access-key-file.json)"
    ))]
    /// Sign the transaction using the account access key file (access-key-file.json)
    SignWithAccessKeyFile(
        crate::transaction_signature_options::sign_with_access_key_file::SignAccessKeyFile,
    ),
    #[strum_discriminants(strum(
        message = "sign-with-seed-phrase            - Sign the transaction using the seed phrase"
    ))]
    /// Sign the transaction using the seed phrase
    SignWithSeedPhrase(crate::transaction_signature_options::sign_with_seed_phrase::SignSeedPhrase),
    #[strum_discriminants(strum(
        message = "sign-later                       - Prepare an unsigned transaction to sign it later"
    ))]
    /// Prepare unsigned transaction to sign it later
    SignLater(crate::transaction_signature_options::sign_later::SignLater),
    #[strum_discriminants(strum(
        message = "submit-as-dao-proposal           - Convert current transaction to DAO proposal"
    ))]
    /// Prepare transaction as dao proposal
    SubmitAsDaoProposal(crate::transaction_signature_options::submit_dao_proposal::DaoProposal),
}
