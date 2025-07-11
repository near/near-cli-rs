use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod sign_nep413;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct MessageCommand {
    #[interactive_clap(subcommand)]
    pub message_actions: MessageActions,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What do you want to do with a message?
pub enum MessageActions {
    #[strum_discriminants(strum(
        message = "sign-nep413         - Sign a NEP-413 message off-chain"
    ))]
    /// Sign a NEP-413 message off-chain
    SignNep413(self::sign_nep413::SignNep413),
}
