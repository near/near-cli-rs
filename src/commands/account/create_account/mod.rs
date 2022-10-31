use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod create_implicit_account;
mod create_new_account;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CreateAccount {
    #[interactive_clap(subcommand)]
    account_actions: CoverCostsCreateAccount,
}

impl CreateAccount {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.account_actions.process(config).await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///How do you cover the costs of account creation?
pub enum CoverCostsCreateAccount {
    #[strum_discriminants(strum(
        message = "sponsor-by-linkdrop  - I would like the sponsor linkdrop to cover the cost of creating an account"
    ))]
    ///I would like the sponsor linkdrop to cover the cost of creating an account
    SponsorByLinkdrop,
    #[strum_discriminants(strum(
        message = "sponsor-by-wallet    - I would like the sponsor by wallet to cover the cost of creating an account (testnet only)"
    ))]
    ///I would like the sponsor by wallet to cover the cost of creating an account (testnet only)
    SponsorByWallet,
    #[strum_discriminants(strum(
        message = "fund-myself          - I would like fund myself to cover the cost of creating an account"
    ))]
    ///I would like fund myself to cover the cost of creating an account
    FundMyself(self::create_new_account::NewAccount),
    #[strum_discriminants(strum(message = "fund-later           - Create an implicit-account"))]
    ///Create an implicit-account
    FundLater(self::create_implicit_account::ImplicitAccount),
}

impl CoverCostsCreateAccount {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        match self {
            Self::SponsorByLinkdrop => todo!(),
            Self::SponsorByWallet => todo!(),
            Self::FundMyself(new_account) => new_account.process(config).await,
            Self::FundLater(implicit_account) => implicit_account.process().await,
        }
    }
}
