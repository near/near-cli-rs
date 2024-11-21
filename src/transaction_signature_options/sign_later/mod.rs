use near_primitives::transaction::TransactionV0;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod display;
mod save_to_file;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignLaterContext)]
pub struct SignLater {
    #[interactive_clap(long)]
    /// Enter sender (signer) public key:
    signer_public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(long)]
    /// Enter a nonce for the access key:
    nonce: u64,
    #[interactive_clap(long)]
    /// Enter recent block hash:
    block_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(subcommand)]
    output: Output,
}

#[derive(Debug, Clone)]
pub struct SignLaterContext {
    unsigned_transaction: near_primitives::transaction::Transaction,
}

impl SignLaterContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLater as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let unsigned_transaction = near_primitives::transaction::Transaction::V0(TransactionV0 {
            signer_id: previous_context.prepopulated_transaction.signer_id,
            public_key: scope.signer_public_key.clone().into(),
            nonce: scope.nonce,
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            block_hash: scope.block_hash.into(),
            actions: previous_context.prepopulated_transaction.actions,
        });
        Ok(Self {
            unsigned_transaction,
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = SignLaterContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to proceed?
pub enum Output {
    #[strum_discriminants(strum(
        message = "save-to-file     - Save the unsigned transaction to file"
    ))]
    /// Save the unsigned transaction to file
    SaveToFile(self::save_to_file::SaveToFile),
    #[strum_discriminants(strum(
        message = "display          - Print the unsigned transaction to terminal"
    ))]
    /// Print the unsigned transaction to terminal
    Display(self::display::Display),
}
