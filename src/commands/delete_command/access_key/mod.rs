use dialoguer::Input;

pub mod operation_mode;
mod sender;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliDeleteAccessKeyAction {
    /// Specify public key
    PublicKey(CliDeleteAccessKeyType),
}

#[derive(Debug, Clone)]
pub enum DeleteAccessKeyAction {
    PublicKey(DeleteAccessKeyType),
}

impl CliDeleteAccessKeyAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::PublicKey(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("public-key".to_owned());
                args
            }
        }
    }
}

impl From<DeleteAccessKeyAction> for CliDeleteAccessKeyAction {
    fn from(delete_access_key_action: DeleteAccessKeyAction) -> Self {
        match delete_access_key_action {
            DeleteAccessKeyAction::PublicKey(delete_access_key_type) => {
                Self::PublicKey(delete_access_key_type.into())
            }
        }
    }
}

impl DeleteAccessKeyAction {
    pub fn from(
        item: CliDeleteAccessKeyAction,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliDeleteAccessKeyAction::PublicKey(cli_delete_access_key_type) => {
                Ok(Self::PublicKey(DeleteAccessKeyType::from(
                    cli_delete_access_key_type,
                    connection_config,
                    sender_account_id,
                )?))
            }
        }
    }
}

impl DeleteAccessKeyAction {
    pub fn choose_delete_access_key_action(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliDeleteAccessKeyAction::PublicKey(Default::default()),
            connection_config,
            sender_account_id,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            DeleteAccessKeyAction::PublicKey(delete_access_key_type) => {
                delete_access_key_type
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// Specify the access key to be deleted
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliDeleteAccessKeyType {
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug, Clone)]
pub struct DeleteAccessKeyType {
    pub public_key: near_crypto::PublicKey,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl CliDeleteAccessKeyType {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .sign_option
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(public_key) = &self.public_key {
            args.push_front(public_key.to_string());
        }
        args
    }
}

impl From<DeleteAccessKeyType> for CliDeleteAccessKeyType {
    fn from(delete_access_key_type: DeleteAccessKeyType) -> Self {
        Self {
            public_key: Some(delete_access_key_type.public_key),
            sign_option: Some(delete_access_key_type.sign_option.into()),
        }
    }
}

impl DeleteAccessKeyType {
    fn from(
        item: CliDeleteAccessKeyType,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => DeleteAccessKeyType::input_public_key(),
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::from(cli_sign_transaction, connection_config, sender_account_id)?,
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(connection_config, sender_account_id)?,
        };
        Ok(Self {
            public_key,
            sign_option,
        })
    }
}

impl DeleteAccessKeyType {
    fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::DeleteKey(
            near_primitives::transaction::DeleteKeyAction {
                public_key: self.public_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config.clone())
            .await?
        {
            Some(transaction_info) => {
                crate::common::print_transaction_status(
                    transaction_info,
                    network_connection_config,
                );
            }
            None => {}
        };
        Ok(())
    }
}
