#[cfg(test)]
mod tests;

use near_primitives::types::ValidatorKickoutReason;

use crate::common::{StakingPoolInfo, ValidatorStatus};

/// A short label for the reason a validator was kicked out, so the table stays narrow.
fn format_kickout_reason(kickout_reason: &ValidatorKickoutReason) -> &'static str {
    match kickout_reason {
        ValidatorKickoutReason::_UnusedSlashed => "slashed",
        ValidatorKickoutReason::NotEnoughBlocks { .. } => "few blocks",
        ValidatorKickoutReason::NotEnoughChunks { .. } => "few chunks",
        ValidatorKickoutReason::NotEnoughChunkEndorsements { .. } => "few endorsements",
        ValidatorKickoutReason::Unstaked => "unstaked",
        ValidatorKickoutReason::NotEnoughStake { .. } => "low stake",
        ValidatorKickoutReason::DidNotGetASeat => "no seat",
        ValidatorKickoutReason::ProtocolVersionTooOld { .. } => "old protocol",
    }
}

/// Formats the status with the kickout reason in parentheses, if there is one
/// (e.g. `Leaving (few endorsements)`).
pub fn format_validator_status(validator: &StakingPoolInfo) -> String {
    let (status, kickout_reason) = match &validator.status {
        ValidatorStatus::Active => ("Active", None),
        ValidatorStatus::Leaving { kickout_reason } => ("Leaving", kickout_reason.as_ref()),
        ValidatorStatus::Joining => ("Joining", None),
        ValidatorStatus::Proposal { kickout_reason } => ("Proposal", kickout_reason.as_ref()),
    };

    match kickout_reason {
        Some(kickout_reason) => format!("{status} ({})", format_kickout_reason(kickout_reason)),
        None => status.to_string(),
    }
}
