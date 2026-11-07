use near_primitives::types::AccountId;
use near_token::NearToken;
use serde::Deserialize;

use crate::common::get_validator_list::shared::validator_status::ValidatorStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakingPoolInfo {
    pub validator_id: AccountId,
    pub status: ValidatorStatus,
    pub fee: Option<RewardFeeFraction>,
    pub delegators: Option<u64>,
    pub stake: NearToken,
}
