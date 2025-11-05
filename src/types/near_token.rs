const ONE_NEAR: u128 = 10u128.pow(24);

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
    derive_more::AsRef,
    derive_more::From,
    derive_more::Into,
    derive_more::FromStr,
)]
#[as_ref(forward)]
pub struct NearToken(pub near_token::NearToken);

impl std::fmt::Display for NearToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.as_yoctonear() == 0 {
            write!(f, "0 NEAR")
        } else if self.as_yoctonear() <= 1_000 {
            write!(f, "{} yoctoNEAR", self.as_yoctonear())
        } else if self.as_yoctonear().is_multiple_of(ONE_NEAR) {
            write!(f, "{} NEAR", self.as_yoctonear() / ONE_NEAR,)
        } else {
            write!(
                f,
                "{}.{} NEAR",
                self.as_yoctonear() / ONE_NEAR,
                format!("{:0>24}", (self.as_yoctonear() % ONE_NEAR)).trim_end_matches('0')
            )
        }
    }
}

impl NearToken {
    pub fn as_yoctonear(&self) -> u128 {
        self.0.as_yoctonear()
    }

    pub fn from_yoctonear(inner: u128) -> Self {
        Self(near_token::NearToken::from_yoctonear(inner))
    }

    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl interactive_clap::ToCli for NearToken {
    type CliVariant = NearToken;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn near_token_to_string_0_near() {
        assert_eq!(
            NearToken(near_token::NearToken::from_near(0)).to_string(),
            "0 NEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_0_millinear() {
        assert_eq!(
            NearToken(near_token::NearToken::from_millinear(0)).to_string(),
            "0 NEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_0_yoctonear() {
        assert_eq!(
            NearToken(near_token::NearToken::from_yoctonear(0)).to_string(),
            "0 NEAR".to_string()
        )
    }

    #[test]
    fn near_token_to_string_0dot02_near() {
        assert_eq!(
            NearToken(near_token::NearToken::from_yoctonear(
                20_000_000_000_000_000_000_000
            ))
            .to_string(),
            "0.02 NEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_0dot00001230045600789_near() {
        assert_eq!(
            NearToken(
                near_token::NearToken::from_str("0.000012300456007890000000 Near")
                    .unwrap_or_default()
            )
            .to_string(),
            "0.00001230045600789 NEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_10_near() {
        assert_eq!(
            NearToken(near_token::NearToken::from_yoctonear(
                10_000_000_000_000_000_000_000_000
            ))
            .to_string(),
            "10 NEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_10dot02_000_01near() {
        assert_eq!(
            NearToken(near_token::NearToken::from_yoctonear(
                10_020_000_000_000_000_000_000_001
            ))
            .to_string(),
            "10.020000000000000000000001 NEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_1_yocto_near() {
        assert_eq!(
            NearToken(near_token::NearToken::from_yoctonear(1)).to_string(),
            "1 yoctoNEAR".to_string()
        )
    }
    #[test]
    fn near_token_to_string_100_yocto_near() {
        assert_eq!(
            NearToken(near_token::NearToken::from_yoctonear(100)).to_string(),
            "100 yoctoNEAR".to_string()
        )
    }
}
