use color_eyre::eyre::{self, Context};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_client::methods::validators::RpcValidatorRequest;
use near_primitives::types::{AccountId, EpochReference};
use near_token::NearToken;
use std::collections::HashMap;

use crate::common::JsonRpcClientExt;

#[tracing::instrument(name = "Getting a stake of validators ...", skip_all)]
pub fn get_validators_stake(
    json_rpc_client: &JsonRpcClient,
) -> eyre::Result<HashMap<AccountId, NearToken>> {
    tracing::info!(target: "near_teach_me", "Getting a stake of validators ...");

    let epoch_validator_info = json_rpc_client
        .blocking_call(&RpcValidatorRequest {
            epoch_reference: EpochReference::Latest,
        })
        .wrap_err("Failed to get epoch validators information request.")?;

    Ok(epoch_validator_info
        .current_proposals
        .into_iter()
        .map(|validator_stake_view| {
            let validator_stake = validator_stake_view.into_validator_stake();
            validator_stake.account_and_stake()
        })
        .chain(epoch_validator_info.current_validators.into_iter().map(
            |current_epoch_validator_info| {
                (
                    current_epoch_validator_info.account_id,
                    current_epoch_validator_info.stake,
                )
            },
        ))
        .chain(
            epoch_validator_info
                .next_validators
                .into_iter()
                .map(|next_epoch_validator_info| {
                    (
                        next_epoch_validator_info.account_id,
                        next_epoch_validator_info.stake,
                    )
                }),
        )
        .collect())
}
