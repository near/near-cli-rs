use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod offline_mode;
mod online_mode;


/// инструмент выбора режима online/offline
#[derive(Debug, Default, clap::Clap)]
pub struct CliOperationMode {
    #[clap(subcommand)]
    mode: Option<CliMode>,
}

#[derive(Debug)]
pub struct OperationMode {
    pub mode: Mode,
}

impl From<CliOperationMode> for OperationMode {
    fn from(item: CliOperationMode) -> Self {
        let mode = match item.mode {
            Some(cli_mode) => Mode::from(cli_mode),
            None => Mode::choose_mode(),
        };
        Self { mode }
    }
}

impl OperationMode {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.mode.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliMode {
    Online(self::online_mode::CliOnlineArgs),
    Offline(self::offline_mode::CliOfflineArgs),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Mode {
    #[strum_discriminants(strum(message = "Yes, I keep it simple"))]
    Online(self::online_mode::OnlineArgs),
    #[strum_discriminants(strum(message = "No, I want to work in no-network (air-gapped) environment"))]
    Offline(self::offline_mode::OfflineArgs),
}

impl From<CliMode> for Mode {
    fn from(item: CliMode) -> Self {
        match item {
            CliMode::Online(cli_online_args) => {
                Self::Online(cli_online_args.into())
            }
            CliMode::Offline(cli_offline_args) => {
                Self::Offline(cli_offline_args.into())
            }
        }
    }
}

impl Mode {
    pub fn choose_mode() -> Self {
        println!();
        let variants = ModeDiscriminants::iter().collect::<Vec<_>>();
        let modes = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_mode = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "To construct a transaction you will need to provide information about sender (signer) and receiver accounts, and actions that needs to be performed.
                 \nDo you want to derive some information required for transaction construction automatically querying it online?"
            )
            .items(&modes)
            .default(0)
            .interact()
            .unwrap();
        let cli_mode = match variants[selected_mode] {
            ModeDiscriminants::Online => CliMode::Online(Default::default()),
            ModeDiscriminants::Offline => CliMode::Offline(Default::default()),
        };
        Self::from(cli_mode)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::Online(online_args) => {
                online_args.process(prepopulated_unsigned_transaction).await
            }
            Self::Offline(offline_args) => {
                offline_args.process(prepopulated_unsigned_transaction).await
            }
        }
    }
}
