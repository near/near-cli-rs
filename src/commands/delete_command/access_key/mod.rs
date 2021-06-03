use dialoguer::Input;

pub mod operation_mode;
mod sender;

#[derive(Debug, clap::Clap)]
pub enum CliDeleteAccessKeyAction {
    /// Specify public key
    PublicKey(CliDeleteAccessKeyType),
}

#[derive(Debug)]
pub enum DeleteAccessKeyAction {
    PublicKey(DeleteAccessKeyType),
}

impl From<CliDeleteAccessKeyAction> for DeleteAccessKeyAction {
    fn from(item: CliDeleteAccessKeyAction) -> Self {
        match item {
            CliDeleteAccessKeyAction::PublicKey(cli_delete_access_key_type) => {
                Self::PublicKey(cli_delete_access_key_type.into())
            }
        }
    }
}

impl DeleteAccessKeyAction {
    pub fn choose_delete_access_key_action() -> Self {
        Self::from(CliDeleteAccessKeyAction::PublicKey(Default::default()))
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
#[derive(Debug, Default, clap::Clap)]
pub struct CliDeleteAccessKeyType {
    public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct DeleteAccessKeyType {
    pub public_key: near_crypto::PublicKey,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliDeleteAccessKeyType> for DeleteAccessKeyType {
    fn from(item: CliDeleteAccessKeyType) -> Self {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => DeleteAccessKeyType::input_public_key(),
        };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self {
            public_key,
            sign_option,
        }
    }
}

impl DeleteAccessKeyType {
    pub fn input_public_key() -> near_crypto::PublicKey {
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
        self.sign_option
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
