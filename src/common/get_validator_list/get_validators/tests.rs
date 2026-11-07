use near_primitives::types::ValidatorKickoutReason;
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
fn validators_get_the_status_and_stake_of_their_latest_seat() {
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
            proposal("kicked.near", NearToken::from_near(40)),
        ],
        "prev_epoch_kickout": [
            {
                "account_id": "leaving.near",
                "reason": { "NotEnoughChunkEndorsements": { "produced": 1, "expected": 10 } },
            },
            { "account_id": "kicked.near", "reason": "Unstaked" },
        ],
        "epoch_start_height": 1,
        "epoch_height": 1,
    }))
    .unwrap();

    assert_eq!(
        collect_validators(epoch_validator_info),
        HashMap::from([
            (
                "active.near".parse().unwrap(),
                (NearToken::from_near(110), ValidatorStatus::Active),
            ),
            (
                "leaving.near".parse().unwrap(),
                (
                    NearToken::from_near(50),
                    ValidatorStatus::Leaving {
                        kickout_reason: Some(ValidatorKickoutReason::NotEnoughChunkEndorsements {
                            produced: 1,
                            expected: 10,
                        }),
                    },
                ),
            ),
            (
                "joining.near".parse().unwrap(),
                (NearToken::from_near(30), ValidatorStatus::Joining),
            ),
            (
                "proposal.near".parse().unwrap(),
                (
                    NearToken::from_near(20),
                    ValidatorStatus::Proposal {
                        kickout_reason: None,
                    },
                ),
            ),
            (
                "kicked.near".parse().unwrap(),
                (
                    NearToken::from_near(40),
                    ValidatorStatus::Proposal {
                        kickout_reason: Some(ValidatorKickoutReason::Unstaked),
                    },
                ),
            ),
        ]),
    );
}
