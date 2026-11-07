use super::*;

fn validator_with_stake(stake: near_token::NearToken) -> StakingPoolInfo {
    StakingPoolInfo {
        validator_id: "test.poolv1.near".parse().unwrap(),
        fee: None,
        delegators: None,
        stake,
    }
}

#[test]
fn stake_is_formatted_with_thousands_separators() {
    for (stake, expected) in [
        (near_token::NearToken::from_yoctonear(0), "0 NEAR"),
        (near_token::NearToken::from_millinear(499), "0 NEAR"),
        (near_token::NearToken::from_millinear(500), "1 NEAR"),
        (near_token::NearToken::from_near(999), "999 NEAR"),
        (near_token::NearToken::from_near(1_000), "1,000 NEAR"),
        (near_token::NearToken::from_near(123_456), "123,456 NEAR"),
        (
            near_token::NearToken::from_millinear(33_272_032_170),
            "33,272,032 NEAR",
        ),
    ] {
        assert_eq!(
            format_validator_stake(&validator_with_stake(stake)),
            expected
        );
    }
}
