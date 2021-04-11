use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod sign_manually;
mod sign_with_keychain;
mod sign_with_private_key;


#[derive(Debug, clap::Clap)]
pub enum CliSignTransaction {
    /// Provide arguments to sign a private key transaction
    SignPrivateKey(self::sign_with_private_key::CliSignPrivateKey),
    /// Provide arguments to sign a keychain transaction
    SignWithKeychain(self::sign_with_keychain::CliSignKeychain),
    /// Provide arguments to sign a manually transaction
    SignManually(self::sign_manually::CliSignManually),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum SignTransaction {
    #[strum_discriminants(strum(
        message = "Yes, I want to sign the transaction with my private key"
    ))]
    SignPrivateKey(self::sign_with_private_key::SignPrivateKey),
    #[strum_discriminants(strum(message = "Yes, I want to sign the transaction with keychain"))]
    SignWithKeychain(self::sign_with_keychain::SignKeychain),
    #[strum_discriminants(strum(
        message = "No, I want to construct the transaction and sign it somewhere else"
    ))]
    SignManually(self::sign_manually::SignManually),
}

impl From<CliSignTransaction> for SignTransaction {
    fn from(item: CliSignTransaction) -> Self {
        match item {
            CliSignTransaction::SignPrivateKey(cli_private_key) => {
                let privat_key = self::sign_with_private_key::SignPrivateKey::from(cli_private_key);
                SignTransaction::SignPrivateKey(privat_key)
            },
            CliSignTransaction::SignWithKeychain(cli_key_chain) => {
                let key_chain = self::sign_with_keychain::SignKeychain::from(cli_key_chain);
                SignTransaction::SignWithKeychain(key_chain)
            },
            CliSignTransaction::SignManually(cli_manually) => {
                let manually = self::sign_manually::SignManually::from(cli_manually);
                SignTransaction::SignManually(manually)
            },
        }
    }
}

impl SignTransaction {    
    pub fn choose_sign_option() -> Self {
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
        let cli_sign_option = match variants[select_sign_options] {
            SignTransactionDiscriminants::SignPrivateKey => {
                CliSignTransaction::SignPrivateKey(Default::default())
            },
            SignTransactionDiscriminants::SignWithKeychain => {
                CliSignTransaction::SignWithKeychain(Default::default())
            },
            SignTransactionDiscriminants::SignManually => {
                CliSignTransaction::SignManually(Default::default())
            },
        };
        Self::from(cli_sign_option)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            SignTransaction::SignPrivateKey(keys) => {
                keys.process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
            SignTransaction::SignWithKeychain(chain) => {
                chain.process(prepopulated_unsigned_transaction, selected_server_url)
                .await
            }
            SignTransaction::SignManually(args_manually) => {
                args_manually.process(prepopulated_unsigned_transaction)
                .await
            }
        }
    }
}
