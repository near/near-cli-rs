use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a receiver
    Contract(CliReceiver),
}

#[derive(Debug)]
pub enum SendTo {
    Contract(Receiver),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Contract(cli_receiver) => {
                let receiver = Receiver::from(cli_receiver);
                Self::Contract(receiver)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Contract(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendTo::Contract(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные о контракте
#[derive(Debug, Default, clap::Clap)]
pub struct CliReceiver {
    receiver_account_id: Option<String>,
    #[clap(subcommand)]
    call: Option<super::CliCallFunction>,
}

#[derive(Debug)]
pub struct Receiver {
    pub receiver_account_id: String,
    pub call: super::CallFunction,
}

impl From<CliReceiver> for Receiver {
    fn from(item: CliReceiver) -> Self {
        let receiver_account_id: String = match item.receiver_account_id {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Receiver::input_receiver_account_id(),
        };
        let call = match item.call {
            Some(cli_call) => cli_call.into(),
            None => super::CallFunction::choose_call_function(),
        };
        Self {
            receiver_account_id,
            call,
        }
    }
}

impl Receiver {
    pub fn input_receiver_account_id() -> String {
        Input::new()
            .with_prompt("What is the account ID of the contract?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.receiver_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.call
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
