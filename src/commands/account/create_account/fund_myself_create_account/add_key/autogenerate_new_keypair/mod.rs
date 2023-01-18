use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

impl GenerateKeypair {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        self.save_mode.process(config, account_properties).await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
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
    pub fn save_access_key_to_macos_keychain(
        network_config: crate::config::NetworkConfig,
        account_properties: super::super::AccountProperties,
        storage_properties: Option<super::super::StorageProperties>,
    ) -> color_eyre::eyre::Result<String> {
        match storage_properties {
            Some(properties) => {
                let key_pair_properties_buf =
                    serde_json::to_string(&properties.key_pair_properties)?;
                crate::common::save_access_key_to_macos_keychain(
                    network_config,
                    &key_pair_properties_buf,
                    &properties.key_pair_properties.public_key_str,
                    &account_properties.new_account_id,
                )
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to save a file with access key: {}",
                        err
                    ))
                })
            }
            None => Ok(String::new()),
        }
    }

    pub fn save_access_key_to_keychain(
        config: crate::config::Config,
        network_config: crate::config::NetworkConfig,
        account_properties: super::super::super::AccountProperties,
        storage_properties: Option<super::super::StorageProperties>,
    ) -> color_eyre::eyre::Result<String> {
        match storage_properties {
            Some(properties) => {
                let key_pair_properties_buf =
                    serde_json::to_string(&properties.key_pair_properties)?;
                crate::common::save_access_key_to_keychain(
                    network_config,
                    config.credentials_home_dir,
                    &key_pair_properties_buf,
                    &properties.key_pair_properties.public_key_str,
                    &account_properties.new_account_id,
                )
                .map_err(|err| {
                    color_eyre::Report::msg(format!(
                        "Failed to save a file with access key: {}",
                        err
                    ))
                })
            }
            None => Ok(String::new()),
        }
    }

    pub fn print_access_key_to_terminal(
        storage_properties: Option<super::super::StorageProperties>,
    ) -> color_eyre::eyre::Result<String> {
        match storage_properties {
            Some(properties) => Ok(format!(
                "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                properties.key_pair_properties.master_seed_phrase,
                properties.key_pair_properties.seed_phrase_hd_path,
                properties.key_pair_properties.implicit_account_id,
                properties.key_pair_properties.public_key_str,
                properties.key_pair_properties.secret_keypair_str,
            )),
            None => Ok(String::new()),
        }
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        let account_properties = super::super::super::AccountProperties {
            public_key,
            ..account_properties
        };
        match self {
            #[cfg(target_os = "macos")]
            SaveMode::SaveToMacosKeychain(save_keypair_to_macos_keychain) => {
                let storage_properties = super::super::StorageProperties {
                    key_pair_properties,
                    storage: SaveModeDiscriminants::SaveToMacosKeychain,
                };
                save_keypair_to_macos_keychain
                    .process(config, account_properties, Some(storage_properties))
                    .await
            }
            SaveMode::SaveToKeychain(save_keypair_to_keychain) => {
                let storage_properties = super::super::StorageProperties {
                    key_pair_properties,
                    storage: SaveModeDiscriminants::SaveToKeychain,
                };
                save_keypair_to_keychain
                    .process(config, account_properties, Some(storage_properties))
                    .await
            }
            SaveMode::PrintToTerminal(print_keypair_to_terminal) => {
                let storage_properties = super::super::StorageProperties {
                    key_pair_properties,
                    storage: SaveModeDiscriminants::PrintToTerminal,
                };
                print_keypair_to_terminal
                    .process(config, account_properties, Some(storage_properties))
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}

impl SignAs {
    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
        storage_properties: Option<super::super::StorageProperties>,
    ) -> crate::CliResult {
        self.sign_as
            .process(config, account_properties, storage_properties)
            .await
    }
}
