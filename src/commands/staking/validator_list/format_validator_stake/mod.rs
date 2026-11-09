#[cfg(test)]
mod tests;

use crate::common::StakingPoolInfo;

/// Formats the stake as whole NEAR with thousands separators (e.g. `33,272,032 NEAR`),
/// since fractions of NEAR are noise for validator-sized stakes.
pub fn format_validator_stake(validator: &StakingPoolInfo) -> String {
    const ONE_NEAR: u128 = 10u128.pow(24);

    let digits = ((validator.stake.as_yoctonear() + ONE_NEAR / 2) / ONE_NEAR).to_string();
    let mut stake = String::with_capacity(digits.len() + digits.len() / 3 + 5);

    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            stake.push(',');
        }
        stake.push(digit);
    }
    stake.push_str(" NEAR");

    stake
}
