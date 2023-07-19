use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod unstake;
mod unstake_all;
mod withdraw;
mod withdraw_all;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = DelegateStakeContext)]
pub struct DelegateStake {
    #[interactive_clap(skip_default_input_arg)]
    /// What is validator account ID?
    validator_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    delegate_stake_command: DelegateStakingCommand,
}

#[derive(Debug, Clone)]
pub struct DelegateStakeContext {
    global_context: crate::GlobalContext,
    validator_account_id: near_primitives::types::AccountId,
}

impl DelegateStakeContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<DelegateStake as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            validator_account_id: scope.validator_account_id.clone().into(),
        })
    }
}

impl DelegateStake {
    pub fn input_validator_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What is validator account ID?",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = DelegateStakeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Select actions with delegated staking:
pub enum DelegateStakingCommand {
    #[strum_discriminants(strum(message = "view       - "))]
    /// The transfer is carried out in NEAR tokens
    View,
    #[strum_discriminants(strum(message = "stake      - "))]
    /// The transfer is carried out in NEAR tokens
    Stake,
    #[strum_discriminants(strum(
        message = "unstake         - Unstaking the given amount from the inner account of the predecessor"
    ))]
    /// Unstaking the given amount from the inner account of the predecessor
    Unstake(self::unstake::Unstake),
    #[strum_discriminants(strum(
        message = "unstake-all     - Unstaking all staked balance from the inner account of the predecessor"
    ))]
    /// Unstaking all staked balance from the inner account of the predecessor
    UnstakeAll(self::unstake_all::UnstakeAll),
    #[strum_discriminants(strum(
        message = "withdraw        - Withdrawing the non staked balance for given account"
    ))]
    /// Withdrawing the non staked balance for given account
    Withdraw(self::withdraw::Withdraw),
    #[strum_discriminants(strum(
        message = "withdraw-all    - Withdrawing the entire unstaked balance from the predecessor account"
    ))]
    /// Withdrawing the entire unstaked balance from the predecessor account
    WithdrawAll(self::withdraw_all::WithdrawAll),
}
