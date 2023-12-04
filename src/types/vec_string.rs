#[derive(Debug, Default, Clone)]
pub struct VecString(pub Vec<String>);

impl std::fmt::Display for VecString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\n{}", self.0.join("\n"))
    }
}

impl std::str::FromStr for VecString {
    type Err = color_eyre::eyre::ErrReport;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec_str: Vec<String> = s.split(',').map(|str| str.trim().to_string()).collect();
        Ok(Self(vec_str))
    }
}

impl From<VecString> for Vec<String> {
    fn from(item: VecString) -> Self {
        item.0
    }
}

impl From<Vec<String>> for VecString {
    fn from(item: Vec<String>) -> Self {
        Self(item)
    }
}

impl interactive_clap::ToCli for VecString {
    type CliVariant = VecString;
}
