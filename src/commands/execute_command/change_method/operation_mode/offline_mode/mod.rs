/// аргументы, необходимые для offline mode
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOfflineArgs {
    #[clap(subcommand)]
    pub send_to: Option<super::super::contract::CliSendTo>,
}

#[derive(Debug, Clone)]
pub struct OfflineArgs {
    send_to: super::super::contract::SendTo,
}

impl CliOfflineArgs {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.send_to
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<OfflineArgs> for CliOfflineArgs {
    fn from(offline_args: OfflineArgs) -> Self {
        Self {
            send_to: Some(offline_args.send_to.into()),
        }
    }
}

impl OfflineArgs {
    pub fn from(item: CliOfflineArgs) -> color_eyre::eyre::Result<Self> {
        let send_to = match item.send_to {
            Some(cli_send_to) => super::super::contract::SendTo::from(cli_send_to, None)?,
            None => super::super::contract::SendTo::send_to(None)?,
        };
        Ok(Self { send_to })
    }
}

impl OfflineArgs {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let selected_server_url = None;
        self.send_to
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}
