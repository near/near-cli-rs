use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod autogenerate_new_keypair;
// #[cfg(feature = "ledger")]
// mod use_ledger;
mod use_manually_provided_seed_phrase;
mod use_public_key;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::NewAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Add an access key for this account
pub enum AccessKeyMode {
    #[strum_discriminants(strum(
        message = "autogenerate-new-keypair          - Automatically generate a key pair"
    ))]
    /// Automatically generate a key pair
    AutogenerateNewKeypair(self::autogenerate_new_keypair::GenerateKeypair),
    #[strum_discriminants(strum(
        message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
    ))]
    /// Use the provided seed phrase manually
    UseManuallyProvidedSeedPhrase(
        self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
    ),
    #[strum_discriminants(strum(
        message = "use-manually-provided-public-key  - Use the provided public key manually"
    ))]
    /// Use the provided public key manually
    UseManuallyProvidedPublicKey(self::use_public_key::AddPublicKeyAction),
    // #[cfg(feature = "ledger")]
    // #[strum_discriminants(strum(message = "use-ledger                        - Use a ledger"))]
    // /// Use a ledger
    // UseLedger(self::use_ledger::AddAccessWithLedger),
}
