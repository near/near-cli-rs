use dialoguer::Input;

/// данные об отправителе транзакции
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub owner_account_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    send_to: Option<super::receiver::CliSendTo>,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub owner_account_id: near_primitives::types::AccountId,
    pub send_to: super::receiver::SendTo,
}

impl CliSender {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .send_to
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(owner_account_id) = &self.owner_account_id {
            args.push_front(owner_account_id.to_string());
        }
        args
    }
}

impl From<Sender> for CliSender {
    fn from(sender: Sender) -> Self {
        Self {
            owner_account_id: Some(sender.owner_account_id),
            send_to: Some(sender.send_to.into()),
        }
    }
}

impl Sender {
    pub fn from(
        item: CliSender,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let owner_account_id: near_primitives::types::AccountId = match item.owner_account_id {
            Some(cli_owner_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    network_connection_config,
                    cli_owner_account_id.clone(),
                )? {
                    Some(_) => cli_owner_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", cli_owner_account_id);
                        Sender::input_owner_account_id(connection_config.clone())?
                    }
                },
                None => cli_owner_account_id,
            },
            None => Sender::input_owner_account_id(connection_config.clone())?,
        };
        let send_to: super::receiver::SendTo = match item.send_to {
            Some(cli_send_to) => super::receiver::SendTo::from(
                cli_send_to,
                connection_config,
                owner_account_id.clone(),
            )?,
            None => super::receiver::SendTo::send_to(connection_config, owner_account_id.clone())?,
        };
        Ok(Self {
            owner_account_id,
            send_to,
        })
    }
}

impl Sender {
    fn input_owner_account_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        loop {
            let account_id: near_primitives::types::AccountId = Input::new()
                .with_prompt("What is the owner account ID?")
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
            signer_id: self.owner_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.send_to
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
