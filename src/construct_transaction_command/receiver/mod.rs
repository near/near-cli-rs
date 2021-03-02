use structopt::StructOpt;
use strum_macros::{
    EnumVariantNames,
};
use strum::VariantNames;
use dialoguer::{
    Select,
    Input,
    theme::ColorfulTheme,
    console::Term
};

use super::transaction_actions::transfer_near_tokens_type::{
    TransferNEARTokensAction,
    CliTransferNEARTokensAction,
    NearBalance
};
use super::sign_transaction::{
    SignTransaction,
    CliSignTransaction
};

use super::transaction_actions::create_account_type::{
    CreateAccountAction,
    CliCreateAccountAction
};
use super::transaction_actions::delete_access_key_type::{
    DeleteAccessKeyAction,
    CliDeleteAccessKeyAction
};
use super::transaction_actions::add_access_key_type::{
    AddAccessKeyAction,
    CliAddAccessKeyAction,
    AccessKeyPermission,
};
use super::transaction_actions::delete_account_type::{
    DeleteAccountAction,
    CliDeleteAccountAction
};
// use crate::command::on_off_line_mode::server::sender::receiver::add_access_key_type::full_access_type::FullAccessType;
// use crate::utils_subcommand::generate_keypair_subcommand;
// use add_access_key_type::full_access_type::FullAccessType;


#[derive(Debug)]
pub struct Receiver {
    pub receiver_account_id: String,
    pub transaction_subcommand: ActionSubcommand
}

#[derive(Debug, EnumVariantNames)]
pub enum ActionSubcommand {
    TransferNEARTokens(TransferNEARTokensAction),
    CallFunction,
    StakeNEARTokens,
    CreateAccount(CreateAccountAction),
    DeleteAccount(DeleteAccountAction),
    AddAccessKey(AddAccessKeyAction),
    DeleteAccessKey(DeleteAccessKeyAction),
    Skip(SkipAction)
}

#[derive(Debug, StructOpt)]
pub struct CliReceiver {
    receiver_account_id: Option<String>,
    #[structopt(subcommand)]
    transaction_subcommand: Option<CliActionSubcommand> 
}

#[derive(Debug, StructOpt)]
pub enum CliActionSubcommand {
    TransferNEARTokens(CliTransferNEARTokensAction),
    CallFunction,
    StakeNEARTokens,
    CreateAccount(CliCreateAccountAction),
    DeleteAccount(CliDeleteAccountAction),
    AddAccessKey(CliAddAccessKeyAction),
    DeleteAccessKey(CliDeleteAccessKeyAction),
    Skip(CliSkipAction)
}

#[derive(Debug, StructOpt)]
pub enum CliActionSkipSubcommand {
    Skip(CliSkipAction)
}

impl ActionSubcommand {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        match self {
            ActionSubcommand::TransferNEARTokens(args_transfer) => args_transfer.process(prepopulated_unsigned_transaction, selected_server_url).await,
            // ActionSubcommand::CallFunction(args_function) => {},
            // ActionSubcommand::StakeNEARTokens(args_stake) => {},
            ActionSubcommand::CreateAccount(args_create_account) => args_create_account.process(prepopulated_unsigned_transaction, selected_server_url).await,
            ActionSubcommand::DeleteAccount(args_delete_account) => args_delete_account.process(prepopulated_unsigned_transaction, selected_server_url).await,
            ActionSubcommand::AddAccessKey(args_add_access_key) => args_add_access_key.process(prepopulated_unsigned_transaction, selected_server_url, "".to_string()).await,
            ActionSubcommand::DeleteAccessKey(args_delete_access_key) => args_delete_access_key.process(prepopulated_unsigned_transaction, selected_server_url).await,
            ActionSubcommand::Skip(args_skip) => args_skip.process(prepopulated_unsigned_transaction, selected_server_url).await,
            _ => unreachable!("Error")
        }
    }
    pub fn choose_action_command() -> Self {
        println!();
        let action_subcommands= ActionSubcommand::VARIANTS;
        let select_action_subcommand = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action that you want to add to the action:")
            .items(&action_subcommands)
            .default(0)
            .interact_on_opt(&Term::stderr())
            .unwrap();
        match select_action_subcommand {
            Some(0) => {
                let amount: NearBalance = NearBalance::input_amount();
                let next_action: Box<ActionSubcommand> = Box::new(ActionSubcommand::choose_action_command());
                ActionSubcommand::TransferNEARTokens(TransferNEARTokensAction {
                    amount,
                    next_action
                })
            },
            Some(1) => ActionSubcommand::CallFunction,
            Some(2) => ActionSubcommand::StakeNEARTokens,
            Some(3) => {
                let next_action: Box<ActionSubcommand> = Box::new(ActionSubcommand::choose_action_command());
                ActionSubcommand::CreateAccount(CreateAccountAction {
                    next_action
                })
            },
            Some(4) => {
                let beneficiary_id: String = DeleteAccountAction::input_beneficiary_id();
                let next_action: Box<ActionSubcommand> = Box::new(ActionSubcommand::choose_action_command());
                ActionSubcommand::DeleteAccount(DeleteAccountAction {
                    beneficiary_id,
                    next_action
                })
            },
            Some(5) => {
                let public_key: String = AddAccessKeyAction::input_public_key();
                let nonce: near_primitives::types::Nonce = AddAccessKeyAction::input_nonce();
                let permission: AccessKeyPermission = AccessKeyPermission::choose_permission();
                ActionSubcommand::AddAccessKey(AddAccessKeyAction {
                    public_key,
                    nonce,
                    permission
                })
            },
            Some(6) => {
                let public_key: String = DeleteAccessKeyAction::input_public_key();
                let next_action: Box<ActionSubcommand> = Box::new(ActionSubcommand::choose_action_command());
                ActionSubcommand::DeleteAccessKey(DeleteAccessKeyAction {
                    public_key,
                    next_action
                })
            },
            Some(7) => ActionSubcommand::Skip(SkipAction{sign_option: SignTransaction::choose_sign_option()}),
            _ => unreachable!("Error")
        }
    }
}

