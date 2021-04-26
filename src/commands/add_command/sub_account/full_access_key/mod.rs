use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod public_key_mode;


#[derive(Debug, clap::Clap)]
pub enum CliFullAccessKey {
    /// Specify a full access key for the sub-account
    SubAccountFullAccess(CliSubAccountFullAccess),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum FullAccessKey {
    #[strum_discriminants(strum(message = "Add a full access key for the sub-account"))]
    SubAccountFullAccess(SubAccountFullAccess),
}

impl From<CliFullAccessKey> for FullAccessKey {
    fn from(item: CliFullAccessKey) -> Self {
        match item {
            CliFullAccessKey::SubAccountFullAccess(cli_sub_account_full_access) => {
                FullAccessKey::SubAccountFullAccess(cli_sub_account_full_access.into())
            }
        }
    }
}

impl FullAccessKey {
    pub fn choose_full_access_key() -> Self {
        println!();
        let variants = FullAccessKeyDiscriminants::iter().collect::<Vec<_>>();
        let actions = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "Сhoose what you want to add"
            )
            .items(&actions)
            .default(0)
            .interact()
            .unwrap();
        let cli_action = match variants[selected_action] {
            FullAccessKeyDiscriminants::SubAccountFullAccess => CliFullAccessKey::SubAccountFullAccess(Default::default()),
        };
        Self::from(cli_action)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            FullAccessKey::SubAccountFullAccess(sub_account_full_access) => {
                sub_account_full_access
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}

/// данные о ключе доступа
#[derive(Debug, Default, clap::Clap)]
pub struct CliSubAccountFullAccess {
    #[clap(subcommand)]
    public_key_mode: Option<self::public_key_mode::CliPublicKeyMode>,
}

#[derive(Debug)]
pub struct SubAccountFullAccess {
    pub public_key_mode: self::public_key_mode::PublicKeyMode,
}

impl From<CliSubAccountFullAccess> for SubAccountFullAccess {
    fn from(item: CliSubAccountFullAccess) -> Self {
        let public_key_mode = match item.public_key_mode {
            Some(cli_public_key_mode) => self::public_key_mode::PublicKeyMode::from(cli_public_key_mode),
            None => self::public_key_mode::PublicKeyMode::choose_public_key_mode(),
        };
        Self {
            public_key_mode,
        }
    }
}

impl SubAccountFullAccess {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        self.public_key_mode
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
