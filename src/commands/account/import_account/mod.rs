use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod from_web_wallet;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct ImportAccountCommand {
    #[interactive_clap(subcommand)]
    import_account_actions: ImportAccountActions,
}

impl ImportAccountCommand {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.import_account_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to import the account?
pub enum ImportAccountActions {
    #[strum_discriminants(strum(
        message = "import-account-from-web-wallet          - Import existing account (a.k.a. \"sign in\")"
    ))]
    /// Import existing account (a.k.a. "sign in")
    ImportAccountFromWebWallet(self::from_web_wallet::Login),
}

impl ImportAccountActions {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::ImportAccountFromWebWallet(login) => login.process(config).await,
        }
    }
}
