use color_eyre::eyre::{self, Context};
use near_jsonrpc_client::JsonRpcClient;
use near_jsonrpc_client::errors::{JsonRpcError, JsonRpcServerError};
use near_jsonrpc_client::methods::query::{RpcQueryError, RpcQueryRequest};
use near_primitives::types::{AccountId, Finality, FunctionArgs};
use near_primitives::views::QueryRequest;
use near_token::NearToken;
use serde::Deserialize;

use super::shared::staking_pool_info::StakingPoolInfo;
use crate::common::{CallResultExt, RpcQueryResponseExt};

/// Returns `None` if the validator account is not a staking pool
/// (it has no contract or the contract has no such method).
async fn call_staking_pool_view_method<T>(
    json_rpc_client: &JsonRpcClient,
    validator_account_id: &AccountId,
    method_name: &str,
) -> eyre::Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    match json_rpc_client
        .call(RpcQueryRequest {
            block_reference: Finality::Final.into(),
            request: QueryRequest::CallFunction {
                account_id: validator_account_id.clone(),
                method_name: method_name.to_string(),
                args: FunctionArgs::from(vec![]),
            },
        })
        .await
    {
        Ok(response) => Ok(Some(
            response
                .call_result()?
                .parse_result_from_json()
                .wrap_err_with(|| {
                    format!(
                        "Failed to parse return value of <{method_name}> of <{validator_account_id}>."
                    )
                })?,
        )),
        Err(JsonRpcError::ServerError(JsonRpcServerError::HandlerError(
            RpcQueryError::NoContractCode { .. } | RpcQueryError::ContractExecutionError { .. },
        ))) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

pub(super) async fn get_staking_pool_info(
    json_rpc_client: &JsonRpcClient,
    validator_account_id: AccountId,
    stake: NearToken,
) -> eyre::Result<StakingPoolInfo> {
    let fee = call_staking_pool_view_method(
        json_rpc_client,
        &validator_account_id,
        "get_reward_fee_fraction",
    )
    .await?;

    let delegators = call_staking_pool_view_method(
        json_rpc_client,
        &validator_account_id,
        "get_number_of_accounts",
    )
    .await?;

    Ok(StakingPoolInfo {
        validator_id: validator_account_id,
        fee,
        delegators,
        stake,
    })
}
