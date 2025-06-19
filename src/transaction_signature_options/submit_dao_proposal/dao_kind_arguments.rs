use color_eyre::eyre::eyre;
use near_primitives::action::Action;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionCall {
    method_name: String,
    #[serde_as(as = "Base64")]
    args: Vec<u8>,
    deposit: near_token::NearToken,
    gas: near_gas::NearGas,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TransferArgs {
    #[serde(default)]
    token_id: String,
    receiver_id: near_primitives::types::AccountId,
    amount: crate::types::near_token::NearToken,
    msg: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionCallArgs {
    receiver_id: near_primitives::types::AccountId,
    actions: Vec<ActionCall>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Enum for DAO proposal arguments
///
/// Directly translates to `{ "TransferKindName": TransferArguments }`
pub enum ProposalKind {
    Transfer(TransferArgs),
    FunctionCall(FunctionCallArgs),
}

impl TryFrom<&crate::commands::PrepopulatedTransaction> for ProposalKind {
    type Error = color_eyre::eyre::Error;

    fn try_from(
        transaction: &crate::commands::PrepopulatedTransaction,
    ) -> Result<Self, Self::Error> {
        let Some(first_action) = transaction.actions.first() else {
            return Err(eyre!("No actions were found in transaction!"));
        };

        match first_action {
            Action::Transfer(transfer_action) => {
                if transaction.actions.len() > 1 {
                    Err(eyre!("Batch transfers are not supported for DAO proposals"))
                } else {
                    Ok(ProposalKind::Transfer(TransferArgs {
                        token_id: String::new(),
                        receiver_id: transaction.receiver_id.clone(),
                        amount: crate::types::near_token::NearToken::from_yoctonear(
                            transfer_action.deposit,
                        ),
                        msg: None,
                    }))
                }
            }
            Action::FunctionCall(_) => {
                let action_calls: Vec<_> = transaction
                    .actions
                    .iter()
                    .filter_map(|action| {
                        if let Action::FunctionCall(function_call_action) = action {
                            Some(ActionCall {
                                method_name: function_call_action.method_name.clone(),
                                args: function_call_action.args.clone(),
                                deposit: near_token::NearToken::from_yoctonear(
                                    function_call_action.deposit,
                                ),
                                gas: near_gas::NearGas::from_gas(function_call_action.gas),
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                if action_calls.len() != transaction.actions.len() {
                    Err(eyre!(
                        "Mixed action types are not supported for DAO proposals"
                    ))
                } else {
                    Ok(ProposalKind::FunctionCall(FunctionCallArgs {
                        receiver_id: transaction.receiver_id.clone(),
                        actions: action_calls,
                    }))
                }
            }
            action => Err(eyre!(
                "Passed {action:?} type is not supported for DAO proposal"
            )),
        }
    }
}
