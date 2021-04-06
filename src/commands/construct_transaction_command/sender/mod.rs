use dialoguer::Input;


/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    send_to: Option<CliSendTo>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub send_to: SendTo,
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        let send_to: SendTo = match item.send_to {
            Some(cli_send_to) => SendTo::from(cli_send_to),
            None => SendTo::send_to(),
        };
        Self {
            sender_account_id,
            send_to,
        }
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What is the account ID of the sender?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a receiver
    Receiver(super::receiver::CliReceiver),
}

#[derive(Debug)]
pub enum SendTo {
    Receiver(super::receiver::Receiver),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Receiver(cli_receiver) => {
                let receiver = super::receiver::Receiver::from(cli_receiver);
                Self::Receiver(receiver)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Receiver(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            SendTo::Receiver(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}
