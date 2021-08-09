use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendTo {
    /// Specify an account
    Account(CliSender),
}

#[derive(Debug, Clone)]
pub enum SendTo {
    Account(Sender),
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
            SendTo::Account(sender) => Self::Account(sender.into()),
        }
    }
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Account(cli_sender) => {
                let sender = Sender::from(cli_sender);
                Self::Account(sender)
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
            SendTo::Account(sender) => sender.process(network_connection_config).await,
        }
    }
}

/// Specify the account to be view
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    selected_block_id: Option<super::block_id::CliBlockId>,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub sender_account_id: near_primitives::types::AccountId,
    selected_block_id: super::block_id::BlockId,
}

impl CliSender {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .selected_block_id
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(sender_account_id) = &self.sender_account_id {
            args.push_front(sender_account_id.to_string());
        };
        args
    }
}

impl From<Sender> for CliSender {
    fn from(sender: Sender) -> Self {
        Self {
            sender_account_id: Some(sender.sender_account_id),
            selected_block_id: Some(sender.selected_block_id.into()),
        }
    }
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: near_primitives::types::AccountId = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        let selected_block_id: super::block_id::BlockId = match item.selected_block_id {
            Some(cli_block_id) => cli_block_id.into(),
            None => super::block_id::BlockId::choose_block_id(),
        };
        Self {
            sender_account_id,
            selected_block_id,
        }
    }
}

impl Sender {
    pub fn input_sender_account_id() -> near_primitives::types::AccountId {
        println!();
        Input::new()
            .with_prompt("Enter your account ID to view your contract status")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(self.sender_account_id, network_connection_config)
            .await
    }
}
