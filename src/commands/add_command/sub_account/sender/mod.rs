use dialoguer::Input;

/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
pub struct CliSender {
    pub owner_account_id: Option<String>,
    #[clap(subcommand)]
    send_to: Option<super::receiver::CliSendTo>,
}

#[derive(Debug)]
pub struct Sender {
    pub owner_account_id: String,
    pub send_to: super::receiver::SendTo,
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let owner_account_id: String = match item.owner_account_id {
            Some(cli_owner_account_id) => cli_owner_account_id,
            None => Sender::input_owner_account_id(),
        };
        let send_to: super::receiver::SendTo = match item.send_to {
            Some(cli_send_to) => super::receiver::SendTo::from(cli_send_to),
            None => super::receiver::SendTo::send_to(),
        };
        Self {
            owner_account_id,
            send_to,
        }
    }
}

impl Sender {
    pub fn input_owner_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What is the owner account ID?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.owner_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
