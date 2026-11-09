use near_primitives::types::AccountId;
use near_token::NearToken;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorStatus {
    /// Validates in the current and the next epoch.
    Active,
    /// Validates in the current epoch only.
    Leaving,
    /// Validates from the next epoch.
    Joining,
    /// Has only proposed to validate, so it gets a seat in the epoch after the next one
    /// at the earliest.
    Proposal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatorInfo {
    /// The stake of the next epoch, else of the current one, else the proposed one.
    pub stake: NearToken,
    pub status: ValidatorStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

// TODO Consider rename either the structure or the function name (get_validator_list)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakingPoolInfo {
    pub validator_id: AccountId,
    pub status: ValidatorStatus,
    pub fee: Option<RewardFeeFraction>,
    pub delegators: Option<u64>,
    pub stake: NearToken,
}
