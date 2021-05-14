use dialoguer::Input;

/// Specify the account to be deleted
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
            .with_prompt("Which account ID do you need to remove?")
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
            receiver_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a beneficiary
    Beneficiary(super::CliDeleteAccountAction),
}

#[derive(Debug)]
pub enum SendTo {
    Beneficiary(super::DeleteAccountAction),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Beneficiary(cli_delete_accaunt) => {
                let delete_accaunt = super::DeleteAccountAction::from(cli_delete_accaunt);
                Self::Beneficiary(delete_accaunt)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Beneficiary(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            SendTo::Beneficiary(delete_account_action) => {
                delete_account_action
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}
