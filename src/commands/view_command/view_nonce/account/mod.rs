use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify an account
    Account(CliAccount),
}

#[derive(Debug)]
pub enum SendTo {
    Account(Account),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Account(cli_account) => {
                let account = Account::from(cli_account);
                Self::Account(account)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Account(Default::default()))
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        match self {
            SendTo::Account(account) => account.process(selected_server_url).await,
        }
    }
}

/// Specify account to view the nonce for public key
#[derive(Debug, Default, clap::Clap)]
pub struct CliAccount {
    account_id: Option<String>,
    #[clap(subcommand)]
    public_key: Option<super::public_key::CliAccessKey>,
}

#[derive(Debug)]
pub struct Account {
    account_id: String,
    pub public_key: super::public_key::AccessKey,
}

impl From<CliAccount> for Account {
    fn from(item: CliAccount) -> Self {
        let account_id: String = match item.account_id {
            Some(cli_account_id) => cli_account_id,
            None => Account::input_account_id(),
        };
        let public_key = match item.public_key {
            Some(cli_public_key) => super::public_key::AccessKey::from(cli_public_key),
            None => super::public_key::AccessKey::choose_key(),
        };
        Self {
            account_id,
            public_key,
        }
    }
}

impl Account {
    fn input_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("Enter your account ID")
            .interact_text()
            .unwrap()
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        self.public_key
            .process(self.account_id, selected_server_url)
            .await
    }
}
