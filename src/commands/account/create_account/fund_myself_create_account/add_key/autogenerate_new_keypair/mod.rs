use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::CreateAccountContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

impl GenerateKeypair {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        self.save_mode.process(config, account_properties).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::common::CreateAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Save an access key for this account
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    ))]
    ///Save automatically generated key pair to macOS keychain
    SaveToMacosKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    ///Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    ///Print automatically generated key pair in terminal
    PrintToTerminal(SignAs),
}

impl SaveMode {
    #[cfg(target_os = "macos")]
    pub async fn save_access_key_to_macos_keychain(
        network_config: crate::config::NetworkConfig,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        let key_pair_properties = account_properties
            .key_pair_properties
            .expect("Impossible to get key_pair_properties!");
        crate::common::save_access_key_to_macos_keychain(
            network_config,
            key_pair_properties.clone(),
            account_properties
                .new_account_id
                .clone()
                .expect("Impossible to get account_id!")
                .as_ref(),
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })
    }

    pub async fn save_access_key_to_keychain(
        config: crate::config::Config,
        network_config: crate::config::NetworkConfig,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        let key_pair_properties = account_properties
            .key_pair_properties
            .expect("Impossible to get key_pair_properties!");
        crate::common::save_access_key_to_keychain(
            network_config,
            config.credentials_home_dir.clone(),
            key_pair_properties.clone(),
            account_properties
                .new_account_id
                .clone()
                .expect("Impossible to get account_id!")
                .as_ref(),
        )
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })
    }

    pub fn print_access_key_to_terminal(account_properties: super::super::AccountProperties) {
        let key_pair_properties = account_properties
            .key_pair_properties
            .expect("Impossible to get key_pair_properties!");
        println!(
            "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
            key_pair_properties.master_seed_phrase,
            key_pair_properties.seed_phrase_hd_path,
            key_pair_properties.implicit_account_id,
            key_pair_properties.public_key_str,
            key_pair_properties.secret_keypair_str,
        )
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        match self {
            #[cfg(target_os = "macos")]
            SaveMode::SaveToMacosKeychain(save_keypair_to_macos_keychain) => {
                let account_properties = super::super::AccountProperties {
                    public_key: crate::types::public_key::PublicKey::from_str(
                        &key_pair_properties.public_key_str,
                    )?,
                    key_pair_properties: Some(key_pair_properties),
                    storage: Some(SaveModeDiscriminants::SaveToMacosKeychain),
                    ..account_properties
                };
                save_keypair_to_macos_keychain
                    .process(config, account_properties)
                    .await
            }
            SaveMode::SaveToKeychain(save_keypair_to_keychain) => {
                let account_properties = super::super::AccountProperties {
                    public_key: crate::types::public_key::PublicKey::from_str(
                        &key_pair_properties.public_key_str,
                    )?,
                    key_pair_properties: Some(key_pair_properties),
                    storage: Some(SaveModeDiscriminants::SaveToKeychain),
                    ..account_properties
                };
                save_keypair_to_keychain
                    .process(config, account_properties)
                    .await
            }
            SaveMode::PrintToTerminal(print_keypair_to_terminal) => {
                let account_properties = super::super::AccountProperties {
                    public_key: crate::types::public_key::PublicKey::from_str(
                        &key_pair_properties.public_key_str,
                    )?,
                    key_pair_properties: Some(key_pair_properties),
                    ..account_properties
                };
                print_keypair_to_terminal
                    .process(config, account_properties)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::common::CreateAccountContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::SignerAccountId,
}

impl SignAs {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::AccountProperties,
    ) -> crate::CliResult {
        self.sign_as.process(config, account_properties).await
    }
}
