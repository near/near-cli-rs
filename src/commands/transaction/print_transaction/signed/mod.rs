#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PrintContext)]
pub struct PrintTransaction {
    /// Enter the signed transaction encoded in base64:
    signed_transaction: crate::types::signed_transaction::SignedTransactionAsBase64,
}

#[derive(Debug, Clone)]
pub struct PrintContext;

impl PrintContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<PrintTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let signed_transaction: near_primitives::transaction::SignedTransaction =
            scope.signed_transaction.clone().into();
        let info_str = crate::common::print_full_signed_transaction(signed_transaction);
        if let crate::Verbosity::Quiet = previous_context.verbosity {
            println!("Signed transaction (full):{info_str}")
        };
        tracing::info!(
            parent: &tracing::Span::none(),
            "Signed transaction (full):{}",
            crate::common::indent_payload(&info_str)
        );
        Ok(Self)
    }
}
