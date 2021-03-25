use dialoguer::{theme::ColorfulTheme, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod generate_keypair_subcommand;
mod sign_transaction_subcommand_with_secret_key;
mod combine_transaction_subcommand_with_signature;

#[derive(Debug)]
pub struct Utils {
    pub util: Util,
}

#[derive(Debug, Default, StructOpt)]
pub struct CliUtils {
    #[structopt(subcommand)]
    util: Option<CliUtil>,
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Util {
    #[strum_discriminants(strum(message = "Generate a key pair"))]
    GenerateKeypair(self::generate_keypair_subcommand::GenerateKeypair),
    #[strum_discriminants(strum(message = "Sign a transaction with secret key"))]
    SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey),
    #[strum_discriminants(strum(message = "Combine unsigned transaction with signature"))]
    CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CombineTransactionSignature),
}

#[derive(Debug, StructOpt)]
enum CliUtil {
    GenerateKeypair(self::generate_keypair_subcommand::GenerateKeypair),
    SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::CliSignTransactionSecretKey),
    CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CliCombineTransactionSignature),
}

impl From<CliUtils> for Utils {
    fn from(item: CliUtils) -> Self {
        let cli_util: CliUtil = match item.util {
            Some(cli_util) => cli_util,
            None => Util::choose_util(),
        };
        Utils { util: Util::from(cli_util) }
    }
}

impl Util {
    pub fn process(self) {
        match self {
            Util::GenerateKeypair(generate_keypair) => generate_keypair.process(),
            Util::SignTransactionSecretKey(sign_transaction) => sign_transaction.process(),
            Util::CombineTransactionSignature(combine_transaction) => combine_transaction.process(),
        }
    }
    fn choose_util() -> CliUtil {
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
            UtilDiscriminants::GenerateKeypair => {
                CliUtil::GenerateKeypair(self::generate_keypair_subcommand::GenerateKeypair::default())
            },
            UtilDiscriminants::SignTransactionSecretKey => {
                CliUtil::SignTransactionSecretKey(Default::default())
            },
            UtilDiscriminants::CombineTransactionSignature => {
                CliUtil::CombineTransactionSignature(Default::default())
            }
        }
    }
}

impl From<CliUtil> for Util {
    fn from(item: CliUtil) -> Self {
        match item {
            CliUtil::GenerateKeypair(generate_keypair) => Util::GenerateKeypair(generate_keypair),
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
