use std::str::FromStr;

use color_eyre::eyre::WrapErr;

use crate::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DelegateStakeContext)]
#[interactive_clap(output_context = WithdrawAllContext)]
pub struct WithdrawAll {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct WithdrawAllContext(crate::commands::ActionContext);

impl WithdrawAllContext {
    pub fn from_previous_context(
        previous_context: super::DelegateStakeContext,
        scope: &<WithdrawAll as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let validator_account_id = previous_context.validator_account_id.clone();
        let interacting_with_account_ids = vec![validator_account_id.clone()];

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id: near_primitives::types::AccountId =
                    scope.signer_account_id.clone().into();

                move |network_config| {
                    let is_account_unstaked_balance_available = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            &previous_context.validator_account_id,
                            "is_account_unstaked_balance_available",
                            serde_json::json!({
                                "account_id": signer_id.to_string(),
                            })
                            .to_string()
                            .into_bytes(),
                            near_primitives::types::BlockReference::Finality(near_primitives::types::Finality::Final),
                        )
                        .wrap_err(
                            "Failed to fetch query for view method: 'is_account_unstaked_balance_available'"
                        )?
                        .parse_result_from_json::<bool>()
                        .wrap_err(
                            "Failed to parse return value of view function call for bool value."
                        )?;
                    if !is_account_unstaked_balance_available {
                        return Err(color_eyre::Report::msg(format!(
                            "<{signer_id}> can't withdraw tokens in the current epoch."
                        )));
                    }

                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: previous_context.validator_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::FunctionCall(
                            near_primitives::transaction::FunctionCallAction {
                                method_name: "withdraw_all".to_string(),
                                args: serde_json::json!({}).to_string().into_bytes(),
                                gas: crate::common::NearGas::from_str("300 TeraGas")
                                    .unwrap()
                                    .inner,
                                deposit: crate::common::NearBalance::from_yoctonear(0)
                                    .to_yoctonear(),
                            },
                        )],
                    })
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer = scope.signer_account_id.clone();

            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!("<{signer}> has successfully withdrawn the entire amount from <{validator_account_id}>.")
                }
                Ok(())
            }
        });
        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<WithdrawAllContext> for crate::commands::ActionContext {
    fn from(item: WithdrawAllContext) -> Self {
        item.0
    }
}

impl WithdrawAll {
    pub fn input_signer_account_id(
        context: &super::DelegateStakeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the signer account ID?",
        )
    }
}
