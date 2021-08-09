use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendTo {
    /// Specify an account
    Account(CliAccount),
}

#[derive(Debug, Clone)]
pub enum SendTo {
    Account(Account),
}

impl CliSendTo {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Account(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("account".to_owned());
                args
            }
        }
    }
}

impl From<SendTo> for CliSendTo {
    fn from(send_to: SendTo) -> Self {
        match send_to {
            SendTo::Account(account) => Self::Account(account.into()),
        }
    }
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

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            SendTo::Account(account) => account.process(network_connection_config).await,
        }
    }
}

/// Specify account to view the nonce for public key
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliAccount {
    account_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    public_key: Option<super::public_key::CliAccessKey>,
}

#[derive(Debug, Clone)]
pub struct Account {
    account_id: near_primitives::types::AccountId,
    pub public_key: super::public_key::AccessKey,
}

impl CliAccount {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .public_key
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(account_id) = &self.account_id {
            args.push_front(account_id.to_string());
        };
        args
    }
}

impl From<Account> for CliAccount {
    fn from(account: Account) -> Self {
        Self {
            account_id: Some(account.account_id),
            public_key: Some(account.public_key.into()),
        }
    }
}

impl From<CliAccount> for Account {
    fn from(item: CliAccount) -> Self {
        let account_id: near_primitives::types::AccountId = match item.account_id {
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
    fn input_account_id() -> near_primitives::types::AccountId {
        println!();
        Input::new()
            .with_prompt("Enter your account ID")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.public_key
            .process(self.account_id, network_connection_config)
            .await
    }
}
