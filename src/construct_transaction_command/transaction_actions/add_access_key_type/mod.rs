use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use structopt::StructOpt;
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

use super::super::receiver::NextAction;

pub(crate) mod function_call_type;
use function_call_type::{CliFunctionCallType, FunctionCallType};
pub(crate) mod full_access_type;
use full_access_type::{CliFullAccessType, FullAccessType};

#[derive(Debug)]
pub struct AddAccessKeyAction {
    pub public_key: near_crypto::PublicKey,
    pub nonce: near_primitives::types::Nonce,
    pub permission: AccessKeyPermission,
}

#[derive(Debug, StructOpt)]
pub struct CliAddAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
    #[structopt(long)]
    nonce: Option<u64>,
    #[structopt(subcommand)]
    permission: Option<CliAccessKeyPermission>,
}

#[derive(Debug, StructOpt)]
pub enum CliAccessKeyPermission {
    FunctionCallAction(CliFunctionCallType),
    FullAccessAction(CliFullAccessType),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(message = "A permission with function call"))]
    FunctionCallAction(FunctionCallType),
    #[strum_discriminants(strum(message = "A permission with full access"))]
    FullAccessAction(FullAccessType),
}

impl From<CliAddAccessKeyAction> for AddAccessKeyAction {
    fn from(item: CliAddAccessKeyAction) -> Self {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => AddAccessKeyAction::input_public_key(),
        };
        let nonce: near_primitives::types::Nonce = match item.nonce {
            Some(cli_nonce) => near_primitives::types::Nonce::from(cli_nonce),
            None => AddAccessKeyAction::input_nonce(),
        };
        let permission: AccessKeyPermission = match item.permission {
            Some(cli_permission) => AccessKeyPermission::from(cli_permission),
            None => AccessKeyPermission::choose_permission(),
        };
        AddAccessKeyAction {
            public_key,
            nonce,
            permission,
        }
    }
}

impl AddAccessKeyAction {
    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) {
        println!("AddAccessKeyAction process: self:\n       {:?}", &self);
        println!(
            "AddAccessKeyAction process: prepopulated_unsigned_transaction:\n       {:?}",
            &prepopulated_unsigned_transaction
        );
        match self.permission {
            AccessKeyPermission::FullAccessAction(full_access_type) => {
                full_access_type
                    .process(
                        self.nonce,
                        prepopulated_unsigned_transaction,
                        selected_server_url,
                        self.public_key,
                    )
                    .await
            }
            AccessKeyPermission::FunctionCallAction(function_call_type) => {
                function_call_type
                    .process(
                        self.nonce,
                        prepopulated_unsigned_transaction,
                        selected_server_url,
                        self.public_key,
                    )
                    .await
            }
        }
    }
    pub fn input_nonce() -> near_primitives::types::Nonce {
        Input::new()
            .with_prompt("Enter the nonce for this access key")
            .interact_text()
            .unwrap()
    }
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }
}

impl From<CliAccessKeyPermission> for AccessKeyPermission {
    fn from(item: CliAccessKeyPermission) -> Self {
        match item {
            CliAccessKeyPermission::FunctionCallAction(cli_function_call_type) => {
                let function_call_type: FunctionCallType =
                    FunctionCallType::from(cli_function_call_type);
                AccessKeyPermission::FunctionCallAction(function_call_type)
            }
            CliAccessKeyPermission::FullAccessAction(cli_full_access_type) => {
                let full_access_type: FullAccessType = FullAccessType::from(cli_full_access_type);
                AccessKeyPermission::FullAccessAction(full_access_type)
            }
        }
    }
}

impl AccessKeyPermission {
    pub fn choose_permission() -> Self {
        let variants = AccessKeyPermissionDiscriminants::iter().collect::<Vec<_>>();
        let permissions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let select_permission = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a permission that you want to add to the access key:")
            .items(&permissions)
            .default(0)
            .interact()
            .unwrap();
        match variants[select_permission] {
            AccessKeyPermissionDiscriminants::FunctionCallAction => {
                let allowance: Option<near_primitives::types::Balance> =
                    FunctionCallType::input_allowance();
                let receiver_id: near_primitives::types::AccountId =
                    FunctionCallType::input_receiver_id();
                let method_names: Vec<String> = FunctionCallType::input_method_names();
                let next_action: Box<NextAction> = Box::new(NextAction::input_next_action());
                AccessKeyPermission::FunctionCallAction(FunctionCallType {
                    allowance,
                    receiver_id,
                    method_names,
                    next_action,
                })
            }
            AccessKeyPermissionDiscriminants::FullAccessAction => {
                AccessKeyPermission::FullAccessAction(FullAccessType {
                    next_action: Box::new(NextAction::input_next_action()),
                })
            }
        }
    }
}
