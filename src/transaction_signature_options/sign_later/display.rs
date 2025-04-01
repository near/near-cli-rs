use std::io::Write;

use color_eyre::eyre::WrapErr;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SignLaterContext)]
#[interactive_clap(output_context = DisplayContext)]
pub struct Display;

#[derive(Debug, Clone)]
pub struct DisplayContext;

impl DisplayContext {
    pub fn from_previous_context(
        previous_context: super::SignLaterContext,
        _scope: &<Display as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let info_str = format!(
            "\nTransaction hash to sign:\n{}.\n\nUnsigned transaction (serialized as base64):\n{}\n\nThis base64-encoded transaction can be signed and sent later. There is a helper command on near CLI that can do that:\n$ {} transaction sign-transaction\n",
            hex::encode(previous_context.unsigned_transaction.get_hash_and_size().0),
            crate::types::transaction::TransactionAsBase64::from(previous_context.unsigned_transaction),
            crate::common::get_near_exec_path()
        );
        if let crate::Verbosity::Quiet = previous_context.global_context.verbosity {
            std::io::stdout()
                .write_all(info_str.as_bytes())
                .wrap_err(sysexits::ExitCode::DataErr)?;
            return Ok(Self);
        }
        tracing::info!(
            parent: &tracing::Span::none(),
            "{}",
            crate::common::indent_payload(&info_str)
        );
        Ok(Self)
    }
}
