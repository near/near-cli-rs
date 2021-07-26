use dialoguer::Input;

/// данные об отправителе транзакции
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
    #[clap(subcommand)]
    public_key_mode: Option<super::public_key_mode::CliPublicKeyMode>,
}

#[derive(Debug)]
pub struct Sender {
    pub sender_account_id: String,
    pub public_key_mode: super::public_key_mode::PublicKeyMode,
}

impl Sender {
    pub fn from(
        item: CliSender,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::check_account_id(
                    network_connection_config.clone(),
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
        let public_key_mode = match item.public_key_mode {
            Some(cli_public_key_mode) => super::public_key_mode::PublicKeyMode::from(
                cli_public_key_mode,
                connection_config,
                sender_account_id.clone(),
            )?,
            None => super::public_key_mode::PublicKeyMode::choose_public_key_mode(
                connection_config,
                sender_account_id.clone(),
            )?,
        };
        Ok(Self {
            sender_account_id,
            public_key_mode,
        })
    }
}

impl Sender {
    fn input_sender_account_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<String> {
        loop {
            let account_id: String = Input::new()
                    .with_prompt("What account ID do you need to add a key?")
                    .interact_text()
                    .unwrap();
            if let Some(connection_config) = &connection_config {
                if let Some(_) = crate::common::check_account_id(
                    connection_config.clone(),
                    account_id.clone(),
                )? {
                    break Ok(account_id);
                } else {
                    println!("Account <{}> doesn't exist", account_id);
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
        self.public_key_mode
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
