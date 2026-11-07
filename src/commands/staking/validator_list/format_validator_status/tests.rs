use super::*;

fn validator_with_status(status: ValidatorStatus) -> StakingPoolInfo {
    StakingPoolInfo {
        validator_id: "test.poolv1.near".parse().unwrap(),
        status,
        fee: None,
        delegators: None,
        stake: near_token::NearToken::from_near(1),
    }
}

#[test]
fn status_is_formatted_with_the_kickout_reason() {
    for (status, expected) in [
        (ValidatorStatus::Active, "Active"),
        (ValidatorStatus::Joining, "Joining"),
        (
            ValidatorStatus::Leaving {
                kickout_reason: Some(ValidatorKickoutReason::NotEnoughChunkEndorsements {
                    produced: 1,
                    expected: 10,
                }),
            },
            "Leaving (few endorsements)",
        ),
        (
            ValidatorStatus::Leaving {
                kickout_reason: None,
            },
            "Leaving",
        ),
        (
            ValidatorStatus::Proposal {
                kickout_reason: Some(ValidatorKickoutReason::Unstaked),
            },
            "Proposal (unstaked)",
        ),
        (
            ValidatorStatus::Proposal {
                kickout_reason: None,
            },
            "Proposal",
        ),
    ] {
        assert_eq!(
            format_validator_status(&validator_with_status(status)),
            expected
        );
    }
}
