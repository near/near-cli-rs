use dialoguer::Input;

/// Add full access key to the sub-account
#[derive(Debug, Default, clap::Clap)]
pub struct CliAddAccessKeyAction {
    public_key: Option<near_crypto::PublicKey>,
    nonce: Option<u64>,
    #[clap(subcommand)]
    deposit: Option<super::super::super::deposit::CliDeposit>,
}

#[derive(Debug)]
pub struct AddAccessKeyAction {
    pub public_key: near_crypto::PublicKey,
    pub nonce: near_primitives::types::Nonce,
    pub deposit: super::super::super::deposit::Deposit,
}

impl From<CliAddAccessKeyAction> for AddAccessKeyAction {
    fn from(item: CliAddAccessKeyAction) -> Self {
        let public_key: near_crypto::PublicKey = match item.public_key {
            Some(cli_public_key) => cli_public_key,
            None => AddAccessKeyAction::input_public_key(),
        };
        let deposit = match item.deposit {
            Some(cli_deposit) => super::super::super::deposit::Deposit::from(cli_deposit),
            None => super::super::super::deposit::Deposit::choose_deposit(),
        };
        Self {
            public_key,
            nonce: 0,
            deposit,
        }
    }
}

impl AddAccessKeyAction {
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this access key")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        let access_key: near_primitives::account::AccessKey = near_primitives::account::AccessKey {
            nonce: self.nonce.clone(),
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        };
        let action = near_primitives::transaction::Action::AddKey(
            near_primitives::transaction::AddKeyAction {
                public_key: self.public_key.clone(),
                access_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.deposit
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
