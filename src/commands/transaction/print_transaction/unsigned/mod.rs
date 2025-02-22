use near_primitives::transaction::Transaction;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = PrintContext)]
pub struct PrintTransaction {
    /// Enter the unsigned transaction encoded in base64:
    unsigned_transaction: crate::types::transaction::TransactionAsBase64,
}

#[derive(Debug, Clone)]
pub struct PrintContext;

impl PrintContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<PrintTransaction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let unsigned_transaction: near_primitives::transaction::TransactionV0 =
            scope.unsigned_transaction.clone().into();

        crate::common::print_full_unsigned_transaction(
            Transaction::V0(unsigned_transaction),
            "\nUnsigned transaction (full):\n".to_string(),
        );

        Ok(Self)
    }
}