impl Receiver {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        println!("Receiver process: self:\n       {:?}", &self);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.receiver_account_id.clone(),
            .. prepopulated_unsigned_transaction
        };
        self.transaction_subcommand.process(unsigned_transaction, selected_server_url).await;
    }
    pub fn input_receiver_account_id() -> String {
        Input::new()
            .with_prompt("What is the account ID of the receiver?")
            .interact_text()
            .unwrap()
    }
}

impl From<CliReceiver> for Receiver {
    fn from(item: CliReceiver) -> Self {
        let receiver_account_id: String = match item.receiver_account_id {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Receiver::input_receiver_account_id()
        };
        let transaction_subcommand: ActionSubcommand = match item.transaction_subcommand {
            Some(cli_action_subcommand) => ActionSubcommand::from(cli_action_subcommand),
            None => ActionSubcommand::choose_action_command()
        };
        Receiver {
            receiver_account_id,
            transaction_subcommand
        }
    }
}

impl From<CliActionSubcommand> for ActionSubcommand {
    fn from(item: CliActionSubcommand) -> Self {
        match item {
            CliActionSubcommand::TransferNEARTokens(cli_transfer_near_token) => {
                let transfer_near_token: TransferNEARTokensAction = TransferNEARTokensAction::from(cli_transfer_near_token);
                ActionSubcommand::TransferNEARTokens(transfer_near_token)
            },
            CliActionSubcommand::CreateAccount(cli_create_account) => {
                let create_account: CreateAccountAction = CreateAccountAction::from(cli_create_account);
                ActionSubcommand::CreateAccount(create_account)
            },
            CliActionSubcommand::DeleteAccount(cli_delete_account) => {
                let delete_account: DeleteAccountAction = DeleteAccountAction::from(cli_delete_account);
                ActionSubcommand::DeleteAccount(delete_account)
            },
            CliActionSubcommand::AddAccessKey(cli_add_access_key) => {
                let add_access_key: AddAccessKeyAction = AddAccessKeyAction::from(cli_add_access_key);
                ActionSubcommand::AddAccessKey(add_access_key)
            },
            CliActionSubcommand::DeleteAccessKey(cli_delete_access_key) => {
                let delete_access_key: DeleteAccessKeyAction = DeleteAccessKeyAction::from(cli_delete_access_key);
                ActionSubcommand::DeleteAccessKey(delete_access_key)
            },
            _ => unreachable!("Error")
        }
    }
}

impl From<CliActionSkipSubcommand> for ActionSubcommand {
    fn from(item: CliActionSkipSubcommand) -> Self {
        match item {
            CliActionSkipSubcommand::Skip(cli_skip_action) => {
                let skip_action: SkipAction = SkipAction::from(cli_skip_action);
                ActionSubcommand::Skip(skip_action)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use near_primitives::hash::CryptoHash;
    use std::str::FromStr;
    use super::*;

    #[actix_rt::test]
    async fn test_receiver_process() {
        let my_self = Receiver {
            receiver_account_id: "qwe.testnet".to_string(),
            transaction_subcommand: ActionSubcommand::CreateAccount(
                CreateAccountAction {
                    next_action: Box::new(ActionSubcommand::Skip(
                        SkipAction {
                            sign_option: SignTransaction::SignKeychain(super::super::sign_transaction::sign_keychain::SignKeychain{
                                key_chain: "qweqwe".to_string()
                            })
                        }
                    ))
                }
            )
            
        };
        let prepopulated_unsigned_transaction: near_primitives::transaction::Transaction = near_primitives::transaction::Transaction {
            signer_id: "volodymyr.testnet".to_string(),
            public_key: near_crypto::PublicKey::from_str("ed25519:7FmDRADa1v4BcLiiR9MPPdmWQp3Um1iPdAYATvBY1YzS").unwrap(),
            nonce: 55,
            receiver_id: "qwe.testnet".to_string(),
            block_hash: crate::common::BlobAsBase58String::<CryptoHash>::from_str("F2KwJ2rBE5LfuPFPRTYtu243hTniYggfC6P24WQVfZnx").unwrap().into_inner(),
            actions: vec![],
        };
        let selected_server_url: String = "https://rpc.testnet.near.org".to_string();
        Receiver::process(my_self, prepopulated_unsigned_transaction, selected_server_url).await;

    }
}

#[derive(Debug)]
pub struct SkipAction {
    pub sign_option: SignTransaction
}

#[derive(Debug, StructOpt)]
pub struct CliSkipAction {
    #[structopt(subcommand)]
    sign_option: Option<CliSignTransaction> 
}

impl SkipAction {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        println!("Skip process:\n       {:?}", &self);
        println!("Skip process: prepopulated_unsigned_transaction:\n       {:?}", &prepopulated_unsigned_transaction);
        self.sign_option.process(prepopulated_unsigned_transaction, selected_server_url).await;
    }
}

impl From<CliSkipAction> for SkipAction {
    fn from(item: CliSkipAction) -> Self {
        let sign_option: SignTransaction = match item.sign_option {
            Some(cli_sign_transaction) => SignTransaction::from(cli_sign_transaction),
            None => SignTransaction::choose_sign_option()
        };
        SkipAction {sign_option}
    }
}
