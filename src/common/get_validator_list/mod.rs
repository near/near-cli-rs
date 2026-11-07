mod get_nearblocks_pools_info;
mod get_staking_pool_info;
mod get_validators;
mod shared;

pub use get_validators::get_validators;
pub use shared::staking_pool_info::{RewardFeeFraction, StakingPoolInfo};
pub use shared::validator_status::ValidatorStatus;

use crate::config::NetworkConfig;
use get_nearblocks_pools_info::get_nearblocks_pools_info;
use get_staking_pool_info::get_staking_pool_info;

use color_eyre::eyre;
use futures::{StreamExt, TryStreamExt, stream};
use std::cmp::Reverse;
use std::collections::HashMap;
use tokio::runtime;

#[tracing::instrument(name = "Getting a list of validators ...", skip_all)]
pub fn get_validator_list(network_config: &NetworkConfig) -> eyre::Result<Vec<StakingPoolInfo>> {
    tracing::info!(target: "near_teach_me", "Getting a list of validators ...");

    let json_rpc_client = network_config.json_rpc_client();

    // 1. Fetch a full list of validators + their stakes and statuses from RPC;
    let validators = get_validators(&json_rpc_client)?;

    // 2. Get the staking pools info (smart contract) for all validators;
    // Getting the fee and the number of delegators takes two RPC calls per validator, which
    // exceeds the RPC rate limits on mainnet (400+ validators) so we try to fetch it from
    // NearBlocks, and use RPC only for the validators that NearBlocks has no staking pool info for,
    // or if the network has no NearBlocks or its API failed.
    let mut nearblocks_pools_info = match &network_config.nearblocks_url {
        Some(nearblocks_url) => {
            get_nearblocks_pools_info(nearblocks_url, validators.keys().collect())
        }
        None => HashMap::new(),
    };

    let mut validator_list = Vec::with_capacity(validators.len());
    let mut validators_to_query_from_rpc = Vec::new();

    for (validator_id, (stake, status)) in validators {
        match nearblocks_pools_info.remove(&validator_id) {
            Some((fee, delegators)) => validator_list.push(StakingPoolInfo {
                validator_id,
                status,
                fee: Some(fee),
                delegators: Some(delegators),
                stake,
            }),
            None => validators_to_query_from_rpc.push((validator_id, stake, status)),
        }
    }

    if !validators_to_query_from_rpc.is_empty() {
        let runtime = runtime::Builder::new_multi_thread().enable_all().build()?;
        let concurrency = 10;

        validator_list.extend(
            runtime.block_on(
                stream::iter(validators_to_query_from_rpc)
                    .map(|(validator_account_id, stake, status)| {
                        get_staking_pool_info(&json_rpc_client, validator_account_id, stake, status)
                    })
                    .buffer_unordered(concurrency)
                    .try_collect::<Vec<_>>(),
            )?,
        );
    }

    validator_list.sort_by_key(|b| Reverse(b.stake));

    Ok(validator_list)
}
