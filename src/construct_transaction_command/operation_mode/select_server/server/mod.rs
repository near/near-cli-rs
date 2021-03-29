use dialoguer::Input;
use structopt::StructOpt;

use crate::construct_transaction_command::sender::{CliSender, Sender};

#[derive(Debug)]
pub struct Server {
    pub url: Option<url::Url>,
    pub send_from: SendFrom,
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        println!("Server process:\n        {:?}", &self);
        let selected_server_url = self.url.clone();
        self.send_from
            .process(prepopulated_unsigned_transaction, selected_server_url)
            .await
    }
}

#[derive(Debug)]
pub enum SendFrom {
    Sender(Sender),
}

impl SendFrom {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        println!("Sendfrom process:\n      {:?}", &self);
        match self {
            SendFrom::Sender(sender) => {
                sender
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
    pub fn send_from() -> CliSendFrom {
        CliSendFrom::Sender(Default::default())
    }
}

#[derive(Debug, Default, StructOpt)]
pub struct CliServer {
    #[structopt(subcommand)]
    pub send_from: Option<CliSendFrom>,
}

#[derive(Debug, Default, StructOpt)]
pub struct CliCustomServer {
    #[structopt(long)]
    pub url: Option<crate::common::AvailableRpcServerUrl>,
    #[structopt(subcommand)]
    send_from: Option<CliSendFrom>,
}

#[derive(Debug, StructOpt)]
pub enum CliSendFrom {
    Sender(CliSender),
}

impl CliServer {
    pub fn into_server(self, url: String) -> Server {
        let cli_send_from: CliSendFrom = match self.send_from {
            Some(cli_send_from) => cli_send_from,
            None => SendFrom::send_from(),
        };
        Server {
            url: Some(url::Url::parse(&url).unwrap()),
            send_from: SendFrom::from(cli_send_from),
        }
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> Server {
        let url: crate::common::AvailableRpcServerUrl = match self.url {
            Some(url) => url,
            None => Input::new()
                .with_prompt("What is the RPC endpoi?")
                .interact_text()
                .unwrap(),
        };
        let cli_send_from: CliSendFrom = match self.send_from {
            Some(cli_send_from) => cli_send_from,
            None => SendFrom::send_from(),
        };
        Server {
            url: Some(url.inner),
            send_from: SendFrom::from(cli_send_from),
        }
    }
}

impl From<CliSendFrom> for SendFrom {
    fn from(item: CliSendFrom) -> Self {
        match item {
            CliSendFrom::Sender(cli_sender) => {
                let sender: Sender = Sender::from(cli_sender);
                SendFrom::Sender(sender)
            }
        }
    }
}
