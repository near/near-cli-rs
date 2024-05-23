#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SubmitContext)]
#[interactive_clap(output_context = DisplayContext)]
pub struct Display;

#[derive(Debug, Clone)]
pub struct DisplayContext;

impl DisplayContext {
    pub fn from_previous_context(
        previous_context: super::SubmitContext,
        _scope: &<Display as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        match previous_context.signed_transaction_or_signed_delegate_action {
            super::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                signed_transaction,
            ) => {
                let storage_message = (previous_context.on_before_sending_transaction_callback)(
                    &signed_transaction,
                    &previous_context.network_config,
                )
                .map_err(color_eyre::Report::msg)?;
                eprintln!(
                    "\nSigned transaction (serialized as base64):\n{}\n",
                    crate::types::signed_transaction::SignedTransactionAsBase64::from(
                        signed_transaction
                    )
                );
                eprintln!(
                    "This base64-encoded signed transaction is ready to be sent to the network. You can call RPC server directly, or use a helper command on near CLI:\n$ {} transaction send-signed-transaction\n",
                    crate::common::get_near_exec_path()
                );
                eprintln!("{storage_message}");
            }
            super::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                signed_delegate_action,
            ) => {
                eprintln!(
                    "\nSigned delegate action (serialized as base64):\n{}\n",
                    crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
                        signed_delegate_action
                    )
                );
                eprintln!(
                    "This base64-encoded signed delegate action is ready to be sent to the meta-transaction relayer. There is a helper command on near CLI that can do that:\n$ {} transaction send-meta-transaction\n",
                    crate::common::get_near_exec_path()
                );
            }
        }
        Ok(Self)
    }
}
