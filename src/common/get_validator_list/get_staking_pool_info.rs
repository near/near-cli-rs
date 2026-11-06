use crate::common::{CallResultExt, RpcQueryResponseExt};
use color_eyre::eyre::Context;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakingPoolInfo {
    pub validator_id: near_primitives::types::AccountId,
    pub fee: Option<RewardFeeFraction>,
    pub delegators: Option<u64>,
    pub stake: near_primitives::types::Balance,
}

pub(super) async fn get_staking_pool_info(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    validator_account_id: near_primitives::types::AccountId,
    stake: near_token::NearToken,
) -> color_eyre::Result<StakingPoolInfo> {
    let fee = match json_rpc_client
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: validator_account_id.clone(),
                method_name: "get_reward_fee_fraction".to_string(),
                args: near_primitives::types::FunctionArgs::from(vec![]),
            },
        })
        .await
    {
        Ok(response) => Some(
            response
                .call_result()?
                .parse_result_from_json::<RewardFeeFraction>()
                .wrap_err(
                    "Failed to parse return value of view function call for RewardFeeFraction.",
                )?,
        ),
        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                near_jsonrpc_client::methods::query::RpcQueryError::NoContractCode { .. }
                | near_jsonrpc_client::methods::query::RpcQueryError::ContractExecutionError {
                    ..
                },
            ),
        )) => None,
        Err(err) => return Err(err.into()),
    };

    let delegators = match json_rpc_client
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: near_primitives::types::Finality::Final.into(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: validator_account_id.clone(),
                method_name: "get_number_of_accounts".to_string(),
                args: near_primitives::types::FunctionArgs::from(vec![]),
            },
        })
        .await
    {
        Ok(response) => Some(
            response
                .call_result()?
                .parse_result_from_json::<u64>()
                .wrap_err("Failed to parse return value of view function call for u64.")?,
        ),
        Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                near_jsonrpc_client::methods::query::RpcQueryError::NoContractCode { .. }
                | near_jsonrpc_client::methods::query::RpcQueryError::ContractExecutionError {
                    ..
                },
            ),
        )) => None,
        Err(err) => return Err(err.into()),
    };

    Ok(StakingPoolInfo {
        validator_id: validator_account_id.clone(),
        fee,
        delegators,
        stake,
    })
}
