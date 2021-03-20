use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

// mod generate_keypair_subcommand;
mod sign_transaction_subcommand_with_secret_key;
mod combine_transaction_subcommand_with_signature;

#[derive(Debug)]
pub struct Utils {
    pub util: Util,
}

#[derive(Debug, StructOpt)]
pub struct CliUtils {
    #[structopt(subcommand)]
    util: Option<CliUtil>,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Util {
    #[strum_discriminants(strum(message = "Sign a transaction with secret key"))]
    SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey),
    #[strum_discriminants(strum(message = "Combine unsigned transaction with signature"))]
    CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CombineTransactionSignature),
}

#[derive(Debug, StructOpt)]
enum CliUtil {
    SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::CliSignTransactionSecretKey),
    CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CliCombineTransactionSignature),
}

impl From<CliUtils> for Utils {
    fn from(item: CliUtils) -> Self {
        let util: Util = match item.util {
            Some(cli_util) => Util::from(cli_util),
            None => Util::choose_util(),
        };
        Utils { util }
    }
}

impl Util {
    pub fn process(self) {
        match self {
            Util::SignTransactionSecretKey(sign_transaction) => sign_transaction.process(),
            Util::CombineTransactionSignature(combine_transaction) => combine_transaction.process(),
        }
    }
    pub fn choose_util() -> Self {
        println!();
        let variants = UtilDiscriminants::iter().collect::<Vec<_>>();
        let utils = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&utils)
            .default(0)
            .interact()
            .unwrap();
        match variants[selection] {
            UtilDiscriminants::SignTransactionSecretKey => {
                let signer_secret_key =
                    self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey::input_signer_secret_key();
                let unsigned_transaction =
                    self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey::input_unsigned_transaction();
                Self::SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey {
                    signer_secret_key,
                    unsigned_transaction,
                })
            },
            UtilDiscriminants::CombineTransactionSignature => {
                let signature =
                    self::combine_transaction_subcommand_with_signature::CombineTransactionSignature::input_signature();
                let unsigned_transaction =
                    self::combine_transaction_subcommand_with_signature::CombineTransactionSignature::input_unsigned_transaction();
                Self::CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CombineTransactionSignature {
                    signature,
                    unsigned_transaction
                })
            }
        }
    }
}

impl From<CliUtil> for Util {
    fn from(item: CliUtil) -> Self {
        match item {
            CliUtil::SignTransactionSecretKey(cli_sign_transaction) => {
                let sign_transaction =
                    self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey::from(cli_sign_transaction);
                Util::SignTransactionSecretKey(sign_transaction)
            },
            CliUtil::CombineTransactionSignature(cli_combine_transaction) => {
                let combine_transaction =
                    self::combine_transaction_subcommand_with_signature::CombineTransactionSignature::from(cli_combine_transaction);
                Util::CombineTransactionSignature(combine_transaction)
            }
        }
    }
}

impl Utils {
    pub fn process(self) {
        self.util.process()
    }
}
