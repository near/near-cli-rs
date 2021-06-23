use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify an account
    Account(CliSender),
}

#[derive(Debug)]
pub enum SendTo {
    Account(Sender),
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
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    selected_block_id: Option<super::block_id::CliBlockId>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    selected_block_id: super::block_id::BlockId,
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: String = match item.sender_account_id {
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
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What Account ID do you need to view?")
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
