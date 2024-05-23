use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod call_function_type;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = super::ContractFileContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select the need for initialization:
pub enum InitializeMode {
    /// Add an initialize
    #[strum_discriminants(strum(message = "with-init-call     - Add an initialize"))]
    WithInitCall(self::call_function_type::CallFunctionAction),
    /// Don't add an initialize
    #[strum_discriminants(strum(message = "without-init-call  - Don't add an initialize"))]
    WithoutInitCall(NoInitialize),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ContractFileContext)]
#[interactive_clap(output_context = NoInitializeContext)]
pub struct NoInitialize {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct NoInitializeContext(super::ContractFileContext);

impl NoInitializeContext {
    pub fn from_previous_context(
        previous_context: super::ContractFileContext,
        _scope: &<NoInitialize as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::ContractFileContext {
            global_context: previous_context.global_context,
            receiver_account_id: previous_context.receiver_account_id,
            signer_account_id: previous_context.signer_account_id,
            code: previous_context.code,
        }))
    }
}

impl From<NoInitializeContext> for crate::commands::ActionContext {
    fn from(item: NoInitializeContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.0.signer_account_id.clone();
                let receiver_account_id = item.0.receiver_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: receiver_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::DeployContract(
                            near_primitives::transaction::DeployContractAction {
                                code: item.0.code.clone(),
                            },
                        )],
                    })
                }
            });

        Self {
            global_context: item.0.global_context,
            interacting_with_account_ids: vec![
                item.0.signer_account_id,
                item.0.receiver_account_id,
            ],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
