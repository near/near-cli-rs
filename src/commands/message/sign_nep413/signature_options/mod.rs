use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod sign_with_access_key_file;
pub mod sign_with_keychain;
#[cfg(feature = "ledger")]
pub mod sign_with_ledger;
pub mod sign_with_legacy_keychain;
pub mod sign_with_private_key;
pub mod sign_with_seed_phrase;

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::SignNep413Context)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you want to sign the message?
pub enum SignWith {
    #[strum_discriminants(strum(
        message = "sign-with-keychain               - Sign with a key saved in the secure keychain"
    ))]
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "sign-with-legacy-keychain        - Sign with a key saved in legacy keychain (compatible with the old near CLI)"
    ))]
    SignWithLegacyKeychain(self::sign_with_legacy_keychain::SignLegacyKeychain),
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "sign-with-ledger                 - Sign with Ledger Nano device"
    ))]
    SignWithLedger(self::sign_with_ledger::SignLedger),
    #[strum_discriminants(strum(
        message = "sign-with-plaintext-private-key  - Sign with a plaintext private key"
    ))]
    SignWithPlaintextPrivateKey(self::sign_with_private_key::SignPrivateKey),
    #[strum_discriminants(strum(
        message = "sign-with-access-key-file        - Sign using an account access key file"
    ))]
    SignWithAccessKeyFile(self::sign_with_access_key_file::SignAccessKeyFile),
    #[strum_discriminants(strum(
        message = "sign-with-seed-phrase            - Sign using a seed phrase"
    ))]
    SignWithSeedPhrase(self::sign_with_seed_phrase::SignSeedPhrase),
}
