use near_primitives::types::ValidatorKickoutReason;

/// Whether the validator has a seat in the current and the next epoch.
/// `kickout_reason` says why the validator was kicked out at the start of the current epoch,
/// if it was: a leaving validator loses its seat in the next epoch for that reason, and a pool
/// that only proposes may be trying to get its seat back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorStatus {
    /// Validates in the current and the next epoch.
    Active,
    /// Validates in the current epoch only.
    Leaving {
        kickout_reason: Option<ValidatorKickoutReason>,
    },
    /// Validates from the next epoch.
    Joining,
    /// Has only proposed to validate, so it gets a seat in the epoch after the next one
    /// at the earliest.
    Proposal {
        kickout_reason: Option<ValidatorKickoutReason>,
    },
}
