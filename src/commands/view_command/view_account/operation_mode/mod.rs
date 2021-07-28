use strum::{EnumDiscriminants, EnumIter, EnumMessage};

pub mod online_mode;

/// инструмент выбора режима online/offline
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOperationMode {
    #[clap(subcommand)]
    mode: Option<CliMode>,
}

#[derive(Debug, Clone)]
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
    pub async fn process(self) -> crate::CliResult {
        self.mode.process().await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliMode {
    /// Execute a change method with online mode
    Network(self::online_mode::CliNetworkArgs),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Mode {
    #[strum_discriminants(strum(message = "Yes, I keep it simple"))]
    Network(self::online_mode::NetworkArgs),
}

impl From<CliMode> for Mode {
    fn from(item: CliMode) -> Self {
        match item {
            CliMode::Network(cli_network_args) => Self::Network(cli_network_args.into()),
        }
    }
}

impl Mode {
    pub fn choose_mode() -> Self {
        Self::from(CliMode::Network(Default::default()))
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            Self::Network(network_args) => network_args.process().await,
        }
    }
}
