use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod add_key;
pub mod call_function;
pub mod create_account;
pub mod delete_account;
pub mod delete_key;
pub mod deploy_contract;
pub mod deploy_global_contract;
pub mod stake;
pub mod transfer;
pub mod use_global_contract;
#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::ConstructTransactionContext)]
pub struct AddAction {
    #[interactive_clap(subcommand)]
    pub action: ActionSubcommand,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::super::ConstructTransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action that you want to add to the action:
pub enum ActionSubcommand {
    #[strum_discriminants(strum(
        message = "transfer               - The transfer is carried out in NEAR tokens"
    ))]
    /// Specify data for transfer tokens
    Transfer(self::transfer::TransferAction),
    #[strum_discriminants(strum(
        message = "function-call          - Execute function (contract method)"
    ))]
    /// Specify data to call the function
    FunctionCall(self::call_function::FunctionCallAction),
    #[strum_discriminants(strum(message = "stake                  - Stake NEAR Tokens"))]
    /// Specify data to stake NEAR Tokens
    Stake(self::stake::StakeAction),
    #[strum_discriminants(strum(message = "create-account         - Create a new sub-account"))]
    /// Specify data to create a sub-account
    CreateAccount(self::create_account::CreateAccountAction),
    #[strum_discriminants(strum(message = "delete-account         - Delete an account"))]
    /// Specify data to delete an account
    DeleteAccount(self::delete_account::DeleteAccountAction),
    #[strum_discriminants(strum(
        message = "add-key                - Add an access key to an account"
    ))]
    /// Specify the data to add an access key to the account
    AddKey(self::add_key::AddKeyAction),
    #[strum_discriminants(strum(
        message = "delete-key             - Delete an access key from an account"
    ))]
    /// Specify the data to delete the access key to the account
    DeleteKey(self::delete_key::DeleteKeyAction),
    #[strum_discriminants(strum(message = "deploy                 - Add a new contract code"))]
    /// Specify the details to deploy the contract code
    DeployContract(self::deploy_contract::DeployContractAction),
    #[strum_discriminants(strum(
        message = "deploy-global-contract - Add a new global contract code"
    ))]
    /// Specify the details to deploy the global contract code
    DeployGlobalContract(self::deploy_global_contract::DeployGlobalContractAction),
    #[strum_discriminants(strum(
        message = "use-global-contract    - Use a global contract to re-use the pre-deployed on-chain code"
    ))]
    /// Specify the details to use the global contract
    UseGlobalContract(self::use_global_contract::UseGlobalContractAction),
}
