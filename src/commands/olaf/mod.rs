use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod frost_aggregate;
mod frost_round1;
mod frost_round2;
mod simplpedpop_round1;
mod simplpedpop_round2;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct OlafCommands {
    #[interactive_clap(subcommand)]
    config_actions: OlafActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What do you want to do with a near CLI config?
pub enum OlafActions {
    #[strum_discriminants(strum(
        message = "simplpedpop-round1         - First round of the SimplPedPoP protocol"
    ))]
    /// First round of the SimplPedPoP protocol
    SimplpedpopRound1(self::simplpedpop_round1::SimplpedpopRound1),
    #[strum_discriminants(strum(
        message = "simplpedpop-round2         - Second round of the SimplPedPoP protocol"
    ))]
    /// Second round of the SimplPedPoP protocol
    SimplpedpopRound2(self::simplpedpop_round2::SimplpedpopRound2),
    #[strum_discriminants(strum(
        message = "frost-round1         - First round of the FROST protocol"
    ))]
    /// First round of the FROST protocol
    FrostRound1(self::frost_round1::FrostRound1),
    #[strum_discriminants(strum(
        message = "frost-round2         - Second round of the FROST protocol"
    ))]
    /// Second round of the FROST protocol
    FrostRound2(self::frost_round2::FrostRound2),
    #[strum_discriminants(strum(
        message = "frost-aggregate      - Aggregation round of the FROST protocol"
    ))]
    /// Aggregation round of the FROST protocol
    FrostAggregate(self::frost_aggregate::FrostAggregate),
}
