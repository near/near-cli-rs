#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = DisplayContext)]
pub struct Display {
    #[interactive_clap(long)]
    /// Enter sender (signer) public key:
    signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    /// Enter a nonce for the access key:
    nonce: u64,
    #[interactive_clap(long)]
    /// Enter recent block hash:
    block_hash: crate::types::crypto_hash::CryptoHash,
}

#[derive(Debug, Clone)]
pub struct DisplayContext;

impl DisplayContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<Display as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: previous_context.prepopulated_transaction.signer_id,
            public_key: scope.signer_public_key.clone().into(),
            nonce: scope.nonce,
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            block_hash: scope.block_hash.into(),
            actions: previous_context.prepopulated_transaction.actions,
        };

        eprintln!(
            "\nTransaction hash to sign:\n{}",
            hex::encode(unsigned_transaction.get_hash_and_size().0)
        );

        eprintln!(
            "\nUnsigned transaction (serialized as base64):\n{}\n",
            crate::types::transaction::TransactionAsBase64::from(unsigned_transaction)
        );
        eprintln!(
            "This base64-encoded transaction can be signed and sent later. There is a helper command on near CLI that can do that:\n$ {} transaction sign-transaction\n",
            crate::common::get_near_exec_path()
        );
        Ok(Self)
    }
}
