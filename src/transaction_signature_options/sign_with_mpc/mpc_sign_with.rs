use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select a tool for signing transaction to send to MPC:
pub enum MpcSignWith {
    #[strum_discriminants(strum(
        message = "sign-mpc-tx-with-keychain                - Sign MPC contract call a key saved in the secure keychain"
    ))]
    /// Sign MPC contract call with a key saved in keychain
    SignMpcWithKeychain(crate::transaction_signature_options::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "sign-mpc-with-legacy-keychain            - Sign MPC contract call with a key saved in keychain (compatible with the old near CLI)"
    ))]
    /// Sign MPC contract call with a key saved in legacy keychain (compatible with the old near CLI)
    SignMpcWithLegacyKeychain(
        crate::transaction_signature_options::sign_with_legacy_keychain::SignLegacyKeychain,
    ),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "sign-mpc-tx-with-ledger                  - Sign MPC contract call with Ledger Nano device"
    ))]
    /// Sign MPC contract call with Ledger Nano device
    SignMpcWithLedger(crate::transaction_signature_options::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "sign-mpc-tx-with-plaintext-private-key   - Sign MPC contract call with a plaintext private key"
    ))]
    /// Sign MPC contract call with a plaintext private key
    SignMpcWithPlaintextPrivateKey(
        crate::transaction_signature_options::sign_with_private_key::SignPrivateKey,
    ),
    #[strum_discriminants(strum(
        message = "sign-mpc-tx-with-access-key-file         - Sign MPC contract call using the account access key file (access-key-file.json)"
    ))]
    /// Sign MPC contract call using the account access key file (access-key-file.json)
    SignMpcWithAccessKeyFile(
        crate::transaction_signature_options::sign_with_access_key_file::SignAccessKeyFile,
    ),
    #[strum_discriminants(strum(
        message = "sign-mpc-tx-with-seed-phrase             - Sign MPC contract call using the seed phrase"
    ))]
    /// Sign MPC contract call using the seed phrase
    SignMpcWithSeedPhrase(
        crate::transaction_signature_options::sign_with_seed_phrase::SignSeedPhrase,
    ),
    #[strum_discriminants(strum(
        message = "submit-mpc-as-dao-proposal               - Convert current MPC transaction to DAO proposal"
    ))]
    /// Prepare MPC transaction as dao proposal
    SubmitMpcAsDaoProposal(crate::transaction_signature_options::submit_dao_proposal::DaoProposal),
}
