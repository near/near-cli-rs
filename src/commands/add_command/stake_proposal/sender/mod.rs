use dialoguer::Input;

/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    transfer: Option<super::transfer_near_tokens_type::CliTransfer>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub transfer: super::transfer_near_tokens_type::Transfer,
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        let transfer: super::transfer_near_tokens_type::Transfer = match item.transfer {
            Some(cli_transfer) => cli_transfer.into(),
            None => super::transfer_near_tokens_type::Transfer::choose_transfer_near(),
        };
        Self {
            sender_account_id,
            transfer,
        }
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What is the account ID of the validator?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.sender_account_id.clone(),
            receiver_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.transfer
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
