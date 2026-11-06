mod get_staking_pool_info;
mod get_validators_stake;

pub use get_staking_pool_info::{RewardFeeFraction, StakingPoolInfo};
pub use get_validators_stake::get_validators_stake;

use get_staking_pool_info::get_staking_pool_info;

use futures::{StreamExt, TryStreamExt};

#[tracing::instrument(name = "Getting a list of validators ...", skip_all)]
pub fn get_validator_list(
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<Vec<StakingPoolInfo>> {
    tracing::info!(target: "near_teach_me", "Getting a list of validators ...");

    let json_rpc_client = network_config.json_rpc_client();

    let validators_stake = get_validators_stake(&json_rpc_client)?;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let concurrency = 10;

    let mut validator_list = runtime.block_on(
        futures::stream::iter(validators_stake.iter())
            .map(|(validator_account_id, stake)| async {
                get_staking_pool_info(
                    &json_rpc_client.clone(),
                    validator_account_id.clone(),
                    *stake,
                )
                .await
            })
            .buffer_unordered(concurrency)
            .try_collect::<Vec<_>>(),
    )?;
    validator_list.sort_by_key(|b| std::cmp::Reverse(b.stake));
    Ok(validator_list)
}
