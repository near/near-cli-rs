use color_eyre::eyre::Context;
use futures::StreamExt;

use crate::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewAccountSummaryContext)]
pub struct ViewAccountSummary {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewAccountSummaryContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewAccountSummaryContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewAccountSummary as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

            move |network_config, block_reference| {
                let rpc_query_response = network_config
                    .json_rpc_client()
                    .blocking_call_view_account(&account_id, block_reference.clone())
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch query ViewAccount for <{}>",
                            &account_id
                        )
                    })?;
                let account_view = rpc_query_response.account_view()?;

                let access_key_list = json_rpc_client
                    .blocking_call_view_access_key_list(
                        &account_id,
                        block_reference.clone(),
                    )
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch ViewAccessKeyList for {}",
                            &account_id
                        )
                    })?
                    .access_key_list_view()?;

                let validators_stake = crate::common::get_validators_stake(&json_rpc_client)?;

                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()?;
                let chunk_size = 15;
                let concurrency = 10;

                let delegated_stake: std::collections::HashMap<near_primitives::types::AccountId, crate::common::NearBalance> = runtime
                .block_on(
                    futures::stream::iter(validators_stake
                        .iter()
                        .collect::<Vec<_>>()
                        .chunks(chunk_size),
                    )
                    .map(|validators| async {
                        get_delegated_stake(&json_rpc_client, block_reference, &account_id, validators).await
                    })
                    .buffer_unordered(concurrency)
                    .collect::<Vec<Result<_, _>>>(),
                )
                .into_iter()
                .try_fold(std::collections::HashMap::new(), |mut acc, x| {
                    acc.extend(x?);
                    Ok::<_, color_eyre::eyre::Error>(acc)
                })?;

                crate::common::display_account_info(
                    &rpc_query_response.block_hash,
                    &rpc_query_response.block_height,
                    &account_id,
                    delegated_stake,
                    &account_view,
                    &access_key_list.keys,
                );
                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewAccountSummaryContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewAccountSummaryContext) -> Self {
        item.0
    }
}

impl ViewAccountSummary {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What Account ID do you need to view?",
        )
    }
}

async fn get_delegated_stake(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &near_primitives::types::BlockReference,
    account_id: &near_primitives::types::AccountId,
    validators: &[(&near_primitives::types::AccountId, &u128)],
) -> color_eyre::Result<
    std::collections::HashMap<near_primitives::types::AccountId, crate::common::NearBalance>,
> {
    let mut validators_stake: std::collections::HashMap<
        near_primitives::types::AccountId,
        crate::common::NearBalance,
    > = std::collections::HashMap::new();
    for (validator_id, _) in validators {
        let validator_id = *validator_id;
        let user_staked_balance =
            get_user_staked_balance(json_rpc_client, block_reference, validator_id, account_id)
                .await
                .ok();
        let balance = if let Some(balance) = user_staked_balance {
            if balance == 0 {
                continue;
            } else {
                balance
            }
        } else {
            continue;
        };
        validators_stake.insert(
            validator_id.clone(),
            crate::common::NearBalance::from_yoctonear(balance),
        );
    }
    Ok(validators_stake)
}

async fn get_user_staked_balance(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    block_reference: &near_primitives::types::BlockReference,
    validator_account_id: &near_primitives::types::AccountId,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<u128> {
    Ok(json_rpc_client
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: block_reference.clone(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: validator_account_id.clone(),
                method_name: "get_account_staked_balance".to_string(),
                args: near_primitives::types::FunctionArgs::from(
                    serde_json::json!({
                        "account_id": account_id,
                    })
                    .to_string()
                    .into_bytes(),
                ),
            },
        })
        .await
        .wrap_err("Failed to fetch query for view method: 'get_account_staked_balance'")?
        .call_result()?
        .parse_result_from_json::<String>()
        .wrap_err("Failed to parse return value of view function call for String.")?
        .parse::<u128>()?)
}
