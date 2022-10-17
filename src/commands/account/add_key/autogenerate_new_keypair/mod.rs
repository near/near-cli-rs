use std::str::FromStr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod print_keypair_to_terminal;
mod save_keypair_to_keychain;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct GenerateKeypair {
    #[interactive_clap(subcommand)]
    save_mode: SaveMode,
}

impl GenerateKeypair {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        self.save_mode
            .process(config, prepopulated_unsigned_transaction, permission)
            .await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Save an access key for this account
pub enum SaveMode {
    #[cfg(target_os = "macos")]
    #[strum_discriminants(strum(
        message = "save-to-macos-keychain   - Save automatically generated key pair to OS X keychain"
    ))]
    ///Save automatically generated key pair to OS X keychain
    SaveToOsxKeychain(self::save_keypair_to_keychain::SaveKeypairToKeychain),
    #[strum_discriminants(strum(
        message = "save-to-keychain         - Save automatically generated key pair to the legacy keychain (compatible with JS CLI)"
    ))]
    ///Save automatically generated key pair to keychain
    SaveToKeychain(self::save_keypair_to_keychain::SaveKeypairToKeychain),
    #[strum_discriminants(strum(
        message = "print-to-terminal        - Print automatically generated key pair in terminal"
    ))]
    ///Print automatically generated key pair in terminal
    PrintToTerminal(self::print_keypair_to_terminal::PrintKeypairToTerminal),
}

impl SaveMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        permission: near_primitives::account::AccessKeyPermission,
    ) -> crate::CliResult {
        let key_pair_properties: crate::common::KeyPairProperties =
            crate::common::generate_keypair().await?;
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce: 0,
            permission,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: near_crypto::PublicKey::from_str(&key_pair_properties.public_key_str)?,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self {
            SaveMode::SaveToKeychain(save_keypair_to_keychain) => {
                let is_save_to_macos_keychain = false;
                save_keypair_to_keychain
                    .process(
                        config,
                        key_pair_properties,
                        prepopulated_unsigned_transaction,
                        is_save_to_macos_keychain,
                    )
                    .await
            }
            #[cfg(target_os = "macos")]
            SaveMode::SaveToOsxKeychain(save_keypair_to_keychain) => {
                let is_save_to_macos_keychain = true;
                save_keypair_to_keychain
                    .process(
                        config,
                        key_pair_properties,
                        prepopulated_unsigned_transaction,
                        is_save_to_macos_keychain,
                    )
                    .await
            }
            SaveMode::PrintToTerminal(print_keypair_to_terminal) => {
                print_keypair_to_terminal
                    .process(
                        config,
                        key_pair_properties,
                        prepopulated_unsigned_transaction,
                    )
                    .await
            }
        }
    }
}
