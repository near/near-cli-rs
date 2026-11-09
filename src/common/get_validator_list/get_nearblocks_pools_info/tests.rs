use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;

use super::*;

/// Serves the given responses one per connection and returns the request lines it received.
/// One more connection is accepted afterwards, so the test has to connect once more itself
/// after the code under test is done; an unexpected extra request takes that slot instead.
fn spawn_mock_nearblocks_server(
    responses: Vec<(&'static str, &'static str)>,
) -> (SocketAddr, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let handle = thread::spawn(move || {
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
) -> (HashMap<AccountId, (RewardFeeFraction, u64)>, Vec<String>) {
    let (addr, server_handle) = spawn_mock_nearblocks_server(responses);
    let validators = validators
        .iter()
        .map(|validator_id| validator_id.parse().unwrap())
        .collect::<Vec<AccountId>>();
    let staking_pools_info = get_nearblocks_pools_info(
        &format!("http://{addr}/").parse().unwrap(),
        validators.iter().collect(),
    );
    drop(TcpStream::connect(addr).unwrap());
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

/// A page without `meta` is treated as the last one.
#[test]
fn get_nearblocks_pools_info_stops_on_the_last_page() {
    let (staking_pools_info, requests) = get_nearblocks_pools_info_from_mock_server(
        vec![(
            "200 OK",
            r#"{"data":[
                {"account_id":"a.near","fee_numerator":5,"fee_denominator":100,"delegators_count":42}
            ]}"#,
        )],
        &["a.near", "b.near"],
    );

    assert_eq!(
        staking_pools_info.keys().collect::<Vec<_>>(),
        [&"a.near".parse::<AccountId>().unwrap()]
    );
    assert_eq!(requests, ["GET /v3/validators?limit=100 HTTP/1.1", ""]);
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
        [&"a.near".parse::<AccountId>().unwrap()]
    );
    assert_eq!(requests.len(), 3);
    assert_eq!(requests[2], "");
}
