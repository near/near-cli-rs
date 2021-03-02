use structopt::StructOpt;
use dialoguer::{
    Input,
};

use super::receiver::{
    Receiver,
    CliReceiver,
    ActionSubcommand,
};


#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub send_to: SendTo
}

#[derive(Debug)]
pub enum SendTo {
    Receiver(Receiver)
}

#[derive(Debug, StructOpt)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[structopt(subcommand)]
    send_to: Option<CliSendTo> 
}
#[derive(Debug, StructOpt)]
pub enum CliSendTo {
    Receiver(CliReceiver),
}

impl Sender {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.clone(),
            .. prepopulated_unsigned_transaction
        };
        self.send_to.process(unsigned_transaction, selected_server_url).await;
    }
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What is the account ID of the sender?")
            .interact_text()
            .unwrap()
    }
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id()
        };
        let send_to: SendTo = match item.send_to {
            Some(cli_send_to) => SendTo::from(cli_send_to),
            None => SendTo::send_to()
        }; 
        Sender {
            sender_account_id,
            send_to
        }
    }
}

impl SendTo {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: String,
    ) {
        match self {
            SendTo::Receiver(receiver) => receiver.process(prepopulated_unsigned_transaction, selected_server_url).await
        }
    }
    pub fn send_to() -> Self {
        let receiver_account_id: String = Receiver::input_receiver_account_id();
        let transaction_subcommand: ActionSubcommand = ActionSubcommand::choose_action_command();
        SendTo::Receiver(Receiver {
            receiver_account_id,
            transaction_subcommand
        })
    }
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Receiver(cli_receiver) => {
                let receiver = Receiver::from(cli_receiver);
                SendTo::Receiver(receiver)
            }
        }
    }
}
