use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a sub-account
    SubAccount(CliSubAccount),
}

#[derive(Debug)]
pub enum SendTo {
    SubAccount(SubAccount),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::SubAccount(cli_receiver) => {
                let receiver = SubAccount::from(cli_receiver);
                Self::SubAccount(receiver)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::SubAccount(Default::default()))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendTo::SubAccount(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// Specify a sub-account
#[derive(Debug, Default, clap::Clap)]
pub struct CliSubAccount {
    sub_account_id: Option<String>,
    #[clap(subcommand)]
    full_access_key: Option<super::full_access_key::CliFullAccessKey>,
}

#[derive(Debug)]
pub struct SubAccount {
    pub sub_account_id: String,
    pub full_access_key: super::full_access_key::FullAccessKey,
}

impl From<CliSubAccount> for SubAccount {
    fn from(item: CliSubAccount) -> Self {
        let sub_account_id: String = match item.sub_account_id {
            Some(cli_sub_account_id) => cli_sub_account_id,
            None => SubAccount::input_sub_account_id(),
        };
        let full_access_key = match item.full_access_key {
            Some(cli_full_access_key) => {
                super::full_access_key::FullAccessKey::from(cli_full_access_key)
            }
            None => super::full_access_key::FullAccessKey::choose_full_access_key(),
        };
        Self {
            sub_account_id,
            full_access_key,
        }
    }
}

impl SubAccount {
    pub fn input_sub_account_id() -> String {
        Input::new()
            .with_prompt("What is the sub-account ID?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::CreateAccount(
            near_primitives::transaction::CreateAccountAction {},
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.sub_account_id.clone(),
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.full_access_key
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
