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
        eprintln!(
            "\nTransaction hash to sign:\n{}",
            hex::encode(previous_context.unsigned_transaction.get_hash_and_size().0)
        );

        eprintln!(
            "\nUnsigned transaction (serialized as base64):\n{}\n",
            crate::types::transaction::TransactionAsBase64::from(
                previous_context.unsigned_transaction
            )
        );
        eprintln!(
            "This base64-encoded transaction can be signed and sent later. There is a helper command on near CLI that can do that:\n$ {} transaction sign-transaction\n",
            crate::common::get_near_exec_path()
        );
        Ok(Self)
    }
}
