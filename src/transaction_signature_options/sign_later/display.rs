use base64::Engine as _;
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
        let (hash, _size) = previous_context.unsigned_transaction.get_hash_and_size();
        let tx_bytes = borsh::to_vec(&previous_context.unsigned_transaction)
            .expect("Transaction serialization should not fail");
        let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);
        println!(
            "\nTransaction hash to sign:\n{}.\n\nUnsigned transaction (serialized as base64):\n{}\n\nThis base64-encoded transaction can be signed and sent later. There is a helper command on near CLI that can do that:\n$ {} transaction sign-transaction",
            hex::encode(hash.as_bytes()),
            tx_base64,
            crate::common::get_near_exec_path()
        );
        Ok(Self)
    }
}
