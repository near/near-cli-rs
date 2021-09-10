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
    delete_public_key: Option<super::CliDeleteAccessKeyAction>,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub sender_account_id: near_primitives::types::AccountId,
    pub delete_public_key: super::DeleteAccessKeyAction,
}

impl CliSender {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .delete_public_key
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
            delete_public_key: Some(sender.delete_public_key.into()),
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
        let delete_public_key = match item.delete_public_key {
            Some(cli_delete_access_key) => super::DeleteAccessKeyAction::from(
                cli_delete_access_key,
                connection_config,
                sender_account_id.clone(),
            )?,
            None => super::DeleteAccessKeyAction::choose_delete_access_key_action(
                connection_config,
                sender_account_id.clone(),
            )?,
        };
        Ok(Self {
            sender_account_id,
            delete_public_key,
        })
    }
}

impl Sender {
    fn input_sender_account_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        loop {
            let account_id: near_primitives::types::AccountId = Input::new()
                .with_prompt("Which account ID do you need to remove the key from?")
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
        self.delete_public_key
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
