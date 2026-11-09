#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use color_eyre::eyre::{self, Context};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_client::methods::validators::RpcValidatorRequest;
use near_primitives::types::{AccountId, EpochReference};
use near_primitives::views::EpochValidatorInfo;

use super::shared::{ValidatorInfo, ValidatorStatus};
use crate::common::JsonRpcClientExt;

/// Merges the validators of the next and the current epoch with the current proposals, in this
/// order, so each account keeps its latest stake: the one of the next epoch, of the current
/// epoch, or the proposed one. The status follows from the same lists.
fn merge_validators_info(
    epoch_validator_info: EpochValidatorInfo,
) -> HashMap<AccountId, ValidatorInfo> {
    let EpochValidatorInfo {
        current_validators,
        next_validators,
        current_proposals,
        ..
    } = epoch_validator_info;

    let current_validator_ids: HashSet<_> = current_validators
        .iter()
        .map(|validator| validator.account_id.clone())
        .collect();

    let mut validators = HashMap::new();

    for validator in next_validators {
        let status = if current_validator_ids.contains(&validator.account_id) {
            ValidatorStatus::Active
        } else {
            ValidatorStatus::Joining
        };
        validators.insert(
            validator.account_id,
            ValidatorInfo {
                stake: validator.stake,
                status,
            },
        );
    }

    for validator in current_validators {
        validators
            .entry(validator.account_id)
            .or_insert(ValidatorInfo {
                stake: validator.stake,
                status: ValidatorStatus::Leaving,
            });
    }

    for proposal in current_proposals {
        let (account_id, stake) = proposal.into_validator_stake().account_and_stake();
        validators.entry(account_id).or_insert(ValidatorInfo {
            stake,
            status: ValidatorStatus::Proposal,
        });
    }
    validators
}

/// Returns the stake and the status of every validator of the current and the next epoch,
/// and of every account that proposed to validate in the current epoch.
#[tracing::instrument(name = "Getting the stake and status of validators ...", skip_all)]
pub fn get_validators_info(
    json_rpc_client: &JsonRpcClient,
) -> eyre::Result<HashMap<AccountId, ValidatorInfo>> {
    tracing::info!(target: "near_teach_me", "Getting the stake and status of validators ...");

    let epoch_validator_info = json_rpc_client
        .blocking_call(&RpcValidatorRequest {
            epoch_reference: EpochReference::Latest,
        })
        .wrap_err("Failed to get epoch validators information request.")?;

    Ok(merge_validators_info(epoch_validator_info))
}
