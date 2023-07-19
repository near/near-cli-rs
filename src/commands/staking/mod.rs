use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod delegate;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Staking {
    #[interactive_clap(subcommand)]
    stake: StakingType,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Select the type of stake:
pub enum StakingType {
    #[strum_discriminants(strum(message = "delegate         - "))]
    ///
    Delegate(self::delegate::DelegateStake),
}
