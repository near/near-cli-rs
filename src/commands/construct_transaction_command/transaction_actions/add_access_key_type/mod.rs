use async_recursion::async_recursion;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod function_call_type;
mod full_access_type;


#[derive(Debug, Default, clap::Clap)]
pub struct CliAddAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
    #[clap(long)]
    nonce: Option<u64>,
    #[clap(subcommand)]
    permission: Option<CliAccessKeyPermission>,
}

#[derive(Debug)]
pub struct AddAccessKeyAction {
    pub public_key: near_crypto::PublicKey,
    pub nonce: near_primitives::types::Nonce,
    pub permission: AccessKeyPermission,
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
        Self {
            public_key,
            nonce,
            permission,
        }
    }
}

impl AddAccessKeyAction {
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

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
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
}

#[derive(Debug, clap::Clap)]
pub enum CliAccessKeyPermission {
    FunctionCallAction(self::function_call_type::CliFunctionCallType),
    FullAccessAction(self::full_access_type::CliFullAccessType),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(message = "A permission with function call"))]
    FunctionCallAction(self::function_call_type::FunctionCallType),
    #[strum_discriminants(strum(message = "A permission with full access"))]
    FullAccessAction(self::full_access_type::FullAccessType),
}

impl From<CliAccessKeyPermission> for AccessKeyPermission {
    fn from(item: CliAccessKeyPermission) -> Self {
        match item {
            CliAccessKeyPermission::FunctionCallAction(cli_function_call_type) => {
                let function_call_type =
                    self::function_call_type::FunctionCallType::from(cli_function_call_type);
                AccessKeyPermission::FunctionCallAction(function_call_type)
            }
            CliAccessKeyPermission::FullAccessAction(cli_full_access_type) => {
                let full_access_type = 
                    self::full_access_type::FullAccessType::from(cli_full_access_type);
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
            AccessKeyPermissionDiscriminants::FunctionCallAction => Self::from(CliAccessKeyPermission::FunctionCallAction(Default::default())),
            AccessKeyPermissionDiscriminants::FullAccessAction => Self::from(CliAccessKeyPermission::FullAccessAction(Default::default()))
        }
    }
}
