use near_token::NearToken;
use serde_json::{Value, json};

use super::*;

const PUBLIC_KEY: &str = "ed25519:6E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp";

fn current_validator(account_id: &str, stake: NearToken) -> Value {
    json!({
        "account_id": account_id,
        "public_key": PUBLIC_KEY,
        "is_slashed": false,
        "stake": stake,
        "shards": [],
        "num_produced_blocks": 0,
        "num_expected_blocks": 0,
    })
}

fn next_validator(account_id: &str, stake: NearToken) -> Value {
    json!({
        "account_id": account_id,
        "public_key": PUBLIC_KEY,
        "stake": stake,
        "shards": [],
    })
}

fn proposal(account_id: &str, stake: NearToken) -> Value {
    json!({
        "validator_stake_struct_version": "V1",
        "account_id": account_id,
        "public_key": PUBLIC_KEY,
        "stake": stake,
    })
}

#[test]
fn each_account_keeps_its_latest_stake_and_status() {
    let epoch_validator_info: EpochValidatorInfo = serde_json::from_value(json!({
        "current_validators": [
            current_validator("active.near", NearToken::from_near(100)),
            current_validator("leaving.near", NearToken::from_near(50)),
        ],
        "next_validators": [
            next_validator("active.near", NearToken::from_near(110)),
            next_validator("joining.near", NearToken::from_near(30)),
        ],
        "current_fishermen": [],
        "next_fishermen": [],
        "current_proposals": [
            proposal("active.near", NearToken::from_near(120)),
            proposal("leaving.near", NearToken::from_near(55)),
            proposal("proposal.near", NearToken::from_near(20)),
        ],
        "prev_epoch_kickout": [],
        "epoch_start_height": 1,
        "epoch_height": 1,
    }))
    .unwrap();

    assert_eq!(
        merge_validators_info(epoch_validator_info),
        HashMap::from([
            (
                "active.near".parse().unwrap(),
                ValidatorInfo {
                    stake: NearToken::from_near(110),
                    status: ValidatorStatus::Active,
                },
            ),
            (
                "leaving.near".parse().unwrap(),
                ValidatorInfo {
                    stake: NearToken::from_near(50),
                    status: ValidatorStatus::Leaving,
                },
            ),
            (
                "joining.near".parse().unwrap(),
                ValidatorInfo {
                    stake: NearToken::from_near(30),
                    status: ValidatorStatus::Joining,
                },
            ),
            (
                "proposal.near".parse().unwrap(),
                ValidatorInfo {
                    stake: NearToken::from_near(20),
                    status: ValidatorStatus::Proposal,
                },
            ),
        ]),
    );
}
