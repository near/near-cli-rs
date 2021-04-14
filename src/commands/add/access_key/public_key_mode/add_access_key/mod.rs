use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod function_call_type;
mod full_access_type;


/// добавление ключа пользователю
#[derive(Debug, Default, clap::Clap)]
pub struct CliAddAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
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
        let permission: AccessKeyPermission = match item.permission {
            Some(cli_permission) => AccessKeyPermission::from(cli_permission),
            None => AccessKeyPermission::choose_permission(),
        };
        Self {
            public_key,
            nonce: 0,
            permission,
        }
    }
}

impl AddAccessKeyAction {
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self.permission {
            AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(
                        self.nonce,
                        prepopulated_unsigned_transaction,
                        selected_server_url,
                        self.public_key,
                    )
                    .await
            }
            AccessKeyPermission::GrantFunctionCallAccess(function_call_type) => {
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
    /// Предоставьте данные для ключа с function call
    GrantFunctionCallAccess(self::function_call_type::CliFunctionCallType),
    /// Предоставьте данные для ключа с полным доступом
    GrantFullAccess(self::full_access_type::CliFullAccessType),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(message = "A permission with function call"))]
    GrantFunctionCallAccess(self::function_call_type::FunctionCallType),
    #[strum_discriminants(strum(message = "A permission with full access"))]
    GrantFullAccess(self::full_access_type::FullAccessType),
}

impl From<CliAccessKeyPermission> for AccessKeyPermission {
    fn from(item: CliAccessKeyPermission) -> Self {
        match item {
            CliAccessKeyPermission::GrantFunctionCallAccess(cli_function_call_type) => {
                let function_call_type =
                    self::function_call_type::FunctionCallType::from(cli_function_call_type);
                AccessKeyPermission::GrantFunctionCallAccess(function_call_type)
            }
            CliAccessKeyPermission::GrantFullAccess(cli_full_access_type) => {
                let full_access_type = 
                    self::full_access_type::FullAccessType::from(cli_full_access_type);
                AccessKeyPermission::GrantFullAccess(full_access_type)
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
            AccessKeyPermissionDiscriminants::GrantFunctionCallAccess => Self::from(CliAccessKeyPermission::GrantFunctionCallAccess(Default::default())),
            AccessKeyPermissionDiscriminants::GrantFullAccess => Self::from(CliAccessKeyPermission::GrantFullAccess(Default::default()))
        }
    }
}
