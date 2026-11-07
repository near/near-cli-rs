use super::_common::staking_pool_info::RewardFeeFraction;
use color_eyre::owo_colors::OwoColorize;
use std::collections::{HashMap, HashSet};
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Debug, serde::Deserialize)]
struct ValidatorsPage {
    data: Vec<Validator>,
    #[serde(default)]
    meta: PageMeta,
}

#[derive(Debug, Default, serde::Deserialize)]
struct PageMeta {
    next_page: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct Validator {
    account_id: near_primitives::types::AccountId,
    fee_numerator: Option<u32>,
    fee_denominator: Option<u32>,
    delegators_count: Option<u64>,
}

#[tracing::instrument(name = "NearBlocks API rate limit is exceeded.", skip_all)]
fn sleep_before_retry(retries_left: usize) {
    let message = format!("Waiting 15 seconds before retrying ({retries_left} retries left)");
    tracing::Span::current().pb_set_message(&message);
    tracing::info!(target: "near_teach_me", "NearBlocks API rate limit is exceeded. {message}");
    std::thread::sleep(std::time::Duration::from_secs(15));
}

fn get_validators_page(
    client: &reqwest::blocking::Client,
    nearblocks_url: &url::Url,
    next_page: Option<String>,
) -> color_eyre::eyre::Result<ValidatorsPage> {
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
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
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
pub(super) fn get_nearblocks_pools_info<'a>(
    nearblocks_url: &url::Url,
    validators: impl IntoIterator<Item = &'a near_primitives::types::AccountId>,
) -> HashMap<near_primitives::types::AccountId, (RewardFeeFraction, u64)> {
    tracing::info!(target: "near_teach_me", "Getting staking pools info from NearBlocks API ...");

    let client = reqwest::blocking::Client::new();
    let mut staking_pools_info = HashMap::new();
    let mut validators_not_found = validators.into_iter().collect::<HashSet<_>>();
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

#[cfg(test)]
mod tests {
    use std::io::{Read, Write};

    use super::*;

    /// Serves the given responses one per connection and returns the request lines it received.
    /// One more connection is accepted afterwards, so the test has to connect once more itself
    /// after the code under test is done; an unexpected extra request takes that slot instead.
    fn spawn_mock_nearblocks_server(
        responses: Vec<(&'static str, &'static str)>,
    ) -> (std::net::SocketAddr, std::thread::JoinHandle<Vec<String>>) {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let mut responses = responses.into_iter();
            let mut requests = vec![];
            for _ in 0..=responses.len() {
                let (mut stream, _) = listener.accept().unwrap();
                let mut buffer = [0u8; 4096];
                let len = stream.read(&mut buffer).unwrap_or_default();
                requests.push(
                    String::from_utf8_lossy(&buffer[..len])
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                );
                if let Some((status, body)) = responses.next() {
                    let _ = write!(
                        stream,
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                }
            }
            requests
        });
        (addr, handle)
    }

    fn get_nearblocks_pools_info_from_mock_server(
        responses: Vec<(&'static str, &'static str)>,
        validators: &[&str],
    ) -> (
        HashMap<near_primitives::types::AccountId, (RewardFeeFraction, u64)>,
        Vec<String>,
    ) {
        let (addr, server_handle) = spawn_mock_nearblocks_server(responses);
        let validators = validators
            .iter()
            .map(|validator_id| validator_id.parse().unwrap())
            .collect::<Vec<near_primitives::types::AccountId>>();
        let staking_pools_info =
            get_nearblocks_pools_info(&format!("http://{addr}/").parse().unwrap(), &validators);
        drop(std::net::TcpStream::connect(addr).unwrap());
        (staking_pools_info, server_handle.join().unwrap())
    }

    #[test]
    fn get_nearblocks_pools_info_stops_when_all_validators_are_found() {
        let (staking_pools_info, requests) = get_nearblocks_pools_info_from_mock_server(
            vec![
                (
                    "200 OK",
                    r#"{"data":[
                        {"account_id":"a.near","fee_numerator":5,"fee_denominator":100,"delegators_count":42},
                        {"account_id":"x.near","fee_numerator":1,"fee_denominator":100,"delegators_count":7}
                    ],"meta":{"next_page":"cursor-1"}}"#,
                ),
                (
                    "200 OK",
                    r#"{"data":[
                        {"account_id":"b.near","fee_numerator":null,"fee_denominator":null,"delegators_count":null}
                    ],"meta":{"next_page":"cursor-2","prev_page":"cursor-0"}}"#,
                ),
            ],
            &["a.near", "b.near"],
        );

        assert_eq!(
            staking_pools_info,
            HashMap::from([(
                "a.near".parse().unwrap(),
                (
                    RewardFeeFraction {
                        numerator: 5,
                        denominator: 100
                    },
                    42
                )
            )])
        );
        assert_eq!(
            requests,
            [
                "GET /v3/validators?limit=100 HTTP/1.1",
                "GET /v3/validators?limit=100&next=cursor-1 HTTP/1.1",
                "",
            ]
        );
    }

    #[test]
    fn get_nearblocks_pools_info_keeps_fetched_pages_on_error() {
        let (staking_pools_info, requests) = get_nearblocks_pools_info_from_mock_server(
            vec![
                (
                    "200 OK",
                    r#"{"data":[
                        {"account_id":"a.near","fee_numerator":5,"fee_denominator":100,"delegators_count":42}
                    ],"meta":{"next_page":"cursor-1"}}"#,
                ),
                ("500 Internal Server Error", "{}"),
            ],
            &["a.near", "b.near"],
        );

        assert_eq!(
            staking_pools_info.keys().collect::<Vec<_>>(),
            [&"a.near"
                .parse::<near_primitives::types::AccountId>()
                .unwrap()]
        );
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[2], "");
    }
}
