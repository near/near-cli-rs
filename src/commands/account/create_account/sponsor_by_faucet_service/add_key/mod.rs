use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod autogenerate_new_keypair;
// #[cfg(feature = "ledger")]
// mod use_ledger;
// mod use_manually_provided_seed_phrase;
mod use_public_key;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Add an access key for this account
pub enum AccessKeyMode {
    #[strum_discriminants(strum(
        message = "autogenerate-new-keypair          - Automatically generate a key pair"
    ))]
    ///Automatically generate a key pair
    AutogenerateNewKeypair(self::autogenerate_new_keypair::GenerateKeypair),
    // #[strum_discriminants(strum(
    //     message = "use-manually-provided-seed-prase  - Use the provided seed phrase manually"
    // ))]
    // ///Use the provided seed phrase manually
    // UseManuallyProvidedSeedPhrase(
    //     self::use_manually_provided_seed_phrase::AddAccessWithSeedPhraseAction,
    // ),
    #[strum_discriminants(strum(
        message = "use-manually-provided-public-key  - Use the provided public key manually"
    ))]
    ///Use the provided public key manually
    UseManuallyProvidedPublicKey(self::use_public_key::AddAccessKeyAction),
    //     #[cfg(feature = "ledger")]
    //     #[strum_discriminants(strum(message = "use-ledger                        - Use a ledger"))]
    //     ///Use a ledger
    //     UseLedger(self::use_ledger::AddAccessWithLedger),
}

impl AccessKeyMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::AccountProperties,
    ) -> crate::CliResult {
        match self {
            AccessKeyMode::UseManuallyProvidedPublicKey(add_access_key_action) => {
                add_access_key_action
                    .process(config, account_properties)
                    .await
            }
            AccessKeyMode::AutogenerateNewKeypair(generate_keypair) => {
                generate_keypair.process(config, account_properties).await
            } // AccessKeyMode::UseManuallyProvidedSeedPhrase(add_access_with_seed_phrase_action) => {
              //     add_access_with_seed_phrase_action
              //         .process(config, account_properties)
              //         .await
              // }
              // #[cfg(feature = "ledger")]
              // AccessKeyMode::UseLedger(add_access_with_ledger) => {
              //     add_access_with_ledger
              //         .process(config, account_properties)
              //         .await
              // }
        }
    }
}
