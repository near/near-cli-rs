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
    // NOTE: We cannot use `crate::common::NearGas`, as sputnikdao uses `U128`
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
/// Directly translates to `{ "TransferKindName": { TransferArguments }}`
pub enum ProposalKind {
    Transfer(TransferArgs),
    FunctionCall(FunctionCallArgs),
}

impl TryFrom<&crate::commands::PrepopulatedTransaction> for ProposalKind {
    type Error = color_eyre::eyre::Error;

    fn try_from(value: &crate::commands::PrepopulatedTransaction) -> Result<Self, Self::Error> {
        match value.actions.first() {
            Some(Action::Transfer(transaction_action)) => {
                Ok(ProposalKind::Transfer(TransferArgs {
                    token_id: String::new(),
                    receiver_id: value.receiver_id.clone(),
                    amount: crate::types::near_token::NearToken::from_yoctonear(
                        transaction_action.deposit,
                    ),
                    msg: None,
                }))
            }
            Some(Action::FunctionCall(_)) => {
                let action_calls = value
                    .actions
                    .iter()
                    .map(|action| match action {
                        Action::FunctionCall(function_call_action) => Ok(ActionCall {
                            method_name: function_call_action.method_name.clone(),
                            args: function_call_action.args.clone(),
                            deposit: crate::types::near_token::NearToken::from_yoctonear(
                                function_call_action.deposit,
                            ),
                            gas: function_call_action.gas,
                        }),
                        _ => Err(color_eyre::eyre::eyre!(
                            "Mixed action types is not supported by DAO proposals"
                        )),
                    })
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(ProposalKind::FunctionCall(FunctionCallArgs {
                    receiver_id: value.receiver_id.clone(),
                    actions: action_calls,
                }))
            }
            Some(other_action) => Err(color_eyre::eyre::eyre!(
                "Action type {:?} is not supported for DAO proposals",
                std::mem::discriminant(other_action)
            )),
            None => Err(color_eyre::eyre::eyre!("No actions found in transaction")),
        }
    }
}
