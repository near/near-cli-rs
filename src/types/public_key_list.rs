use interactive_clap::ToCli;

#[derive(Debug, Clone)]
pub struct PublicKeyList(Vec<near_kit::PublicKey>);

impl std::fmt::Display for PublicKeyList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let keys: Vec<String> = self.0.iter().map(|key| key.to_string()).collect();
        write!(f, "{}", keys.join(","))
    }
}

impl std::str::FromStr for PublicKeyList {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = s
            .split(',')
            .map(|str| crate::types::public_key::parse_public_key(str.trim()))
            .collect::<color_eyre::eyre::Result<Vec<near_kit::PublicKey>>>()?;
        Ok(Self(keys))
    }
}

impl From<PublicKeyList> for Vec<near_kit::PublicKey> {
    fn from(item: PublicKeyList) -> Self {
        item.0
    }
}

impl From<Vec<near_kit::PublicKey>> for PublicKeyList {
    fn from(item: Vec<near_kit::PublicKey>) -> Self {
        Self(item)
    }
}

impl ToCli for PublicKeyList {
    type CliVariant = PublicKeyList;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_comma_separated_keys() {
        let list: PublicKeyList =
            "ed25519:5387nnYC7uiWrrPevw7FAopaL8hfr6dZVJqpg6HPrPKr, ed25519:2c2nCne4by3DPS1hqU9naihMpLtz1DGhCUbVu52XxoK1"
                .parse()
                .unwrap();
        assert_eq!(list.0.len(), 2);
    }

    #[test]
    fn rejects_ml_dsa_65_hash_handle_in_list() {
        let err = "ed25519:5387nnYC7uiWrrPevw7FAopaL8hfr6dZVJqpg6HPrPKr,ml-dsa-65-hash:11111111111111111111111111111111"
            .parse::<PublicKeyList>()
            .unwrap_err();
        assert!(
            err.to_string().contains("not a public key"),
            "unexpected error: {err}"
        );
    }
}
