use dialoguer::Input;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod full_access_type;
mod function_call_type;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
pub struct AddAccessKeyAction {
    pub public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(subcommand)]
    pub permission: AccessKeyPermission,
}

impl AddAccessKeyAction {
    fn input_public_key(
        _context: &crate::common::SenderContext,
    ) -> color_eyre::eyre::Result<crate::types::public_key::PublicKey> {
        Ok(Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap())
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self.permission {
            AccessKeyPermission::GrantFullAccess(full_access_type) => {
                full_access_type
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        self.public_key.into(),
                    )
                    .await
            }
            AccessKeyPermission::GrantFunctionCallAccess(function_call_type) => {
                function_call_type
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        self.public_key.into(),
                    )
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SenderContext)]
///Select a permission that you want to add to the access key
pub enum AccessKeyPermission {
    #[strum_discriminants(strum(message = "A permission with function call"))]
    /// Предоставьте данные для ключа с function call
    GrantFunctionCallAccess(self::function_call_type::FunctionCallType),
    #[strum_discriminants(strum(message = "A permission with full access"))]
    /// Предоставьте данные для ключа с полным доступом
    GrantFullAccess(self::full_access_type::FullAccessType),
}
