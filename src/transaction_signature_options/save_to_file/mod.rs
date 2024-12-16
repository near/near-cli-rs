use std::io::Write;

use color_eyre::eyre::Context;
use inquire::CustomType;

use super::super::commands::transaction::send_signed_transaction::FileSignedTransaction;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SubmitContext)]
#[interactive_clap(output_context = SaveToFileContext)]
pub struct SaveToFile {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the location of the file to save the transaction information (path/to/signed-transaction-info.json)?
    file_path: crate::types::path_buf::PathBuf,
}

#[derive(Debug, Clone)]
pub struct SaveToFileContext;

impl SaveToFileContext {
    pub fn from_previous_context(
        previous_context: super::SubmitContext,
        scope: &<SaveToFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path: std::path::PathBuf = scope.file_path.clone().into();

        let storage_message = (previous_context.on_before_sending_transaction_callback)(
            &previous_context.signed_transaction_or_signed_delegate_action,
            &previous_context.network_config,
        )
        .map_err(color_eyre::Report::msg)?;

        match previous_context.signed_transaction_or_signed_delegate_action {
            super::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                signed_transaction,
            ) => {
                let data_signed_transaction =
                    serde_json::to_value(FileSignedTransaction { signed_transaction })?;

                std::fs::File::create(&file_path)
                    .wrap_err_with(|| format!("Failed to create file: {:?}", &file_path))?
                    .write(&serde_json::to_vec(&data_signed_transaction)?)
                    .wrap_err_with(|| format!("Failed to write to file: {:?}", &file_path))?;
                eprintln!("\nThe file {:?} was created successfully. It has a signed transaction (serialized as base64).", &file_path);

                eprintln!(
                    "This base64-encoded signed transaction is ready to be sent to the network. You can call RPC server directly, or use a helper command on near CLI:\n$ {} transaction send-signed-transaction\n",
                    crate::common::get_near_exec_path()
                );
                eprintln!("{storage_message}");
            }
            super::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                signed_delegate_action,
            ) => {
                let signed_delegate_action_as_base64 =
                    crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
                        signed_delegate_action,
                    )
                    .to_string();

                let data_signed_delegate_action = serde_json::json!(
                    {"signed_delegate_action_as_base64": signed_delegate_action_as_base64});

                std::fs::File::create(&file_path)
                    .wrap_err_with(|| format!("Failed to create file: {:?}", &file_path))?
                    .write(&serde_json::to_vec(&data_signed_delegate_action)?)
                    .wrap_err_with(|| format!("Failed to write to file: {:?}", &file_path))?;
                eprintln!("\nThe file {:?} was created successfully. It has a signed delegate action (serialized as base64).", &file_path);

                eprintln!(
                    "This base64-encoded signed delegate action is ready to be sent to the meta-transaction relayer. There is a helper command on near CLI that can do that:\n$ {} transaction send-meta-transaction\n",
                    crate::common::get_near_exec_path()
                );
                eprintln!("{storage_message}");
            }
        }
        Ok(Self)
    }
}

impl SaveToFile {
    fn input_file_path(
        _context: &super::SubmitContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            CustomType::new(
                "What is the location of the file to save the transaction information?",
            )
            .with_starting_input("signed-transaction-info.json")
            .prompt()?,
        ))
    }
}
