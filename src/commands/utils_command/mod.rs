use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod combine_transaction_subcommand_with_signature;
pub mod generate_keypair_subcommand;
mod sign_transaction_subcommand_with_secret_key;
mod view_serialized_transaction;

/// набор утилит-помощников
#[derive(Debug, Default, clap::Clap)]
pub struct CliUtils {
    #[clap(subcommand)]
    util: Option<CliUtil>,
}

#[derive(Debug)]
pub struct Utils {
    pub util: Util,
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

#[derive(Debug, clap::Clap)]
enum CliUtil {
    /// It generates a random key pair
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    /// Предоставьте данные для подписания данных с помощью privte key
    SignTransactionSecretKey(
        self::sign_transaction_subcommand_with_secret_key::CliSignTransactionSecretKey,
    ),
    /// Предоставьте данные для соединения подготовленной неподписаной транзакции с сигнатурой
    CombineTransactionSignature(
        self::combine_transaction_subcommand_with_signature::CliCombineTransactionSignature,
    ),
    /// Using this module, you can view the contents of a serialized transaction (whether signed or not).
    ViewSerializedTransaction(self::view_serialized_transaction::CliViewSerializedTransaction)
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Util {
    #[strum_discriminants(strum(message = "Generate a key pair"))]
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    #[strum_discriminants(strum(message = "Sign a transaction with secret key"))]
    SignTransactionSecretKey(
        self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey,
    ),
    #[strum_discriminants(strum(message = "Combine unsigned transaction with signature"))]
    CombineTransactionSignature(
        self::combine_transaction_subcommand_with_signature::CombineTransactionSignature,
    ),
    #[strum_discriminants(strum(message = "Deserializing the bytes from base64"))]
    ViewSerializedTransaction(self::view_serialized_transaction::ViewSerializedTransaction)
}

impl From<CliUtil> for Util {
    fn from(item: CliUtil) -> Self {
        match item {
            CliUtil::GenerateKeypair(generate_keypair) => Util::GenerateKeypair(generate_keypair),
            CliUtil::SignTransactionSecretKey(cli_sign_transaction) => {
                let sign_transaction =
                    self::sign_transaction_subcommand_with_secret_key::SignTransactionSecretKey::from(cli_sign_transaction);
                Util::SignTransactionSecretKey(sign_transaction)
            }
            CliUtil::CombineTransactionSignature(cli_combine_transaction) => {
                let combine_transaction =
                    self::combine_transaction_subcommand_with_signature::CombineTransactionSignature::from(cli_combine_transaction);
                Util::CombineTransactionSignature(combine_transaction)
            }
            CliUtil::ViewSerializedTransaction(cli_view_serialized_transaction) => {
                let view_serialized_transaction =
                    self::view_serialized_transaction::ViewSerializedTransaction::from(cli_view_serialized_transaction);
                Util::ViewSerializedTransaction(view_serialized_transaction)
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
            UtilDiscriminants::GenerateKeypair => CliUtil::GenerateKeypair(
                self::generate_keypair_subcommand::CliGenerateKeypair::default(),
            ),
            UtilDiscriminants::SignTransactionSecretKey => {
                CliUtil::SignTransactionSecretKey(Default::default())
            }
            UtilDiscriminants::CombineTransactionSignature => {
                CliUtil::CombineTransactionSignature(Default::default())
            }
            UtilDiscriminants::ViewSerializedTransaction => {
                CliUtil::ViewSerializedTransaction(Default::default())
            }
        };
        Self::from(cli_util)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::GenerateKeypair(generate_keypair) => generate_keypair.process().await,
            Self::SignTransactionSecretKey(sign_transaction) => sign_transaction.process().await,
            Self::CombineTransactionSignature(combine_transaction) => {
                combine_transaction.process().await
            }
            Self::ViewSerializedTransaction(view_serialized_transaction) => {view_serialized_transaction.process().await},
        }
    }
}
