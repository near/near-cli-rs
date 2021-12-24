use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod combine_transaction_subcommand_with_signature;
pub mod generate_keypair_subcommand;
#[cfg(feature = "ledger")]
mod ledger_publickey_subcommand;
mod send_signed_transaction;
mod sign_transaction_subcommand_with_secret_key;
#[cfg(feature = "ledger")]
mod sign_transaction_with_ledger_subcommand;
mod view_serialized_transaction;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct Utils {
    #[interactive_clap(subcommand)]
    pub util: Util,
}

impl Utils {
    pub async fn process(self) -> crate::CliResult {
        self.util.process().await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliUtil {
    /// It generates a random key pair
    GenerateKeypair(self::generate_keypair_subcommand::CliGenerateKeypair),
    /// Предоставьте данные для подписания данных с помощью private key
    SignTransactionPrivateKey(
        self::sign_transaction_subcommand_with_secret_key::CliSignTransactionPrivateKey,
    ),
    #[cfg(feature = "ledger")]
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
    #[cfg(feature = "ledger")]
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
    #[cfg(feature = "ledger")]
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
    #[cfg(feature = "ledger")]
    #[strum_discriminants(strum(message = "Get public key from Ledger device"))]
    LedgerPublicKey(self::ledger_publickey_subcommand::CliLedgerPublicKey),
    #[strum_discriminants(strum(message = "Send signed transaction"))]
    SendSignedTransaction(self::send_signed_transaction::operation_mode::OperationMode),
}

impl interactive_clap::ToCli for Util {
    type CliVariant = CliUtil;
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
            #[cfg(feature = "ledger")]
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
            #[cfg(feature = "ledger")]
            Self::LedgerPublicKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("ledger-public-key".to_owned());
                args
            }
            Self::SendSignedTransaction(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("send-signed-transaction".to_owned());
                args
            }
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
            #[cfg(feature = "ledger")]
            Util::SignTransactionWithLedger(sign_transaction_with_ledger) => {
                Self::SignTransactionWithLedger(sign_transaction_with_ledger.into())
            }
            Util::CombineTransactionSignature(combine_transaction_signaturte) => {
                Self::CombineTransactionSignature(combine_transaction_signaturte.into())
            }
            Util::ViewSerializedTransaction(view_serialized_transaction) => {
                Self::ViewSerializedTransaction(view_serialized_transaction.into())
            }
            #[cfg(feature = "ledger")]
            Util::LedgerPublicKey(ledger_publickey) => Self::LedgerPublicKey(ledger_publickey),
            Util::SendSignedTransaction(operation_mode) => {
                Self::SendSignedTransaction(operation_mode.into())
            }
        }
    }
}

impl Util {
    pub fn from_cli(
        optional_clap_variant: Option<<Util as interactive_clap::ToCli>::CliVariant>,
        context: (),
    ) -> color_eyre::eyre::Result<Self> {
        match optional_clap_variant {
            Some(CliUtil::GenerateKeypair(generate_keypair)) => {
                Ok(Util::GenerateKeypair(generate_keypair))
            }
            Some(CliUtil::SignTransactionPrivateKey(cli_sign_transaction)) => {
                let sign_transaction =
                    self::sign_transaction_subcommand_with_secret_key::SignTransactionPrivateKey::from(cli_sign_transaction);
                Ok(Util::SignTransactionPrivateKey(sign_transaction))
            }
            #[cfg(feature = "ledger")]
            Some(CliUtil::SignTransactionWithLedger(cli_sign_transaction_with_ledger)) => {
                let sign_transaction =
                    self::sign_transaction_with_ledger_subcommand::SignTransactionWithLedger::from(
                        cli_sign_transaction_with_ledger,
                    );
                Ok(Util::SignTransactionWithLedger(sign_transaction))
            }
            Some(CliUtil::CombineTransactionSignature(cli_combine_transaction)) => {
                let combine_transaction =
                    self::combine_transaction_subcommand_with_signature::CombineTransactionSignature::from(cli_combine_transaction);
                Ok(Util::CombineTransactionSignature(combine_transaction))
            }
            Some(CliUtil::ViewSerializedTransaction(cli_view_serialized_transaction)) => {
                let view_serialized_transaction =
                    self::view_serialized_transaction::ViewSerializedTransaction::from(
                        cli_view_serialized_transaction,
                    );
                Ok(Util::ViewSerializedTransaction(view_serialized_transaction))
            }
            #[cfg(feature = "ledger")]
            Some(CliUtil::LedgerPublicKey(ledger_publickey)) => {
                Ok(Util::LedgerPublicKey(ledger_publickey))
            }
            Some(CliUtil::SendSignedTransaction(cli_operation_mode)) => {
                Ok(Util::SendSignedTransaction(cli_operation_mode.into()))
            }
            None => Self::choose_variant(context),
        }
    }
}

impl Util {
    fn choose_variant(context: ()) -> color_eyre::eyre::Result<Self> {
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
            #[cfg(feature = "ledger")]
            UtilDiscriminants::SignTransactionWithLedger => {
                CliUtil::SignTransactionWithLedger(Default::default())
            }
            UtilDiscriminants::CombineTransactionSignature => {
                CliUtil::CombineTransactionSignature(Default::default())
            }
            UtilDiscriminants::ViewSerializedTransaction => {
                CliUtil::ViewSerializedTransaction(Default::default())
            }
            #[cfg(feature = "ledger")]
            UtilDiscriminants::LedgerPublicKey => CliUtil::LedgerPublicKey(
                self::ledger_publickey_subcommand::CliLedgerPublicKey::default(),
            ),
            UtilDiscriminants::SendSignedTransaction => {
                CliUtil::SendSignedTransaction(Default::default())
            }
        };
        Ok(Self::from_cli(Some(cli_util), context)?)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::GenerateKeypair(generate_keypair) => generate_keypair.process().await,
            Self::SignTransactionPrivateKey(sign_transaction) => sign_transaction.process().await,
            #[cfg(feature = "ledger")]
            Self::SignTransactionWithLedger(sign_transaction) => sign_transaction.process().await,
            Self::CombineTransactionSignature(combine_transaction) => {
                combine_transaction.process().await
            }
            Self::ViewSerializedTransaction(view_serialized_transaction) => {
                view_serialized_transaction.process().await
            }
            #[cfg(feature = "ledger")]
            Self::LedgerPublicKey(ledger_publickey) => ledger_publickey.process().await,
            Self::SendSignedTransaction(operation_mode) => operation_mode.process().await,
        }
    }
}
