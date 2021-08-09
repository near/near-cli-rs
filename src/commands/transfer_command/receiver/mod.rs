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
    transfer: Option<super::transfer_near_tokens_type::CliTransfer>,
}

#[derive(Debug, Clone)]
pub struct Receiver {
    pub receiver_account_id: near_primitives::types::AccountId,
    pub transfer: super::transfer_near_tokens_type::Transfer,
}

impl CliReceiver {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .transfer
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
            transfer: Some(super::transfer_near_tokens_type::CliTransfer::from(
                receiver.transfer,
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
        let receiver_account_id: near_primitives::types::AccountId = match item.receiver_account_id {
            Some(cli_receiver_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::check_account_id(
                    network_connection_config.clone(),
                    cli_receiver_account_id.clone(),
                )? {
                    Some(_) => cli_receiver_account_id,
                    None => {
                        if !crate::common::is_64_len_hex(&cli_receiver_account_id) {
                            println!("Account <{}> doesn't exist", cli_receiver_account_id);
                            Receiver::input_receiver_account_id(connection_config.clone())?
                        } else {
                            cli_receiver_account_id
                        }
                    }
                },
                None => cli_receiver_account_id,
            },
            None => Receiver::input_receiver_account_id(connection_config.clone())?,
        };
        let transfer: super::transfer_near_tokens_type::Transfer = match item.transfer {
            Some(cli_transfer) => super::transfer_near_tokens_type::Transfer::from(
                cli_transfer,
                connection_config,
                sender_account_id,
            )?,
            None => super::transfer_near_tokens_type::Transfer::choose_transfer_near(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(Self {
            receiver_account_id,
            transfer,
        })
    }
}

impl Receiver {
    fn input_receiver_account_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        loop {
            let account_id: near_primitives::types::AccountId = Input::new()
                .with_prompt("What is the account ID of the receiver?")
                .interact_text()
                .unwrap();
            if let Some(connection_config) = &connection_config {
                if let Some(_) =
                    crate::common::check_account_id(connection_config.clone(), account_id.clone())?
                {
                    break Ok(account_id);
                } else {
                    if !crate::common::is_64_len_hex(&account_id) {
                        println!("Account <{}> doesn't exist", account_id.to_string());
                    } else {
                        break Ok(account_id);
                    }
                }
            } else {
                break Ok(account_id);
            }
        }
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
        self.transfer
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
