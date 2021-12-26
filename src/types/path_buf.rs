#[derive(Debug, Default, Clone)]
pub struct PathBuf(pub std::path::PathBuf);

impl From<PathBuf> for std::path::PathBuf {
    fn from(path_buf: PathBuf) -> Self {
        path_buf.0
    }
}

impl From<std::path::PathBuf> for PathBuf {
    fn from(path_buf: std::path::PathBuf) -> Self {
        Self(path_buf)
    }
}

impl std::fmt::Display for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl std::str::FromStr for PathBuf {
    type Err = core::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let path_buf = std::path::PathBuf::from_str(s)?;
        Ok(Self(path_buf))
    }
}

impl interactive_clap::ToCli for PathBuf {
    type CliVariant = PathBuf;
}
