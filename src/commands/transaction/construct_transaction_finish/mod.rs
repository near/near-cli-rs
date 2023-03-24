use interactive_clap::FromCli;

use interactive_clap::ToCliArgs;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_access_key;
mod call_function;
mod create_subaccount;
mod delete_access_key;
mod delete_account;
// mod deploy_contract;
mod stake_near_tokens;
mod transfer_tokens;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionActionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action that you want to add to the action:
pub enum NextAction {
    #[strum_discriminants(strum(message = "add-action   - Select a new action"))]
    /// Choose next action
    AddAction(SelectAction),
    #[strum_discriminants(strum(message = "skip         - Skip adding a new action"))]
    /// Go to transaction signing
    Skip(SkipAction),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionActionContext)]
pub struct SelectAction {
    #[interactive_clap(subcommand)]
    transaction_subcommand: ActionSubcommand,
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionActionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// Select an action that you want to add to the action:
pub enum ActionSubcommand {
    #[strum_discriminants(strum(
        message = "send-near            - The transfer is carried out in NEAR tokens"
    ))]
    /// Specify data for transfer tokens
    Transfer(self::transfer_tokens::TransferAction),
    #[strum_discriminants(strum(
        message = "call-function        - Execute function (contract method)"
    ))]
    /// Specify data to call the function
    FunctionCall(self::call_function::FunctionCallAction),
    #[strum_discriminants(strum(message = "stake-near-tokens    - Stake NEAR Tokens"))]
    /// Specify data to stake NEAR Tokens
    Stake(self::stake_near_tokens::StakeAction),
    #[strum_discriminants(strum(message = "create-subaccount    - Create a new sub-account"))]
    /// Specify data to create a sub-account
    CreateAccount(self::create_subaccount::CreateAccountAction),
    #[strum_discriminants(strum(message = "delete-account       - Delete an account"))]
    /// Specify data to delete an account
    DeleteAccount(self::delete_account::DeleteAccountAction),
    #[strum_discriminants(strum(
        message = "add-key              - Add an access key to an account"
    ))]
    /// Specify the data to add an access key to the account
    AddKey(self::add_access_key::AddKeyAction),
    #[strum_discriminants(strum(
        message = "delete-key           - Delete an access key from an account"
    ))]
    ///Specify the data to delete the access key to the account
    DeleteKey(self::delete_access_key::DeleteKeyAction),
    // #[strum_discriminants(strum(message = "deploy               - Add a new contract code"))]
    // ///Specify the details to deploy the contract code
    // Deploy(self::deploy_contract::Contract),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ConstructTransactionActionContext)]
#[interactive_clap(output_context = SkipActionContext)]
pub struct SkipAction {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct SkipActionContext(super::ConstructTransactionActionContext);

impl SkipActionContext {
    pub fn from_previous_context(
        previous_context: super::ConstructTransactionActionContext,
        _scope: &<SkipAction as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(previous_context))
    }
}

impl From<SkipActionContext> for crate::commands::ActionContext {
    fn from(item: SkipActionContext) -> Self {
        Self {
            config: item.0.config,
            signer_account_id: item.0.signer_account_id,
            receiver_account_id: item.0.receiver_account_id,
            actions: item.0.actions,
            on_after_getting_network_callback: std::sync::Arc::new(|_actions, network_config| {
                Ok(())
            }),
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
