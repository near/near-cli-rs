use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod deposit_and_stake;
mod stake;
mod stake_all;
mod unstake;
mod unstake_all;
pub mod view_balance;
mod withdraw;
mod withdraw_all;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = StakeDelegationContext)]
pub struct StakeDelegation {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the account that you want to manage delegated stake for:
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    delegate_stake_command: StakeDelegationCommand,
}

#[derive(Debug, Clone)]
pub struct StakeDelegationContext {
    global_context: crate::GlobalContext,
    account_id: near_primitives::types::AccountId,
}

impl StakeDelegationContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<StakeDelegation as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context,
            account_id: scope.account_id.clone().into(),
        })
    }
}

impl StakeDelegation {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Enter the account that you want to manage delegated stake for:",
        )
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StakeDelegationContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Select actions with delegated staking:
pub enum StakeDelegationCommand {
    #[strum_discriminants(strum(
        message = "view-balance         - View the delegated stake balance for a given account"
    ))]
    /// View the delegated stake balance for a given account
    ViewBalance(self::view_balance::ViewBalance),
    #[strum_discriminants(strum(
        message = "deposit-and-stake    - Delegate NEAR tokens to a validator's staking pool"
    ))]
    /// Delegate NEAR tokens to a validator's staking pool
    DepositAndStake(self::deposit_and_stake::DepositAndStake),
    #[strum_discriminants(strum(
        message = "stake                - Delegate a certain amount of previously deposited or unstaked NEAR tokens to a validator's staking pool"
    ))]
    /// Delegate a certain amount of previously deposited or unstaked NEAR tokens to a validator's staking pool
    Stake(self::stake::Stake),
    #[strum_discriminants(strum(
        message = "stake-all            - Delegate all previously deposited or unstaked NEAR tokens to a validator's staking pool"
    ))]
    /// Delegate all previously deposited or unstaked NEAR tokens to a validator's staking pool
    StakeAll(self::stake_all::StakeAll),
    #[strum_discriminants(strum(
        message = "unstake              - Unstake a certain amount of delegated NEAR tokens from a avalidator's staking pool"
    ))]
    /// Unstake a certain amount of delegated NEAR tokens from a avalidator's staking pool
    Unstake(self::unstake::Unstake),
    #[strum_discriminants(strum(
        message = "unstake-all          - Unstake all delegated NEAR tokens from a avalidator's staking pool"
    ))]
    /// Unstake all delegated NEAR tokens from a avalidator's staking pool
    UnstakeAll(self::unstake_all::UnstakeAll),
    #[strum_discriminants(strum(
        message = "withdraw             - Withdraw a certain amount of unstaked NEAR tokens from a avalidator's staking pool"
    ))]
    /// Withdraw a certain amount of unstaked NEAR tokens from a avalidator's staking pool
    Withdraw(self::withdraw::Withdraw),
    #[strum_discriminants(strum(
        message = "withdraw-all         - Withdraw all unstaked NEAR tokens from a avalidator's staking pool"
    ))]
    /// Withdraw all unstaked NEAR tokens from a avalidator's staking pool
    WithdrawAll(self::withdraw_all::WithdrawAll),
}
