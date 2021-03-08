use structopt::StructOpt;
use dialoguer::{
    Input,
};

use crate::construct_transaction_command::sender::{
    CliSender,
    SendTo,
    Sender
};


#[derive(Debug)]
pub struct Server {
    pub url: Option<url::Url>,
    pub send_from: SendFrom
}

impl Server {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) {
        println!("Server process:\n        {:?}", &self);
        let selected_server_url = self.url.clone();
        self.send_from.process(prepopulated_unsigned_transaction, selected_server_url).await;
    }
}

#[derive(Debug)]
pub enum SendFrom {
    Sender(Sender)
}

impl SendFrom {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) {
        println!("Sendfrom process:\n      {:?}", &self);
        match self {
            SendFrom::Sender(sender) => sender.process(prepopulated_unsigned_transaction, selected_server_url).await,
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct CliServer {
    #[structopt(subcommand)]
    pub send_from: Option<CliSendFrom> 
}

#[derive(Debug, StructOpt)]
pub struct CliCustomServer {
    #[structopt(long)]
    pub url: Option<String>,
    #[structopt(subcommand)]
    send_from: Option<CliSendFrom> 
}

#[derive(Debug, StructOpt)]
pub enum CliSendFrom {
    Sender(CliSender)
}

impl CliServer {
    pub fn into_server(self, url: String) -> Server {
        let send_from: SendFrom = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from),
            None => SendFrom::send_from()
        };
        Server {
            url: Some(url::Url::parse(&url).unwrap()) ,
            send_from,
        }
    }
}

impl CliCustomServer {
    pub fn into_server(self) -> Server {
        let url: url::Url = match self.url {
            Some(url) => {
                match url::Url::parse(&url) {
                    Ok(url) => url,
                    Err(_) => {
                        Input::new()
                            .with_prompt("What is the RPC endpoi?")
                            .interact_text()
                            .unwrap()
                    }
                }
            },
            None => {
                Input::new()
                    .with_prompt("What is the RPC endpoi?")
                    .interact_text()
                    .unwrap()
            }
        };
        let send_from: SendFrom = match self.send_from {
            Some(cli_send_from) => SendFrom::from(cli_send_from),
            None => SendFrom::send_from()
        };
        Server {
            url: Some(url),
            send_from,
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

impl SendFrom {
    pub fn send_from() -> Self {
        let sender_account_id : String = Sender::input_sender_account_id();
        let send_to: SendTo = SendTo::send_to();
        SendFrom::Sender(Sender {
            sender_account_id,
            send_to
        })
    }
}
