const ONE_NEAR: u128 = 10u128.pow(24);
const UNLIMITED: &str = "unlimited";

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    serde::Serialize,
    serde::Deserialize,
    derive_more::AsRef,
    derive_more::From,
    derive_more::Into,
)]
#[as_ref(forward)]
pub struct NearAllowance(Option<near_token::NearToken>);

impl std::fmt::Display for NearAllowance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(amount) = self.0 {
            amount.fmt(f)
        } else {
            write!(f, "{UNLIMITED}")
        }
    }
}

impl std::str::FromStr for NearAllowance {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == UNLIMITED {
            return Ok(Self(None));
        }
        Ok(Self(Some(near_token::NearToken::from_str(s)?)))
    }
}

impl NearAllowance {
    pub fn optional_near_token(&self) -> Option<near_token::NearToken> {
        self.0
    }
}

impl interactive_clap::ToCli for NearAllowance {
    type CliVariant = NearAllowance;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn near_allowance_to_string_0_near() {
        assert_eq!(
            NearAllowance(Some(near_token::NearToken::from_near(0))).to_string(),
            "0 NEAR".to_string()
        )
    }
    #[test]
    fn near_allowance_to_string_0_millinear() {
        assert_eq!(
            NearAllowance(Some(near_token::NearToken::from_millinear(0))).to_string(),
            "0 NEAR".to_string()
        )
    }
    #[test]
    fn near_allowance_to_string_none() {
        assert_eq!(NearAllowance(None).to_string(), "unlimited".to_string())
    }
    #[test]
    fn near_allowance_to_string_0dot02_near() {
        assert_eq!(
            NearAllowance(Some(near_token::NearToken::from_yoctonear(
                20_000_000_000_000_000_000_000
            )))
            .to_string(),
            "0.02 NEAR".to_string()
        )
    }
    #[test]
    fn near_allowance_from_str_unlimited() {
        assert_eq!(
            NearAllowance::from_str("unlimited")
                .unwrap_or_default()
                .optional_near_token(),
            None
        )
    }
}
