use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod deposit;
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
        if !crate::common::is_used_delegated_validator_list_exist(
            &previous_context.config.credentials_home_dir,
        ) {
            crate::common::create_used_delegated_validator_list(&previous_context.config)?;
        }
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
        message = "view-balance         - View the total balance for a given account"
    ))]
    /// View the total balance for a given account
    ViewBalance(self::view_balance::ViewBalance),
    #[strum_discriminants(strum(
        message = "deposit              - Deposits the attached amount into the inner account of the predecessor"
    ))]
    /// Deposits the attached amount into the inner account of the predecessor
    Deposit(self::deposit::Deposit),
    #[strum_discriminants(strum(
        message = "deposit-and-stake    - Deposits the attached amount into the inner account of the predecessor and stakes it"
    ))]
    /// Deposits the attached amount into the inner account of the predecessor and stakes it
    DepositAndStake(self::deposit_and_stake::DepositAndStake),
    #[strum_discriminants(strum(
        message = "stake                - Staking the given amount from the inner account of the predecessor"
    ))]
    /// Staking the given amount from the inner account of the predecessor
    Stake(self::stake::Stake),
    #[strum_discriminants(strum(
        message = "stake-all            - Staking all available unstaked balance from the inner account of the predecessor"
    ))]
    /// Staking all available unstaked balance from the inner account of the predecessor
    StakeAll(self::stake_all::StakeAll),
    #[strum_discriminants(strum(
        message = "unstake              - Unstaking the given amount from the inner account of the predecessor"
    ))]
    /// Unstaking the given amount from the inner account of the predecessor
    Unstake(self::unstake::Unstake),
    #[strum_discriminants(strum(
        message = "unstake-all          - Unstaking all staked balance from the inner account of the predecessor"
    ))]
    /// Unstaking all staked balance from the inner account of the predecessor
    UnstakeAll(self::unstake_all::UnstakeAll),
    #[strum_discriminants(strum(
        message = "withdraw             - Withdrawing the non staked balance for given account"
    ))]
    /// Withdrawing the non staked balance for given account
    Withdraw(self::withdraw::Withdraw),
    #[strum_discriminants(strum(
        message = "withdraw-all         - Withdrawing the entire unstaked balance from the predecessor account"
    ))]
    /// Withdrawing the entire unstaked balance from the predecessor account
    WithdrawAll(self::withdraw_all::WithdrawAll),
}