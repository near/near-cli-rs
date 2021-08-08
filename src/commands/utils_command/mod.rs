use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod combine_transaction_subcommand_with_signature;
pub mod generate_keypair_subcommand;
mod ledger_publickey_subcommand;
mod send_signed_transaction;
mod sign_transaction_subcommand_with_secret_key;
mod sign_transaction_with_ledger_subcommand;
mod view_serialized_transaction;

/// набор утилит-помощников
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliUtils {
    #[clap(subcommand)]
    util: Option<CliUtil>,
}

#[derive(Debug, Clone)]
pub struct Utils {
    pub util: Util,
}

impl CliUtils {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.util
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Utils> for CliUtils {
    fn from(utils: Utils) -> Self {
        Self {
            util: Some(utils.util.into()),
        }
    }
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

#[derive(Debug, Clone, clap::Clap)]
enum CliUtil {
    /// It generates a random key pair
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    /// Предоставьте данные для подписания данных с помощью private key
    SignTransactionPrivateKey(
        self::sign_transaction_subcommand_with_secret_key::CliSignTransactionPrivateKey,
    ),
    // Provide an unsigned transaction to be signed with Ledger
    SignTransactionWithLedger(
        self::sign_transaction_with_ledger_subcommand::CliSignTransactionWithLedger,
    ),
    /// Предоставьте данные для соединения подготовленной неподписаной транзакции с сигнатурой
    CombineTransactionSignature(
        self::combine_transaction_subcommand_with_signature::CliCombineTransactionSignature,
    ),
    /// Using this module, you can view the contents of a serialized transaction (whether signed or not).
    ViewSerializedTransaction(self::view_serialized_transaction::CliViewSerializedTransaction),
    /// Get Public Key from Ledger
    LedgerPublicKey(self::ledger_publickey_subcommand::CliLedgerPublicKey),
    /// Send signed transaction
    SendSignedTransaction(self::send_signed_transaction::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Util {
    #[strum_discriminants(strum(message = "Generate a key pair"))]
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    #[strum_discriminants(strum(message = "Sign a transaction with private key"))]
    SignTransactionPrivateKey(
        self::sign_transaction_subcommand_with_secret_key::SignTransactionPrivateKey,
    ),
    #[strum_discriminants(strum(message = "Sign a transaction with Ledger"))]
    SignTransactionWithLedger(
        self::sign_transaction_with_ledger_subcommand::SignTransactionWithLedger,
    ),
    #[strum_discriminants(strum(message = "Combine unsigned transaction with signature"))]
    CombineTransactionSignature(
        self::combine_transaction_subcommand_with_signature::CombineTransactionSignature,
    ),
    #[strum_discriminants(strum(message = "Deserializing the bytes from base64"))]
    ViewSerializedTransaction(self::view_serialized_transaction::ViewSerializedTransaction),
    #[strum_discriminants(strum(message = "Get public key from Ledger device"))]
    LedgerPublicKey(self::ledger_publickey_subcommand::CliLedgerPublicKey),
    #[strum_discriminants(strum(message = "Send signed transaction"))]
    SendSignedTransaction(self::send_signed_transaction::operation_mode::OperationMode),
}

impl CliUtil {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::GenerateKeypair(_) => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("generate-keypair".to_owned());
                args
            }
            Self::SignTransactionPrivateKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("sign-transaction-private-key".to_owned());
                args
            }
            Self::SignTransactionWithLedger(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("sign-transaction-with-ledger".to_owned());
                args
            }
            Self::CombineTransactionSignature(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("combine-transaction-signature".to_owned());
                args
            }
            Self::ViewSerializedTransaction(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("view-serialized-transaction".to_owned());
                args
            }
            _ => todo!(),
        }
    }
}

