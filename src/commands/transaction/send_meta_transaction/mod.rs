use color_eyre::eyre::WrapErr;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignedMetaTransaction {
    #[interactive_clap(subcommand)]
    /// Select the base64 signed meta-transaction input method
    signed_meta_transaction_type: SignedMetaTransactionType,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the Base64 signed meta-transaction input method:
pub enum SignedMetaTransactionType {
    #[strum_discriminants(strum(
        message = "base64-signed-meta-transaction             - Base64-encoded string (e.g. e30=)"
    ))]
    /// Base64-encoded string (e.g. e30=)
    Base64SignedMetaTransaction(Base64SignedMetaTransaction),
    #[strum_discriminants(strum(
        message = "file-with-base64-signed-meta-transaction   - Read base64-encoded string from file (e.g. reusable JSON or binary data)"
    ))]
    /// Read base64-encoded string from file (e.g. reusable JSON or binary data)
    FileWithBase64SignedMetaTransaction(FileWithBase64SignedMetaTransaction),
}

#[derive(Debug, Clone)]
pub struct SignedMetaTransactionContext {
    global_context: crate::GlobalContext,
    signed_delegate_action: near_primitives::action::delegate::SignedDelegateAction,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = Base64SignedMetaTransactionContext)]
pub struct Base64SignedMetaTransaction {
    /// Enter a signed delegate action as base64-encoded string:
    signed_delegate_action: crate::types::signed_delegate_action::SignedDelegateActionAsBase64,
    #[interactive_clap(named_arg)]
    /// What is the relayer account ID?
    sign_as: self::sign_as::RelayerAccountId,
}

#[derive(Debug, Clone)]
pub struct Base64SignedMetaTransactionContext(SignedMetaTransactionContext);

impl Base64SignedMetaTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Base64SignedMetaTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(SignedMetaTransactionContext {
            global_context: previous_context,
            signed_delegate_action: scope.signed_delegate_action.clone().into(),
        }))
    }
}

impl From<Base64SignedMetaTransactionContext> for SignedMetaTransactionContext {
    fn from(item: Base64SignedMetaTransactionContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = FileWithBase64SignedMetaTransactionContext)]
pub struct FileWithBase64SignedMetaTransaction {
    /// Enter the path to the file with the meta-transaction as a string in base64 encoding:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// What is the relayer account ID?
    sign_as: self::sign_as::RelayerAccountId,
}

#[derive(Debug, Clone)]
pub struct FileWithBase64SignedMetaTransactionContext(SignedMetaTransactionContext);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FileSignedMetaTransaction {
    #[serde(rename = "signed_delegate_action_as_base64")]
    pub signed_delegate_action: crate::types::signed_delegate_action::SignedDelegateActionAsBase64,
}

impl FileWithBase64SignedMetaTransactionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<FileWithBase64SignedMetaTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = std::fs::read_to_string(&scope.file_path)
            .wrap_err_with(|| format!("File {:?} not found!", &scope.file_path))?;

        let signed_delegate_action = serde_json::from_str::<FileSignedMetaTransaction>(&data)
            .wrap_err_with(|| format!("Error reading data from file: {:?}", &scope.file_path))?
            .signed_delegate_action;

        Ok(Self(SignedMetaTransactionContext {
            global_context: previous_context,
            signed_delegate_action: signed_delegate_action.into(),
        }))
    }
}

impl From<FileWithBase64SignedMetaTransactionContext> for SignedMetaTransactionContext {
    fn from(item: FileWithBase64SignedMetaTransactionContext) -> Self {
        item.0
    }
}
