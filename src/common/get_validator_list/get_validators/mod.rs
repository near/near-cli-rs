#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

use color_eyre::eyre::{self, Context};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_client::methods::validators::RpcValidatorRequest;
use near_primitives::types::{AccountId, EpochReference};
use near_primitives::views::EpochValidatorInfo;
use near_token::NearToken;

use super::shared::validator_status::ValidatorStatus;
use crate::common::JsonRpcClientExt;

/// Merges the validators of the next and the current epoch with the current proposals, in this
/// order, so each validator keeps the stake of the latest epoch it has a seat in. The status
/// follows from the same lists, and a validator that has no seat in the next epoch takes its
/// kickout reason from the kickouts made at the start of the current epoch.
fn collect_validators(
    epoch_validator_info: EpochValidatorInfo,
) -> HashMap<AccountId, (NearToken, ValidatorStatus)> {
    let EpochValidatorInfo {
        current_validators,
        next_validators,
        current_proposals,
        prev_epoch_kickout,
        ..
    } = epoch_validator_info;

    let current_validator_ids: HashSet<_> = current_validators
        .iter()
        .map(|validator| validator.account_id.clone())
        .collect();

    let mut kickout_reasons: HashMap<_, _> = prev_epoch_kickout
        .into_iter()
        .map(|kickout| (kickout.account_id, kickout.reason))
        .collect();

    let mut validators = HashMap::new();

    for validator in next_validators {
        let status = if current_validator_ids.contains(&validator.account_id) {
            ValidatorStatus::Active
        } else {
            ValidatorStatus::Joining
        };
        validators.insert(validator.account_id, (validator.stake, status));
    }

    for validator in current_validators {
        validators
            .entry(validator.account_id)
            .or_insert_with_key(|account_id| {
                let kickout_reason = kickout_reasons.remove(account_id);
                (validator.stake, ValidatorStatus::Leaving { kickout_reason })
            });
    }

    for proposal in current_proposals {
        let (account_id, stake) = proposal.into_validator_stake().account_and_stake();
        validators
            .entry(account_id)
            .or_insert_with_key(|account_id| {
                let kickout_reason = kickout_reasons.remove(account_id);
                (stake, ValidatorStatus::Proposal { kickout_reason })
            });
    }
    validators
}

/// Returns the stake and the status of every validator of the current and the next epoch,
/// and of every account that proposed to validate in the current epoch.
#[tracing::instrument(name = "Getting the stake and status of validators ...", skip_all)]
pub fn get_validators(
    json_rpc_client: &JsonRpcClient,
) -> eyre::Result<HashMap<AccountId, (NearToken, ValidatorStatus)>> {
    tracing::info!(target: "near_teach_me", "Getting the stake and status of validators ...");

    let epoch_validator_info = json_rpc_client
        .blocking_call(&RpcValidatorRequest {
            epoch_reference: EpochReference::Latest,
        })
        .wrap_err("Failed to get epoch validators information request.")?;

    Ok(collect_validators(epoch_validator_info))
}
