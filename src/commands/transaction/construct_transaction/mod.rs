use interactive_clap::FromCli;
use interactive_clap::ToCliArgs;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod add_access_key;
mod call_function;
mod create_subaccount;
mod delete_access_key;
mod delete_account;
mod deploy_contract;
mod stake_near_tokens;
mod transfer_tokens;

#[derive(Debug, Clone, clap::Parser)]
pub enum CliSkipNextAction {
    ///Go to transaction signing
    Skip(CliSkipAction),
}

impl CliSkipNextAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Skip(subcommand) => {
                // // let mut args = ;  it is not implemented now!!!
                // // args.push_front("skip".to_owned());
                // // args
                // subcommand.to_cli_args()
                let mut args = subcommand.to_cli_args();
                args.push_front("skip".to_owned());
                args
            }
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select an action that you want to add to the action:
pub enum NextAction {
    #[strum_discriminants(strum(message = "add-action   - Select a new action"))]
    /// Choose next action
    AddAction(SelectAction),
    #[strum_discriminants(strum(message = "skip         - Skip adding a new action"))]
    /// Go to transaction signing
    Skip(SkipAction),
}

impl From<NextAction> for CliSkipNextAction {
    fn from(next_action: NextAction) -> Self {
        match next_action {
            NextAction::AddAction(_select_action) => Self::Skip(CliSkipAction {
                network_config: None,
            }),
            NextAction::Skip(skip_action) => Self::Skip(skip_action.into()),
        }
    }
}

impl From<CliSkipNextAction> for CliNextAction {
    fn from(cli_skip_next_action: CliSkipNextAction) -> Self {
        match cli_skip_next_action {
            CliSkipNextAction::Skip(skip_action) => Self::Skip(skip_action),
        }
    }
}

impl NextAction {
    pub fn from_cli_skip_next_action(
        item: Option<CliSkipNextAction>,
        context: crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        match item {
            Some(CliSkipNextAction::Skip(cli_skip_action)) => {
                let optional_skip_action =
                    SkipAction::from_cli(Some(cli_skip_action), context.clone())?;
                if let Some(skip_action) = optional_skip_action {
                    Ok(Some(Self::Skip(skip_action)))
                } else {
                    Self::choose_variant(context)
                }
            }
            None => Self::choose_variant(context),
        }
    }
}
//-------------------------------------
/// This mode is not provided now
// impl CliNextAction {
//     fn from(item: CliSkipNextAction) -> color_eyre::eyre::Result<Self> {
//         match item {
//             CliSkipNextAction::Skip(cli_skip_action) => Ok(Self::Skip(cli_skip_action)),
//         }
//     }
// }
//--------------------------------------
impl NextAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            NextAction::AddAction(select_action) => {
                select_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            NextAction::Skip(skip_action) => {
                skip_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct BoxNextAction {
    inner: Box<NextAction>,
}

impl interactive_clap::ToCli for BoxNextAction {
    type CliVariant = CliSkipNextAction;
}

impl From<BoxNextAction> for CliSkipNextAction {
    fn from(box_next_action: BoxNextAction) -> Self {
        Self::from(*box_next_action.inner)
    }
}

impl BoxNextAction {
    fn choose_variant(context: crate::GlobalContext) -> color_eyre::eyre::Result<Option<Self>> {
        let optional_next_action = NextAction::choose_variant(context)?;
        if let Some(next_action) = optional_next_action {
            Ok(Some(Self {
                inner: Box::new(next_action),
            }))
        } else {
            Ok(None)
        }
    }
}

impl interactive_clap::FromCli for BoxNextAction {
    type FromCliContext = crate::GlobalContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<BoxNextAction as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let optional_next_action =
            NextAction::from_cli(optional_clap_variant.map(Into::into), context)?;
        if let Some(next_action) = optional_next_action {
            Ok(Some(Self {
                inner: Box::new(next_action),
            }))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SelectAction {
    #[interactive_clap(subcommand)]
    transaction_subcommand: ActionSubcommand,
}

impl SelectAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.transaction_subcommand
            .process(config, prepopulated_unsigned_transaction)
            .await
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select an action that you want to add to the action:
pub enum ActionSubcommand {
    #[strum_discriminants(strum(
        message = "send-near            - The transfer is carried out in NEAR tokens"
    ))]
    ///Specify data for transfer tokens
    SendNear(self::transfer_tokens::SendNearCommand),
    #[strum_discriminants(strum(
        message = "call-function        - Execute function (contract method)"
    ))]
    ///Specify data to call the function
    CallFunction(self::call_function::CallFunctionAction),
    #[strum_discriminants(strum(message = "stake-near-tokens    - Stake NEAR Tokens"))]
    ///Specify data to stake NEAR Tokens
    StakeNearTokens(self::stake_near_tokens::StakeNearTokensAction),
    #[strum_discriminants(strum(message = "create-subaccount    - Create a new sub-account"))]
    ///Specify data to create a sub-account
    CreateSubaccount(self::create_subaccount::CreateSubAccountAction),
    #[strum_discriminants(strum(message = "delete-account       - Delete an account"))]
    ///Specify data to delete an account
    DeleteAccount(self::delete_account::DeleteAccountAction),
    #[strum_discriminants(strum(
        message = "add-key              - Add an access key to an account"
    ))]
    ///Specify the data to add an access key to the account
    AddKey(self::add_access_key::AddKeyCommand),
    #[strum_discriminants(strum(
        message = "delete-key           - Delete an access key from an account"
    ))]
    ///Specify the data to delete the access key to the account
    DeleteKey(self::delete_access_key::DeleteKeyCommand),
    #[strum_discriminants(strum(message = "deploy               - Add a new contract code"))]
    ///Specify the details to deploy the contract code
    Deploy(self::deploy_contract::Contract),
}

impl ActionSubcommand {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            ActionSubcommand::SendNear(args_transfer) => {
                args_transfer
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::CallFunction(args_function) => {
                args_function
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::StakeNearTokens(args_stake) => {
                args_stake
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::CreateSubaccount(args_create_account) => {
                args_create_account
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::DeleteAccount(args_delete_account) => {
                args_delete_account
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::AddKey(args_add_key_command) => {
                args_add_key_command
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::DeleteKey(args_delete_access_key) => {
                args_delete_access_key
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            ActionSubcommand::Deploy(args_contract_file) => {
                args_contract_file
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SkipAction {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}
//------------------------------------
// impl From<SelectAction> for CliSkipAction {
//     fn from(select_action: SelectAction) -> Self {
//         Self{
//             sign_option:
//         }
//     }
// }
//-----------------------------------------

impl SkipAction {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => crate::common::print_transaction_status(
                transaction_info,
                self.network_config.get_network_config(config),
            ),
            None => Ok(()),
        }
    }
}
