use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a contract ID
    Contract(CliContract),
}

#[derive(Debug)]
pub enum SendTo {
    Contract(Contract),
}

impl SendTo {
    pub fn from(
        item: CliSendTo,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Contract(cli_contract) => {
                let contract = Contract::from(cli_contract, connection_config)?;
                Ok(Self::Contract(contract))
            }
        }
    }
}

impl SendTo {
    pub fn send_to(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        Self::from(CliSendTo::Contract(Default::default()), connection_config)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        match self {
            SendTo::Contract(receiver) => {
                receiver
                    .process(prepopulated_unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}

/// данные о контракте
#[derive(Debug, Default, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContract {
    contract_account_id: Option<String>,
    #[clap(subcommand)]
    call: Option<super::CliCallFunction>,
}

#[derive(Debug)]
pub struct Contract {
    pub contract_account_id: String,
    pub call: super::CallFunction,
}

impl Contract {
    fn from(
        item: CliContract,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let contract_account_id: String = match item.contract_account_id {
            Some(cli_contract_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::check_account_id(
                    network_connection_config.clone(),
                    cli_contract_account_id.clone(),
                )? {
                    Some(_) => cli_contract_account_id,
                    None => {
                        println!(
                            "This account ID <{}> doesn't exist",
                            cli_contract_account_id
                        );
                        Contract::input_receiver_account_id(connection_config.clone())?
                    }
                },
                None => cli_contract_account_id,
            },
            None => Contract::input_receiver_account_id(connection_config.clone())?,
        };
        let call = match item.call {
            Some(cli_call) => super::CallFunction::from(cli_call, connection_config)?,
            None => super::CallFunction::choose_call_function(connection_config)?,
        };
        Ok(Self {
            contract_account_id,
            call,
        })
    }
}

impl Contract {
    fn input_receiver_account_id(
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<String> {
        match &connection_config {
            Some(connection_config) => loop {
                let account_id: String = Input::new()
                    .with_prompt("What is the account ID of the contract?")
                    .interact_text()
                    .unwrap();
                match crate::common::check_account_id(
                    connection_config.clone(),
                    account_id.clone(),
                )? {
                    Some(_) => break Ok(account_id),
                    None => println!("This account ID <{}> doesn't exist", account_id),
                };
            },
            None => Ok(Input::new()
                .with_prompt("What is the account ID of the contract?")
                .interact_text()
                .unwrap()),
        }
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let unsigned_transaction = near_primitives::transaction::Transaction {
            receiver_id: self.contract_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.call
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
