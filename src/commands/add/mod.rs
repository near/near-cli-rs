use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod operation_mode;
mod public_key_mode;
mod sender;


/// добавление ключа пользователю
#[derive(Debug, Default, clap::Clap)]
pub struct CliAddAccessKey {
    #[clap(subcommand)]
    access_key: Option<CliAccessKey>,
}

#[derive(Debug)]
pub struct AddAccessKey {
    pub access_key: AccessKey,
}

impl From<CliAddAccessKey> for AddAccessKey {
    fn from(item: CliAddAccessKey) -> Self {
        let access_key: AccessKey = match item.access_key {
            Some(cli_access_key) => cli_access_key.into(),
            None => AccessKey::choose_access_key()
        };
        Self { access_key }
    }
}

impl AddAccessKey {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.access_key.process(prepopulated_unsigned_transaction).await
    } 
}

#[derive(Debug, clap::Clap)]
enum CliAccessKey {
    /// Add access key
    AccessKey(self::operation_mode::CliOperationMode),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum AccessKey {
    #[strum_discriminants(strum(message = "NEAR tokens"))]
    AccessKey(self::operation_mode::OperationMode),
}

impl From<CliAccessKey> for AccessKey {
    fn from(item: CliAccessKey) -> Self {
        match item {
            CliAccessKey::AccessKey(cli_operation_mode) => {
                AccessKey::AccessKey(cli_operation_mode.into())
            }
        }
    }
}

impl AccessKey {
    fn choose_access_key() -> Self {
        Self::from(CliAccessKey::AccessKey(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            AccessKey::AccessKey(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
