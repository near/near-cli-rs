pub mod select_server;


#[derive(Debug, Default, clap::Clap)]
pub struct CliOnlineArgs {
    #[clap(subcommand)]
    selected_server: Option<self::select_server::CliSelectServer>,
}

#[derive(Debug)]
pub struct OnlineArgs {
    selected_server: self::select_server::SelectServer,
}

impl From<CliOnlineArgs> for OnlineArgs {
    fn from(item: CliOnlineArgs) -> Self {
        let selected_server = match item.selected_server {
            Some(cli_selected_server) => self::select_server::SelectServer::from(cli_selected_server),
            None => self::select_server::SelectServer::choose_server(),
        };
        Self { selected_server }
    }
}

impl OnlineArgs {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.selected_server
            .process(prepopulated_unsigned_transaction)
            .await
    }
}
