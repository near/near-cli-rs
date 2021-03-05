use structopt::StructOpt;
use dialoguer::{
    Select,
    theme::ColorfulTheme,
    console::Term
};

pub mod sign_private_key;
use sign_private_key::{
    SignPrivateKey,
    CliSignPrivateKey
};
pub mod sign_keychain;
use sign_keychain::{
    SignKeychain,
    CliSignKeychain
};
pub mod sign_manually;
use sign_manually::{
    SignManually,
    CliSignManually
};


#[derive(Debug)]
pub enum SignTransaction {
    SignPrivateKey(SignPrivateKey),
    SignKeychain(SignKeychain),
    SignManually(SignManually)
}

#[derive(Debug, StructOpt)]
pub enum CliSignTransaction {
    SignPrivateKey(CliSignPrivateKey),
    SignKeychain(CliSignKeychain),
    SignManually(CliSignManually)
}

impl SignTransaction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        println!("SignTransaction process: self:       {:?}", &self);
        println!("SignTransaction process: prepopulated_unsigned_transaction:       {:?}", &prepopulated_unsigned_transaction);
        match self {
            SignTransaction::SignPrivateKey(keys) => keys.process(prepopulated_unsigned_transaction, selected_server_url).await,
            SignTransaction::SignKeychain(chain) => chain.process(prepopulated_unsigned_transaction, selected_server_url),
            SignTransaction::SignManually(args_manually) => {}
        }
    }
    pub fn choose_sign_option() -> Self {
        println!();
        let sign_options = vec![
            "Yes, I want to sign the transaction with my private key",
            "No, I want to construct the transaction and sign it somewhere else",
        ];
        let select_sign_options = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Would you like to sign the transaction?")
            .items(&sign_options)
            .default(0)
            .interact()
            .unwrap();
        match sign_options[select_sign_options] {
            "Yes, I want to sign the transaction with my private key" => {
                SignTransaction::SignPrivateKey(SignPrivateKey{
                    signer_public_key: SignPrivateKey::signer_public_key(),
                    signer_secret_key: SignPrivateKey::signer_secret_key()
                })
            },
            "No, I want to construct the transaction and sign it somewhere else" => {
                SignTransaction::SignKeychain(SignKeychain{key_chain: SignKeychain::input_key_chain()})
            },
            _ => SignTransaction::SignPrivateKey(SignPrivateKey{
                signer_public_key: SignPrivateKey::signer_public_key(),
                signer_secret_key: SignPrivateKey::signer_secret_key()
            })
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
            // CliSignTransaction::SignManually(cli_manually) => {
            //     
            // }
            _ => unreachable!("Error")
        }
    }
}
