#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;

use color_eyre::eyre;
use color_eyre::owo_colors::OwoColorize;
use near_primitives::types::AccountId;
use reqwest::StatusCode;
use serde::Deserialize;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use url::Url;

use super::shared::staking_pool_info::RewardFeeFraction;

#[derive(Debug, Deserialize)]
struct Validator {
    account_id: AccountId,
    fee_numerator: Option<u32>,
    fee_denominator: Option<u32>,
    delegators_count: Option<u64>,
}

#[derive(Debug, Default, Deserialize)]
struct PageMeta {
    next_page: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ValidatorsPage {
    data: Vec<Validator>,
    #[serde(default)]
    meta: PageMeta,
}

#[tracing::instrument(name = "NearBlocks API rate limit is exceeded.", skip_all)]
fn sleep_before_retry(retries_left: usize) {
    let message = format!("Waiting 15 seconds before retrying ({retries_left} retries left)");
    tracing::Span::current().pb_set_message(&message);
    tracing::info!(target: "near_teach_me", "NearBlocks API rate limit is exceeded. {message}");
    thread::sleep(Duration::from_secs(15));
}

#[allow(clippy::absolute_paths)]
fn get_validators_page(
    client: &reqwest::blocking::Client,
    nearblocks_url: &Url,
    next_page: Option<String>,
) -> eyre::Result<ValidatorsPage> {
    let mut url = nearblocks_url.join("v3/validators")?;
    url.query_pairs_mut().append_pair("limit", "100");

    if let Some(next_page) = next_page {
        url.query_pairs_mut().append_pair("next", &next_page);
    }

    // The anonymous access to NearBlocks API is limited to about 10 requests per minute,
    // so we wait for the limit to reset instead of falling back to RPC right away.
    let mut retries_left = (0..4).rev();
    loop {
        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "HTTP GET {url}",
        );
        let response = client.get(url.clone()).send()?;
        if response.status() == StatusCode::TOO_MANY_REQUESTS
            && let Some(retries_left) = retries_left.next()
        {
            sleep_before_retry(retries_left);
            continue;
        }
        return Ok(response.error_for_status()?.json()?);
    }
}

/// Returns the reward fee and the number of delegators of the given validators' staking pools.
/// Failures are only reported as warnings, since the validators missing from the result
/// are queried via RPC by the caller.
#[tracing::instrument(name = "Getting staking pools info from NearBlocks API ...", skip_all)]
pub(super) fn get_nearblocks_pools_info(
    nearblocks_url: &Url,
    validators: HashSet<&AccountId>,
) -> HashMap<AccountId, (RewardFeeFraction, u64)> {
    tracing::info!(target: "near_teach_me", "Getting staking pools info from NearBlocks API ...");

    let client = reqwest::blocking::Client::new();
    let mut staking_pools_info = HashMap::new();
    let mut validators_not_found = validators;
    let mut next_page = None;

    // NearBlocks lists all the staking pools ever created, so we stop as soon as every
    // validator is found.
    while !validators_not_found.is_empty() {
        let page = match get_validators_page(&client, nearblocks_url, next_page) {
            Ok(page) => page,
            Err(err) => {
                tracing::warn!(
                    parent: &tracing::Span::none(),
                    "{}",
                    format!(
                        "Failed to get staking pools info from NearBlocks API, falling back to RPC: {err}"
                    )
                    .red()
                );
                break;
            }
        };

        for validator in page.data {
            if !validators_not_found.remove(&validator.account_id) {
                continue;
            }
            if let (Some(numerator), Some(denominator), Some(delegators)) = (
                validator.fee_numerator,
                validator.fee_denominator,
                validator.delegators_count,
            ) {
                staking_pools_info.insert(
                    validator.account_id,
                    (
                        RewardFeeFraction {
                            numerator,
                            denominator,
                        },
                        delegators,
                    ),
                );
            }
        }
        next_page = match page.meta.next_page {
            Some(next_page) => Some(next_page),
            None => break,
        };
    }
    staking_pools_info
}
