use color_eyre::eyre::Context;
use futures::StreamExt;
use prettytable::Table;

use near_jsonrpc_client::methods::validators::RpcValidatorRequest;

use crate::common::{CallResultExt, JsonRpcClientExt, RpcQueryResponseExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ValidatorListContext)]
pub struct ValidatorList {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct ValidatorListContext(crate::network::NetworkContext);

impl ValidatorListContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<ValidatorList as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(display_validators_info);
        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            on_after_getting_network_callback,
        }))
    }
}

impl From<ValidatorListContext> for crate::network::NetworkContext {
    fn from(item: ValidatorListContext) -> Self {
        item.0
    }
}

pub fn display_validators_info(network_config: &crate::config::NetworkConfig) -> crate::CliResult {
    let mut table = Table::new();
    table.set_titles(prettytable::row![Fg=>"#", "Validator Id", "Fee", "Delegators", "Stake"]);

    for (index, validator) in get_validator_list(network_config)?.into_iter().enumerate() {
        table.add_row(prettytable::row![
            Fg->index + 1,
            validator.validator_id,
            format!("{:>6.2} %", validator.fee.numerator * 100 / validator.fee.denominator),
            validator.delegators,
            crate::common::NearBalance::from_yoctonear(validator.stake),
        ]);
    }
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorsTable {
    pub validator_id: near_primitives::types::AccountId,
    pub fee: RewardFeeFraction,
    pub delegators: u64,
    pub stake: near_primitives::types::Balance,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

pub fn get_validator_list(
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<Vec<ValidatorsTable>> {
    let json_rpc_client = network_config.json_rpc_client();

    let epoch_validator_info = json_rpc_client
        .blocking_call(&RpcValidatorRequest {
            epoch_reference: near_primitives::types::EpochReference::Latest,
        })
        .wrap_err("Failed to get epoch validators information request.")?;

    let current_proposals = epoch_validator_info.current_proposals;
    let current_proposals_stake: std::collections::HashMap<
        near_primitives::types::AccountId,
        near_primitives::types::Balance,
    > = current_proposals
        .into_iter()
        .map(|validator_stake_view| {
            let validator_stake = validator_stake_view.into_validator_stake();
            validator_stake.account_and_stake()
        })
        .collect();

    let current_validators = epoch_validator_info.current_validators;
    let mut current_validators_stake: std::collections::HashMap<
        near_primitives::types::AccountId,
        near_primitives::types::Balance,
    > = current_validators
        .into_iter()
        .map(|current_epoch_validator_info| {
            (
                current_epoch_validator_info.account_id,
                current_epoch_validator_info.stake,
            )
        })
        .collect();

    let next_validators = epoch_validator_info.next_validators;
    let next_validators_stake: std::collections::HashMap<
        near_primitives::types::AccountId,
        near_primitives::types::Balance,
    > = next_validators
        .into_iter()
        .map(|next_epoch_validator_info| {
            (
                next_epoch_validator_info.account_id,
                next_epoch_validator_info.stake,
            )
        })
        .collect();

    current_validators_stake.extend(next_validators_stake);
    current_validators_stake.extend(current_proposals_stake);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let chunk_size = 15;
    let concurrency = 10;

    let combine_validators: std::collections::HashMap<
        near_primitives::types::AccountId,
        ValidatorsTable,
    > = runtime
        .block_on(
            futures::stream::iter(
                current_validators_stake
                    .iter()
                    .collect::<Vec<_>>()
                    .chunks(chunk_size),
            )
            .map(
                |validators: &[(&near_primitives::types::AccountId, &u128)]| async {
                    get_combine_validators(&json_rpc_client, validators).await
                },
            )
            .buffer_unordered(concurrency)
            .collect::<Vec<Result<_, _>>>(),
        )
        .into_iter()
        .try_fold(std::collections::HashMap::new(), |mut acc, x| {
            acc.extend(x?);
            Ok::<_, color_eyre::eyre::Error>(acc)
        })?;

    let mut validator_list: Vec<ValidatorsTable> = combine_validators.into_values().collect();
    validator_list.sort_by(|a, b| b.stake.cmp(&a.stake));
    Ok(validator_list)
}

async fn get_combine_validators(
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    validators: &[(&near_primitives::types::AccountId, &u128)],
) -> color_eyre::Result<std::collections::HashMap<near_primitives::types::AccountId, ValidatorsTable>>
{
    let mut combine_validators: std::collections::HashMap<
        near_primitives::types::AccountId,
        ValidatorsTable,
    > = std::collections::HashMap::new();
    for (validator_id, stake) in validators {
        let validator_id = *validator_id;
        let reward_fee_fraction = json_rpc_client
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: validator_id.clone(),
                    method_name: "get_reward_fee_fraction".to_string(),
                    args: near_primitives::types::FunctionArgs::from(vec![]),
                },
            })
            .await
            .wrap_err("Failed to fetch query for view method: 'get_reward_fee_fraction'");

        if reward_fee_fraction.is_err() {
            continue;
        };
        let fee = reward_fee_fraction?
            .call_result()?
            .parse_result_from_json::<RewardFeeFraction>()
            .wrap_err(
                "Failed to parse return value of view function call for RewardFeeFraction.",
            )?;

        let number_of_accounts = json_rpc_client
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: validator_id.clone(),
                    method_name: "get_number_of_accounts".to_string(),
                    args: near_primitives::types::FunctionArgs::from(vec![]),
                },
            })
            .await
            .wrap_err("Failed to fetch query for view method: 'get_number_of_accounts'");
        if number_of_accounts.is_err() {
            continue;
        };
        let delegators = number_of_accounts?
            .call_result()?
            .parse_result_from_json::<u64>()
            .wrap_err("Failed to parse return value of view function call for u64.")?;

        let validators_table = ValidatorsTable {
            validator_id: validator_id.clone(),
            fee,
            delegators,
            stake: **stake,
        };
        combine_validators.insert(validator_id.clone(), validators_table);
    }
    Ok(combine_validators)
}
