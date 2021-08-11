use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendTo {
    /// Specify a receiver
    Receiver(CliReceiver),
}

#[derive(Debug, Clone)]
pub enum SendTo {
    Receiver(Receiver),
}

impl CliSendTo {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Receiver(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("receiver".to_owned());
                args
            }
        }
    }
}

impl From<SendTo> for CliSendTo {
    fn from(send_to: SendTo) -> Self {
        match send_to {
            SendTo::Receiver(receiver) => Self::Receiver(CliReceiver::from(receiver)),
        }
    }
}

impl SendTo {
    pub fn from(
        item: CliSendTo,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Receiver(cli_receiver) => {
                let receiver = Receiver::from(cli_receiver, connection_config, sender_account_id)?;
                Ok(Self::Receiver(receiver))
            }
        }
    }
}

impl SendTo {
    pub fn send_to(
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self::from(
            CliSendTo::Receiver(Default::default()),
            connection_config,
            sender_account_id,
        )?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendTo::Receiver(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные о получателе транзакции
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliReceiver {
    receiver_account_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    action: Option<super::transaction_actions::CliNextAction>,
}

#[derive(Debug, Clone)]
pub struct Receiver {
    pub receiver_account_id: near_primitives::types::AccountId,
    pub action: super::transaction_actions::NextAction,
}

impl CliReceiver {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(receiver_account_id) = &self.receiver_account_id {
            args.push_front(receiver_account_id.to_string());
        }
        args
    }
}

impl From<Receiver> for CliReceiver {
    fn from(receiver: Receiver) -> Self {
        Self {
            receiver_account_id: Some(receiver.receiver_account_id),
            action: Some(super::transaction_actions::CliNextAction::from(
                receiver.action,
            )),
        }
    }
}

impl Receiver {
    fn from(
        item: CliReceiver,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let receiver_account_id: near_primitives::types::AccountId = match item.receiver_account_id
        {
            Some(cli_receiver_account_id) => cli_receiver_account_id,
            None => Receiver::input_receiver_account_id(),
        };
        let action: super::transaction_actions::NextAction = match item.action {
            Some(cli_next_action) => super::transaction_actions::NextAction::from_cli_next_action(
                cli_next_action,
                connection_config,
                sender_account_id,
            )?,
            None => super::transaction_actions::NextAction::input_next_action(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            receiver_account_id,
            action,
        })
    }
}

impl Receiver {
    pub fn input_receiver_account_id() -> near_primitives::types::AccountId {
        Input::new()
            .with_prompt("What is the account ID of the receiver?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.receiver_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.action
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
