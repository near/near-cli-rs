use std::{str::FromStr, vec};

use color_eyre::eyre::Context;
use inquire::{CustomType, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod using_private_key;
mod using_seed_phrase;
mod using_web_wallet;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ImportAccountCommand {
    #[interactive_clap(subcommand)]
    import_account_actions: ImportAccountActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to import the account?
pub enum ImportAccountActions {
    #[strum_discriminants(strum(
        message = "using-web-wallet          - Import existing account using NEAR Wallet (a.k.a. \"sign in\")"
    ))]
    /// Import existing account using NEAR Wallet (a.k.a. "sign in")
    UsingWebWallet(self::using_web_wallet::LoginFromWebWallet),
    #[strum_discriminants(strum(
        message = "using-seed-phrase         - Import existing account using a seed phrase"
    ))]
    /// Import existing account using a seed phrase
    UsingSeedPhrase(self::using_seed_phrase::LoginFromSeedPhrase),
    #[strum_discriminants(strum(
        message = "using-private-key         - Import existing account using a private key"
    ))]
    /// Import existing account using a private key
    UsingPrivateKey(self::using_private_key::LoginFromPrivateKey),
}

pub fn login(
    network_config: crate::config::NetworkConfig,
    credentials_home_dir: std::path::PathBuf,
    key_pair_properties_buf: &str,
    public_key_str: &str,
    error_message: &str,
) -> crate::CliResult {
    let public_key: near_crypto::PublicKey = near_crypto::PublicKey::from_str(public_key_str)?;

    let account_id = loop {
        let account_id_from_cli = input_account_id()?;
        println!();
        if crate::common::verify_account_access_key(
            account_id_from_cli.clone(),
            public_key.clone(),
            network_config.clone(),
        )
        .is_err()
        {
            println!("{}", error_message);

            #[derive(strum_macros::Display)]
            enum ConfirmOptions {
                #[strum(to_string = "Yes, I want to re-enter the account_id.")]
                Yes,
                #[strum(to_string = "No, I want to save the access key information.")]
                No,
            }
            let select_choose_input = Select::new(
                "Would you like to re-enter the account_id?",
                vec![ConfirmOptions::Yes, ConfirmOptions::No],
            )
            .prompt()?;
            if let ConfirmOptions::No = select_choose_input {
                break account_id_from_cli;
            }
        } else {
            break account_id_from_cli;
        }
    };
    save_access_key(
        account_id,
        key_pair_properties_buf,
        public_key_str,
        network_config,
        credentials_home_dir,
    )?;
    Ok(())
}

fn input_account_id() -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
    Ok(CustomType::new("Enter account ID").prompt()?)
}

fn save_access_key(
    account_id: near_primitives::types::AccountId,
    key_pair_properties_buf: &str,
    public_key_str: &str,
    network_config: crate::config::NetworkConfig,
    credentials_home_dir: std::path::PathBuf,
) -> crate::CliResult {
    #[cfg(target_os = "macos")]
    {
        #[derive(strum_macros::Display)]
        enum SelectStorage {
            #[strum(to_string = "Store the access key in my macOS keychain")]
            SaveToMacosKeychain,
            #[strum(
                to_string = "Store the access key in my legacy keychain (compatible with the old near CLI)"
            )]
            SaveToKeychain,
        }
        let selection = Select::new(
            "Select a keychain to save the access key to:",
            vec![
                SelectStorage::SaveToMacosKeychain,
                SelectStorage::SaveToKeychain,
            ],
        )
        .prompt()?;
        if let SelectStorage::SaveToMacosKeychain = selection {
            let storage_message = crate::common::save_access_key_to_macos_keychain(
                network_config,
                key_pair_properties_buf,
                public_key_str,
                &account_id,
            )
            .wrap_err_with(|| {
                format!(
                    "Failed to save the access key <{}> to the keychain",
                    public_key_str
                )
            })?;
            println!("{}", storage_message);
            return Ok(());
        }
    }
    let storage_message = crate::common::save_access_key_to_keychain(
        network_config,
        credentials_home_dir,
        key_pair_properties_buf,
        public_key_str,
        &account_id,
    )
    .wrap_err_with(|| format!("Failed to save a file with access key: {}", public_key_str))?;
    println!("{}", storage_message);
    Ok(())
}
