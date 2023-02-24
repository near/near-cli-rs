use std::str::FromStr;

use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::super::NewAccountContext)]
#[interactive_clap(output_context = GenerateKeypairContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

#[derive(Debug, Clone)]
pub struct GenerateKeypairContext {
    config: crate::config::Config,
    account_properties: crate::commands::account::create_account::AccountProperties,
    key_pair_properties: crate::common::KeyPairProperties,
}

impl GenerateKeypairContext {
    pub fn from_previous_context(
        previous_context: super::super::NewAccountContext,
        _scope: &<GenerateKeypair as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let key_pair_properties: crate::common::KeyPairProperties = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(crate::common::generate_keypair())?;
        let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        let account_properties = crate::commands::account::create_account::AccountProperties {
            new_account_id: previous_context.new_account_id,
            initial_balance: previous_context.initial_balance,
            public_key,
        };

        Ok(Self {
            config: previous_context.config,
            account_properties,
            key_pair_properties,
        })
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = GenerateKeypairContext)]
#[interactive_clap(output_context = crate::commands::account::create_account::CreateAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Save an access key for this account
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to macOS keychain"
    ))]
    /// Save automatically generated key pair to macOS keychain
    SaveToMacosKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    /// Save automatically generated key pair to the legacy keychain (compatible with JS CLI)
    SaveToKeychain(SignAs),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    /// Print automatically generated key pair in terminal
    PrintToTerminal(SignAs),
}

#[derive(Clone)]
pub struct SaveModeContext(crate::commands::account::create_account::CreateAccountContext);

impl SaveModeContext {
    pub fn from_previous_context(
        previous_context: GenerateKeypairContext,
        scope: &<SaveMode as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let scope = scope.clone();

        println!("\n-------------------- SaveModeContext  ------------------\n");
        println!(
            "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
            previous_context.key_pair_properties.master_seed_phrase,
            previous_context.key_pair_properties.seed_phrase_hd_path,
            previous_context.key_pair_properties.implicit_account_id,
            previous_context.key_pair_properties.public_key_str,
            previous_context.key_pair_properties.secret_keypair_str,
        );
        println!("\n--------------------------------------------------------");

        let on_before_sending_transaction_callback: crate::transaction_signature_options::OnBeforeSendingTransactionCallback =
        std::sync::Arc::new({
            // let new_account_id = item.new_account_id.clone();
            // let signer_account_id = item.signer_account_id.clone();
            let key_pair_properties = previous_context.key_pair_properties.clone();
            let credentials_home_dir = previous_context.config.credentials_home_dir.clone();

            move |_prepopulated_unsigned_transaction, _network_config| {

                match scope {
                    #[cfg(target_os = "macos")]
                    SaveModeDiscriminants::SaveToMacosKeychain => todo!(),
                    SaveModeDiscriminants::SaveToKeychain => todo!(),
                    SaveModeDiscriminants::PrintToTerminal => {
                        println!("\n--------------------  Access key info ------------------\n");
                        println!(
                            "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
                            key_pair_properties.master_seed_phrase,
                            key_pair_properties.seed_phrase_hd_path,
                            key_pair_properties.implicit_account_id,
                            key_pair_properties.public_key_str,
                            key_pair_properties.secret_keypair_str,
                        );
                        println!("\n--------------------------------------------------------");
                    }
                };

                Ok(())
            }
        });

        Ok(Self(
            crate::commands::account::create_account::CreateAccountContext {
                config: previous_context.config,
                account_properties: previous_context.account_properties,
                on_before_sending_transaction_callback,
            },
        ))
    }
}

impl From<SaveModeContext> for crate::commands::account::create_account::CreateAccountContext {
    fn from(item: SaveModeContext) -> Self {
        item.0
    }
}

