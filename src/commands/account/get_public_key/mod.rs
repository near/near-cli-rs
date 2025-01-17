use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod from_keychain;
#[cfg(feature = "ledger")]
mod from_ledger;
mod from_legacy_keychain;
mod from_seed_phrase;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct GetPublicKey {
    #[interactive_clap(subcommand)]
    get_public_key_mode: GetPublicKeyMode,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Where do you want to get the public key from?
pub enum GetPublicKeyMode {
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(
        message = "from-ledger                 - Get the public key stored on your Ledger Nano device"
    ))]
    /// Get the public key stored on your Ledger Nano device
    FromLedger(self::from_ledger::PublicKeyFromLedger),
    #[strum_discriminants(strum(
        message = "from-seed-phrase            - Get the public key with the seed phrase"
    ))]
    /// Get the public key with the seed phrase
    FromSeedPhrase(self::from_seed_phrase::PublicKeyFromSeedPhrase),
    #[strum_discriminants(strum(
        message = "from-keychain               - Get the public key stored in a secure keychain"
    ))]
    /// Get the public key (full access key) stored in a secure keychain
    FromKeychain(self::from_keychain::PublicKeyFromKeychain),
    #[strum_discriminants(strum(
        message = "from-legacy-keychain        - Get the public key stored in the legacy keychain (compatible with the old near CLI)"
    ))]
    /// Get the public key (full access key) stored in the legacy keychain (compatible with the old near CLI)
    FromLegacyKeychain(self::from_legacy_keychain::PublicKeyFromKeychain),
}
