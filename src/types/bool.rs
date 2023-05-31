use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct Bool(pub bool);

impl From<Bool> for bool {
    fn from(item: Bool) -> Self {
        item.0
    }
}

impl std::fmt::Display for Bool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Bool {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bool: bool = FromStr::from_str(s).map_err(color_eyre::eyre::Report::msg)?;
        Ok(Self(bool))
    }
}

impl interactive_clap::ToCli for Bool {
    type CliVariant = Bool;
}
