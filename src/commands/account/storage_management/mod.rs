use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod storage_deposit;
mod storage_withdraw;
mod view_storage_balance;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ContractContext)]
pub struct Contract {
    /// Which contract account ID do you want to view the balance?
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    storage_actions: StorageActions,
}

#[derive(Debug, Clone)]
pub struct ContractContext {
    config: crate::config::Config,
    contract_account_id: near_primitives::types::AccountId,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.0,
            contract_account_id: scope.contract_account_id.clone().into(),
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = ContractContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What do you want to do with the storage?
pub enum StorageActions {
    #[strum_discriminants(strum(
        message = "view-balance    - View storage balance for an account"
    ))]
    /// View storage balance for an account
    ViewBalance(self::view_storage_balance::Account),
    #[strum_discriminants(strum(
        message = "deposit         - Make a storage deposit for the account"
    ))]
    /// Make a storage deposit for the account
    Deposit(self::storage_deposit::DepositArgs),
    #[strum_discriminants(strum(
        message = "withdraw        - Withdraw a deposit from storage for an account ID"
    ))]
    /// Withdraw a deposit from storage for an account ID
    Withdraw(self::storage_withdraw::WithdrawArgs),
}
