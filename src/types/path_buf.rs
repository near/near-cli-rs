#[derive(
    Debug,
    Default,
    Clone,
    derive_more::AsRef,
    derive_more::From,
    derive_more::Into,
    derive_more::FromStr,
)]
#[as_ref(forward)]
pub struct PathBuf(pub std::path::PathBuf);

impl std::fmt::Display for PathBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

impl interactive_clap::ToCli for PathBuf {
    type CliVariant = PathBuf;
}
