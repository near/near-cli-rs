use dialoguer::Input;


/// Specify the account to be deleted
#[derive(Debug, Default, clap::Clap)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    public_key: Option<super::CliDeleteAccessKeyAction>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub public_key: super::DeleteAccessKeyAction,
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        let public_key = match item.public_key {
            Some(cli_delete_access_key) => super::DeleteAccessKeyAction::from(cli_delete_access_key),
            None => super::DeleteAccessKeyAction::choose_delete_access_key_action(),
        };
        Self {
            sender_account_id,
            public_key,
        }
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("Which account ID do you need to remove the key from?")
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
        self.public_key
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