impl SaveMode {
    // #[cfg(target_os = "macos")]
    // pub fn save_access_key_to_macos_keychain(
    //     network_config: crate::config::NetworkConfig,
    //     account_properties: super::super::super::AccountProperties,
    //     storage_properties: Option<super::super::StorageProperties>,
    // ) -> color_eyre::eyre::Result<String> {
    //     match storage_properties {
    //         Some(properties) => {
    //             let key_pair_properties_buf =
    //                 serde_json::to_string(&properties.key_pair_properties)?;
    //             crate::common::save_access_key_to_macos_keychain(
    //                 network_config,
    //                 &key_pair_properties_buf,
    //                 &properties.key_pair_properties.public_key_str,
    //                 &account_properties.new_account_id,
    //             )
    //             .map_err(|err| {
    //                 color_eyre::Report::msg(format!(
    //                     "Failed to save a file with access key: {}",
    //                     err
    //                 ))
    //             })
    //         }
    //         None => Ok(String::new()),
    //     }
    // }

    // pub fn save_access_key_to_keychain(
    //     config: crate::config::Config,
    //     network_config: crate::config::NetworkConfig,
    //     account_properties: super::super::super::AccountProperties,
    //     storage_properties: Option<super::super::StorageProperties>,
    // ) -> color_eyre::eyre::Result<String> {
    //     match storage_properties {
    //         Some(properties) => {
    //             let key_pair_properties_buf =
    //                 serde_json::to_string(&properties.key_pair_properties)?;
    //             crate::common::save_access_key_to_keychain(
    //                 network_config,
    //                 config.credentials_home_dir,
    //                 &key_pair_properties_buf,
    //                 &properties.key_pair_properties.public_key_str,
    //                 &account_properties.new_account_id,
    //             )
    //             .map_err(|err| {
    //                 color_eyre::Report::msg(format!(
    //                     "Failed to save a file with access key: {}",
    //                     err
    //                 ))
    //             })
    //         }
    //         None => Ok(String::new()),
    //     }
    // }

    // pub fn print_access_key_to_terminal(
    //     storage_properties: Option<super::super::StorageProperties>,
    // ) -> color_eyre::eyre::Result<String> {
    //     match storage_properties {
    //         Some(properties) => Ok(format!(
    //             "Master Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}",
    //             properties.key_pair_properties.master_seed_phrase,
    //             properties.key_pair_properties.seed_phrase_hd_path,
    //             properties.key_pair_properties.implicit_account_id,
    //             properties.key_pair_properties.public_key_str,
    //             properties.key_pair_properties.secret_keypair_str,
    //         )),
    //         None => Ok(String::new()),
    //     }
    // }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: super::super::super::AccountProperties,
    ) -> crate::CliResult {
        // let key_pair_properties: crate::common::KeyPairProperties =
        //     crate::common::generate_keypair().await?;
        // let public_key = near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?;
        // let account_properties = super::super::super::AccountProperties {
        //     public_key,
        //     ..account_properties
        // };
        // match self {
        //     #[cfg(target_os = "macos")]
        //     SaveMode::SaveToMacosKeychain(save_keypair_to_macos_keychain) => {
        //         let storage_properties = super::super::StorageProperties {
        //             key_pair_properties,
        //             storage: SaveModeDiscriminants::SaveToMacosKeychain,
        //         };
        //         save_keypair_to_macos_keychain
        //             .process(config, account_properties, Some(storage_properties))
        //             .await
        //     }
        //     SaveMode::SaveToKeychain(save_keypair_to_keychain) => {
        //         let storage_properties = super::super::StorageProperties {
        //             key_pair_properties,
        //             storage: SaveModeDiscriminants::SaveToKeychain,
        //         };
        //         save_keypair_to_keychain
        //             .process(config, account_properties, Some(storage_properties))
        //             .await
        //     }
        //     SaveMode::PrintToTerminal(print_keypair_to_terminal) => {
        //         let storage_properties = super::super::StorageProperties {
        //             key_pair_properties,
        //             storage: SaveModeDiscriminants::PrintToTerminal,
        //         };
        //         print_keypair_to_terminal
        //             .process(config, account_properties, Some(storage_properties))
        //             .await
        //     }
        // }
        Ok(())
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
pub struct SignAs {
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: super::super::sign_as::SignerAccountId,
}
