/// данные для определения ключа с полным доступом
#[derive(Debug, Default, clap::Clap)]
pub struct CliFullAccessType {
    #[clap(subcommand)]
    sign_option: Option<crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction>,
}

#[derive(Debug)]
pub struct FullAccessType {
    pub sign_option: crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliFullAccessType> for FullAccessType {
    fn from(item: CliFullAccessType) -> Self {
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self { sign_option }
    }
}

impl FullAccessType {
    pub async fn process(
        self,
        nonce: near_primitives::types::Nonce,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
        public_key: near_crypto::PublicKey,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce,
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key,
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.sign_option
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
