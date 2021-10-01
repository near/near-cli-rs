use dialoguer::Input;

/// Specify the account to be deleted
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    send_to: Option<CliSendTo>,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub sender_account_id: near_primitives::types::AccountId,
    pub send_to: SendTo,
}

impl CliSender {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .send_to
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(sender_account_id) = &self.sender_account_id {
            args.push_front(sender_account_id.to_string());
        }
        args
    }
}

impl From<Sender> for CliSender {
    fn from(sender: Sender) -> Self {
        Self {
            sender_account_id: Some(sender.sender_account_id),
            send_to: Some(sender.send_to.into()),
        }
    }
}
impl Sender {
    pub fn from(
        item: CliSender,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let sender_account_id: near_primitives::types::AccountId = match item.sender_account_id {
            Some(cli_sender_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    network_connection_config,
                    cli_sender_account_id.clone(),
                )? {
                    Some(_) => cli_sender_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", cli_sender_account_id);
                        Sender::input_sender_account_id(connection_config.clone())?
                    }
                },
                None => cli_sender_account_id,
            },
            None => Sender::input_sender_account_id(connection_config.clone())?,
        };
        let send_to: SendTo = match item.send_to {
            Some(cli_send_to) => {
                SendTo::from(cli_send_to, connection_config, sender_account_id.clone())?
            }
            None => SendTo::send_to(connection_config, sender_account_id.clone())?,
        };
        Ok(Self {
            sender_account_id,
            send_to,
        })
    }
}

impl Sender {
    fn input_sender_account_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        loop {
            let account_id: near_primitives::types::AccountId = Input::new()
                .with_prompt("Which account ID do you need to remove?")
                .interact_text()
                .unwrap();
            if let Some(connection_config) = &connection_config {
                if let Some(_) =
                    crate::common::get_account_state(connection_config, account_id.clone())?
                {
                    break Ok(account_id);
                } else {
                    println!("Account <{}> doesn't exist", account_id.to_string());
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
            signer_id: self.sender_account_id.clone(),
            receiver_id: self.sender_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendTo {
    /// Specify a beneficiary
    Beneficiary(super::CliDeleteAccountAction),
}

#[derive(Debug, Clone)]
pub enum SendTo {
    Beneficiary(super::DeleteAccountAction),
}

impl CliSendTo {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Beneficiary(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("beneficiary".to_owned());
                args
            }
        }
    }
}

impl From<SendTo> for CliSendTo {
    fn from(send_to: SendTo) -> Self {
        match send_to {
            SendTo::Beneficiary(delete_account_action) => {
                Self::Beneficiary(delete_account_action.into())
            }
        }
    }
}

impl SendTo {
    fn from(
        item: CliSendTo,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Beneficiary(cli_delete_accaunt) => {
                let delete_accaunt = super::DeleteAccountAction::from(
                    cli_delete_accaunt,
                    connection_config,
                    sender_account_id,
                )?;
                Ok(Self::Beneficiary(delete_accaunt))
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
            CliSendTo::Beneficiary(Default::default()),
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
            SendTo::Beneficiary(delete_account_action) => {
                delete_account_action
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
