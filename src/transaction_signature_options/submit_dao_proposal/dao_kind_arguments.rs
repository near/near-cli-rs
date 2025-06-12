use color_eyre::eyre::eyre;
use near_primitives::action::Action;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as, DisplayFromStr};

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActionCall {
    method_name: String,
    #[serde_as(as = "Base64")]
    args: Vec<u8>,
    deposit: crate::types::near_token::NearToken,
    // NOTE: We cannot use `crate::common::NearGas`, as sputnikdao uses `U64`
    // https://github.com/near-daos/sputnik-dao-contract/blob/278adc713e795f95a6da4d3007c7e03e8120f153/sputnikdao2/src/proposals.rs#L42
    #[serde_as(as = "DisplayFromStr")]
    gas: u64,
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
        if transaction.actions.is_empty() {
            return Err(eyre!("No actions were found in transaction!"));
        }

        let mut parsed_actions = Vec::new();

        for action in &transaction.actions {
            match action {
                Action::Transfer(_) => {
                    if parsed_actions.is_empty() {
                        parsed_actions.push(action.clone());
                    } else if let Action::Transfer(_) = parsed_actions.last().unwrap() {
                        return Err(eyre!("Batch transfers are not supported for DAO proposals"));
                    } else {
                        return Err(eyre!(
                            "Mixed action types are not supported for DAO proposals"
                        ));
                    }
                }
                Action::FunctionCall(_) => {
                    if parsed_actions.is_empty() {
                        parsed_actions.push(action.clone());
                    } else if let Action::FunctionCall(_) = parsed_actions.last().unwrap() {
                        parsed_actions.push(action.clone());
                    } else {
                        return Err(eyre!(
                            "Mixed action types are not supported for DAO proposals"
                        ));
                    }
                }
                _ => {
                    return Err(eyre!(
                        "Passed `Action` type is not supported for DAO proposal"
                    ));
                }
            }
        }

        match parsed_actions.first() {
            Some(Action::Transfer(transfer_action)) => Ok(ProposalKind::Transfer(TransferArgs {
                token_id: String::new(),
                receiver_id: transaction.receiver_id.clone(),
                amount: crate::types::near_token::NearToken::from_yoctonear(
                    transfer_action.deposit,
                ),
                msg: None,
            })),
            Some(Action::FunctionCall(_)) => {
                let action_calls = parsed_actions
                    .iter()
                    .filter_map(|action| {
                        if let Action::FunctionCall(function_call_action) = action {
                            Some(ActionCall {
                                method_name: function_call_action.method_name.clone(),
                                args: function_call_action.args.clone(),
                                deposit: crate::types::near_token::NearToken::from_yoctonear(
                                    function_call_action.deposit,
                                ),
                                gas: function_call_action.gas,
                            })
                        } else {
                            None
                        }
                    })
                    .collect();

                Ok(ProposalKind::FunctionCall(FunctionCallArgs {
                    receiver_id: transaction.receiver_id.clone(),
                    actions: action_calls,
                }))
            }
            Some(_) => unreachable!(
                "only `Action::FunctionCall` and `Action::Transfer` were pushed to vector"
            ),
            None => unreachable!(
                "only `Action::FunctionCall` and `Action::Transfer` were pushed to vector"
            ),
        }
    }
}
