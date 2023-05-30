use interactive_clap::ToCli;

#[derive(Debug, Clone)]
pub struct PublicKeyList(pub Vec<crate::types::public_key::PublicKey>);
impl std::fmt::Display for PublicKeyList {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let keys: Vec<String> = self.0.iter().map(|key| key.to_string()).collect();
        write!(f, "{}", keys.join(","))
    }
}

impl std::str::FromStr for PublicKeyList {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys: Vec<crate::types::public_key::PublicKey> = s
            .split(',')
            .map(|str| str.trim().parse())
            .collect::<Result<Vec<crate::types::public_key::PublicKey>, _>>()?;
        Ok(Self(keys))
    }
}

impl From<PublicKeyList> for Vec<crate::types::public_key::PublicKey> {
    fn from(item: PublicKeyList) -> Self {
        item.0
    }
}

impl ToCli for PublicKeyList {
    type CliVariant = PublicKeyList;
}