impl From<Util> for CliUtil {
    fn from(util: Util) -> Self {
        match util {
            Util::GenerateKeypair(generate_keypair) => Self::GenerateKeypair(generate_keypair),
            Util::SignTransactionPrivateKey(sign_transaction_secret_key) => {
                Self::SignTransactionPrivateKey(sign_transaction_secret_key.into())
            }
            Util::SignTransactionWithLedger(sign_transaction_with_ledger) => {
                Self::SignTransactionWithLedger(sign_transaction_with_ledger.into())
            }
            Util::CombineTransactionSignature(combine_transaction_signaturte) => {
                Self::CombineTransactionSignature(combine_transaction_signaturte.into())
            }
            Util::ViewSerializedTransaction(view_serialized_transaction) => {
                Self::ViewSerializedTransaction(view_serialized_transaction.into())
            }
            _ => todo!(),
        }
    }
}

impl From<CliUtil> for Util {
    fn from(item: CliUtil) -> Self {
        match item {
            CliUtil::GenerateKeypair(generate_keypair) => Util::GenerateKeypair(generate_keypair),
            CliUtil::SignTransactionPrivateKey(cli_sign_transaction) => {
                let sign_transaction =
                    self::sign_transaction_subcommand_with_secret_key::SignTransactionPrivateKey::from(cli_sign_transaction);
                Util::SignTransactionPrivateKey(sign_transaction)
            }
            CliUtil::SignTransactionWithLedger(cli_sign_transaction_with_ledger) => {
                let sign_transaction =
                    self::sign_transaction_with_ledger_subcommand::SignTransactionWithLedger::from(
                        cli_sign_transaction_with_ledger,
                    );
                Util::SignTransactionWithLedger(sign_transaction)
            }
            CliUtil::CombineTransactionSignature(cli_combine_transaction) => {
                let combine_transaction =
                    self::combine_transaction_subcommand_with_signature::CombineTransactionSignature::from(cli_combine_transaction);
                Util::CombineTransactionSignature(combine_transaction)
            }
            CliUtil::ViewSerializedTransaction(cli_view_serialized_transaction) => {
                let view_serialized_transaction =
                    self::view_serialized_transaction::ViewSerializedTransaction::from(
                        cli_view_serialized_transaction,
                    );
                Util::ViewSerializedTransaction(view_serialized_transaction)
            }
            CliUtil::LedgerPublicKey(ledger_publickey) => Util::LedgerPublicKey(ledger_publickey),
            CliUtil::SendSignedTransaction(cli_operation_mode) => {
                Util::SendSignedTransaction(cli_operation_mode.into())
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
            UtilDiscriminants::SignTransactionPrivateKey => {
                CliUtil::SignTransactionPrivateKey(Default::default())
            }
            UtilDiscriminants::SignTransactionWithLedger => {
                CliUtil::SignTransactionWithLedger(Default::default())
            }
            UtilDiscriminants::CombineTransactionSignature => {
                CliUtil::CombineTransactionSignature(Default::default())
            }
            UtilDiscriminants::ViewSerializedTransaction => {
                CliUtil::ViewSerializedTransaction(Default::default())
            }
            UtilDiscriminants::LedgerPublicKey => CliUtil::LedgerPublicKey(
                self::ledger_publickey_subcommand::CliLedgerPublicKey::default(),
            ),
            UtilDiscriminants::SendSignedTransaction => {
                CliUtil::SendSignedTransaction(Default::default())
            }
        };
        Self::from(cli_util)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::GenerateKeypair(generate_keypair) => generate_keypair.process().await,
            Self::SignTransactionPrivateKey(sign_transaction) => sign_transaction.process().await,
            Self::SignTransactionWithLedger(sign_transaction) => sign_transaction.process().await,
            Self::CombineTransactionSignature(combine_transaction) => {
                combine_transaction.process().await
            }
            Self::ViewSerializedTransaction(view_serialized_transaction) => {
                view_serialized_transaction.process().await
            }
            Self::LedgerPublicKey(ledger_publickey) => ledger_publickey.process().await,
            Self::SendSignedTransaction(operation_mode) => operation_mode.process().await,
        }
    }
}
