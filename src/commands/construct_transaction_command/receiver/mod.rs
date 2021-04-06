use dialoguer::Input;


/// данные о получателе транзакции
#[derive(Debug, Default, clap::Clap)]
pub struct CliReceiver {
    receiver_account_id: Option<String>,
    #[clap(subcommand)]
    action: Option<super::transaction_actions::CliNextAction>,
}

#[derive(Debug)]
pub struct Receiver {
    pub receiver_account_id: String,
    pub action: super::transaction_actions::NextAction,
}

impl From<CliReceiver> for Receiver {
    fn from(item: CliReceiver) -> Self {
        let receiver_account_id: String = match item.receiver_account_id {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Receiver::input_receiver_account_id(),
        };
        let action: super::transaction_actions::NextAction = match item.action {
            Some(cli_next_action) => super::transaction_actions::NextAction::from(cli_next_action),
            None => super::transaction_actions::NextAction::input_next_action(),
        };
        Self {
            receiver_account_id,
            action,
        }
    }
}

impl Receiver {
    pub fn input_receiver_account_id() -> String {
        Input::new()
            .with_prompt("What is the account ID of the receiver?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.receiver_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.action
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
