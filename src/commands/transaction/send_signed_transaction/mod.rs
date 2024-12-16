use color_eyre::eyre::WrapErr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod network;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignedTransaction {
    #[interactive_clap(subcommand)]
    /// Select the base64 signed transaction input method
    signed_transaction_type: SignedTransactionType,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the Base64 signed transaction input method:
pub enum SignedTransactionType {
    #[strum_discriminants(strum(
        message = "base64-signed-transaction             - Base64-encoded string (e.g. e30=)"
    ))]
    /// Base64-encoded string (e.g. e30=)
    Base64SignedTransaction(Base64SignedTransaction),
    #[strum_discriminants(strum(
        message = "file-with-base64-signed-transaction   - Read base64-encoded string from file (e.g. reusable JSON or binary data)"
    ))]
    /// Read base64-encoded string from file (e.g. reusable JSON or binary data)
    FileWithBase64SignedTransaction(FileWithBase64SignedTransaction),
}

#[derive(Debug, Clone)]
pub struct SignedTransactionContext {
    config: crate::config::Config,
    signed_transaction: near_primitives::transaction::SignedTransaction,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = Base64SignedTransactionContext)]
pub struct Base64SignedTransaction {
    /// Enter a signed transaction as base64-encoded string:
    signed_action: crate::types::signed_transaction::SignedTransactionAsBase64,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: self::network::Network,
}

#[derive(Debug, Clone)]
pub struct Base64SignedTransactionContext(SignedTransactionContext);

impl Base64SignedTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Base64SignedTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(SignedTransactionContext {
            config: previous_context.config,
            signed_transaction: scope.signed_action.inner.clone(),
        }))
    }
}

impl From<Base64SignedTransactionContext> for SignedTransactionContext {
    fn from(item: Base64SignedTransactionContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = FileWithBase64SignedTransactionContext)]
pub struct FileWithBase64SignedTransaction {
    /// Enter the path to the file with the transaction as a string in base64 encoding:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: self::network::Network,
}

#[derive(Debug, Clone)]
pub struct FileWithBase64SignedTransactionContext(SignedTransactionContext);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileSignedTransaction {
    #[serde(rename = "signed_transaction_as_base64")]
    pub signed_transaction: near_primitives::transaction::SignedTransaction,
}

impl FileWithBase64SignedTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<FileWithBase64SignedTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = std::fs::read_to_string(&scope.file_path)
            .wrap_err_with(|| format!("File {:?} not found!", &scope.file_path))?;

        let signed_transaction = serde_json::from_str::<FileSignedTransaction>(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &scope.file_path))?
            .signed_transaction;

        Ok(Self(SignedTransactionContext {
            config: previous_context.config,
            signed_transaction,
        }))
    }
}

impl From<FileWithBase64SignedTransactionContext> for SignedTransactionContext {
    fn from(item: FileWithBase64SignedTransactionContext) -> Self {
        item.0
    }
}
