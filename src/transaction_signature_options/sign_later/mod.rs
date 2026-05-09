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
    pub global_context: crate::GlobalContext,
    pub unsigned_transaction: near_kit::Transaction,
}

impl SignLaterContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignLater as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let nk_public_key: near_kit::PublicKey = {
            let nc_pk: near_kit::PublicKey = scope.signer_public_key.clone().into();
            nc_pk
        };
        let unsigned_transaction = near_kit::Transaction {
            signer_id: previous_context.prepopulated_transaction.signer_id,
            public_key: nk_public_key,
            nonce: scope.nonce,
            receiver_id: previous_context.prepopulated_transaction.receiver_id,
            block_hash: scope.block_hash.0,
            actions: previous_context.prepopulated_transaction.actions,
        };
        Ok(Self {
            global_context: previous_context.global_context,
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
