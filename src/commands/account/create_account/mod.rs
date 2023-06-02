#![allow(clippy::enum_variant_names, clippy::large_enum_variant)]
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod create_implicit_account;
mod fund_myself_create_account;
mod sponsor_by_faucet_service;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CreateAccount {
    #[interactive_clap(subcommand)]
    account_actions: CoverCostsCreateAccount,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = CoverCostsCreateAccountContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How do you cover the costs of account creation?
pub enum CoverCostsCreateAccount {
    #[strum_discriminants(strum(
        message = "sponsor-by-faucet-service    - I would like the faucet service sponsor to cover the cost of creating an account (testnet only for now)"
    ))]
    /// I would like the faucet service sponsor to cover the cost of creating an account (testnet only for now)
    SponsorByFaucetService(self::sponsor_by_faucet_service::NewAccount),
    #[strum_discriminants(strum(
        message = "fund-myself                  - I would like fund myself to cover the cost of creating an account"
    ))]
    /// I would like fund myself to cover the cost of creating an account
    FundMyself(self::fund_myself_create_account::NewAccount),
    #[strum_discriminants(strum(
        message = "fund-later                   - Create an implicit-account"
    ))]
    /// Create an implicit-account
    FundLater(self::create_implicit_account::ImplicitAccount),
}

#[derive(Clone)]
pub struct CoverCostsCreateAccountContext(crate::GlobalContext);

impl CoverCostsCreateAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<CoverCostsCreateAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        match scope {
            CoverCostsCreateAccountDiscriminants::SponsorByFaucetService => {
                if previous_context.offline {
                    Err(color_eyre::Report::msg(
                        "Error: Creating an account with a faucet sponsor is not possible offline.",
                    ))
                } else {
                    Ok(Self(previous_context))
                }
            }
            _ => Ok(Self(previous_context)),
        }
    }
}

impl From<CoverCostsCreateAccountContext> for crate::GlobalContext {
    fn from(item: CoverCostsCreateAccountContext) -> Self {
        item.0
    }
}
