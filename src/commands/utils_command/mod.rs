use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod generate_keypair_subcommand;
mod sign_transaction_subcommand_with_secret_key;
mod combine_transaction_subcommand_with_signature;


#[derive(Debug, Default, clap::Clap)]
pub struct CliUtils {
    #[clap(subcommand)]
    util: Option<CliUtil>,
}

#[derive(Debug)]
pub struct Utils {
    pub util: Util,
}

#[derive(Debug, clap::Clap)]
enum CliUtil {
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::CliSignTransactionSecretKey),
    CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CliCombineTransactionSignature),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Util {
    #[strum_discriminants(strum(message = "Generate a key pair"))]
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    #[strum_discriminants(strum(message = "Sign a transaction with secret key"))]
    SignTransactionSecretKey(self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey),
    #[strum_discriminants(strum(message = "Combine unsigned transaction with signature"))]
    CombineTransactionSignature(self::combine_transaction_subcommand_with_signature::CombineTransactionSignature),
}

impl From<CliUtils> for Utils {
    fn from(item: CliUtils) -> Self {
        let util = match item.util {
            Some(cli_util) => Util::from(cli_util),
            None => Util::choose_util(),
        };
        Self { util }
    }
}

impl Utils {
    pub async fn process(self) -> crate::CliResult {
        self.util.process().await
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

impl Util {
    fn choose_util() -> Self {
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
        let cli_util = match variants[selection] {
            UtilDiscriminants::GenerateKeypair => {
                CliUtil::GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair::default())
            },
            UtilDiscriminants::SignTransactionSecretKey => {
                CliUtil::SignTransactionSecretKey(Default::default())
            },
            UtilDiscriminants::CombineTransactionSignature => {
                CliUtil::CombineTransactionSignature(Default::default())
            }
        };
        Self::from(cli_util)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::GenerateKeypair(generate_keypair) => generate_keypair.process().await,
            Self::SignTransactionSecretKey(sign_transaction) => sign_transaction.process().await,
            Self::CombineTransactionSignature(combine_transaction) => combine_transaction.process().await,
        }
    }
}
