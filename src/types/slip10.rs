#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BIP32Path(pub slip10::BIP32Path);

impl std::fmt::Display for BIP32Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for BIP32Path {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bip32path = slip10::BIP32Path::from_str(s).map_err(Self::Err::msg)?;
        Ok(Self(bip32path))
    }
}

impl From<BIP32Path> for slip10::BIP32Path {
    fn from(item: BIP32Path) -> Self {
        item.0
    }
}

impl serde::ser::Serialize for BIP32Path {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> serde::de::Deserialize<'de> for BIP32Path {
    fn deserialize<D>(deserializer: D) -> Result<BIP32Path, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        String::deserialize(deserializer)?.parse().map_err(|err: color_eyre::eyre::Report| serde::de::Error::custom(err.to_string()))
    }
}

impl interactive_clap::ToCli for crate::types::slip10::BIP32Path {
    type CliVariant = crate::types::slip10::BIP32Path;
}
