use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod sign_with_private_key;
use sign_with_private_key::{CliSignPrivateKey, SignPrivateKey};
pub mod sign_keychain;
use sign_keychain::{CliSignKeychain, SignKeychain};
pub mod sign_manually;
use sign_manually::{CliSignManually, SignManually};

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SignTransaction {
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with my private key"
    ))]
    SignPrivateKey(SignPrivateKey),
    #[strum_discriminants(strum(message = "Yes, I want to sign the transaction with keychain"))]
    SignKeychain(SignKeychain),
    #[strum_discriminants(strum(
        message = "No, I want to construct the transaction and sign it somewhere else"
    ))]
    SignManually(SignManually),
}

#[derive(Debug, StructOpt)]
pub enum CliSignTransaction {
    SignPrivateKey(CliSignPrivateKey),
    SignKeychain(CliSignKeychain),
    SignManually(CliSignManually),
}

impl SignTransaction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            SignTransaction::SignPrivateKey(keys) => {
                keys.process(prepopulated_unsigned_transaction, selected_server_url)
                    .await?
            }
            SignTransaction::SignKeychain(_chain) => {
                println!("Сейчас ведется доработка данного модуля")
                // chain.process(prepopulated_unsigned_transaction, selected_server_url)
            }
            SignTransaction::SignManually(args_manually) => {
                args_manually.process(prepopulated_unsigned_transaction, selected_server_url)?
            }
        }
        Ok(())
    }
    pub fn choose_sign_option() -> CliSignTransaction {
        println!();
        let variants = SignTransactionDiscriminants::iter().collect::<Vec<_>>();
        let sign_options = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_sign_options = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to sign the transaction?")
            .items(&sign_options)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_sign_options] {
            SignTransactionDiscriminants::SignPrivateKey => {
                CliSignTransaction::SignPrivateKey(Default::default())
            },
            SignTransactionDiscriminants::SignKeychain => {
                // SignTransaction::SignKeychain(SignKeychain{key_chain: SignKeychain::input_key_chain()})
                panic!("This module is under development")
            },
            SignTransactionDiscriminants::SignManually => {
                CliSignTransaction::SignManually(Default::default())
            },
        }
    }
}

impl From<CliSignTransaction> for SignTransaction {
    fn from(item: CliSignTransaction) -> Self {
        match item {
            CliSignTransaction::SignPrivateKey(cli_private_key) => {
                let privat_key = SignPrivateKey::from(cli_private_key);
                SignTransaction::SignPrivateKey(privat_key)
            },
            CliSignTransaction::SignKeychain(cli_key_chain) => {
                let key_chain = SignKeychain::from(cli_key_chain);
                SignTransaction::SignKeychain(key_chain)
            },
            CliSignTransaction::SignManually(cli_manually) => {
                let manually = SignManually::from(cli_manually);
                SignTransaction::SignManually(manually)
            },
        }
    }
}
