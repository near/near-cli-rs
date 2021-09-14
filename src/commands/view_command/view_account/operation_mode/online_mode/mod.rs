pub mod select_server;

/// аргументы, необходимые для создания транзакции в online mode
#[derive(Debug, Default, Clone, clap::Clap, near_cli_derive::Interactive)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliNetworkArgs {
    #[clap(subcommand)]
    selected_server: Option<self::select_server::CliSelectServer>,
}

#[derive(Debug, Clone)]
pub struct NetworkArgs {
    selected_server: self::select_server::SelectServer,
}

impl CliNetworkArgs {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.selected_server
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<NetworkArgs> for CliNetworkArgs {
    fn from(network_args: NetworkArgs) -> Self {
        Self {
            selected_server: Some(network_args.selected_server.into()),
        }
    }
}

impl From<CliNetworkArgs> for NetworkArgs {
    fn from(item: CliNetworkArgs) -> Self {
        let selected_server = match item.selected_server {
            Some(cli_selected_server) => {
                self::select_server::SelectServer::from(cli_selected_server)
            }
            None => self::select_server::SelectServer::choose_server(),
        };
        Self { selected_server }
    }
}

impl NetworkArgs {
    pub async fn process(self) -> crate::CliResult {
        self.selected_server.process().await
    }
}
