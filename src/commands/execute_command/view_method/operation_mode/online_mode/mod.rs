pub mod select_server;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = ())]
pub struct NetworkArgs {
    #[interactive_clap(subcommand)]
    selected_server: self::select_server::SelectServer,
}

impl NetworkArgs {
    pub async fn process(self) -> crate::CliResult {
        self.selected_server.process().await
    }
}
