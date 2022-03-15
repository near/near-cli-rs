mod server;

#[derive(clap::Parser)]
pub struct Online {
    #[clap(subcommand)]
    server: server::SelectServer,
}

impl Online {
    pub async fn process(self) {
        self.server.process().await;
    }
}
