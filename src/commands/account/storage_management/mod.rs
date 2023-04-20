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

#[derive(Clone)]
pub struct ContractContext {
    pub config: crate::config::Config,
    pub get_contract_account_id: GetContractAccountID,
}

impl ContractContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<Contract as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let contract_account_id = scope.contract_account_id.clone();
        let get_contract_account_id: GetContractAccountID =
            std::sync::Arc::new(move |_network_config| Ok(contract_account_id.clone().into()));
        Ok(Self {
            config: previous_context.0,
            get_contract_account_id,
        })
    }
}

pub type GetContractAccountId = std::sync::Arc<
    dyn Fn(
        &crate::config::NetworkConfig,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId>,
>;

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
