use dialoguer::Input;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendTo {
    /// Specify a contract ID
    Contract(CliContract),
}

#[derive(Debug, Clone)]
pub enum SendTo {
    Contract(Contract),
}

impl CliSendTo {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::Contract(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("contract".to_owned());
                args
            }
        }
    }
}

impl From<SendTo> for CliSendTo {
    fn from(send_to: SendTo) -> Self {
        match send_to {
            SendTo::Contract(contract) => Self::Contract(contract.into()),
        }
    }
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
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContract {
    contract_account_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    call: Option<super::CliCallFunction>,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub contract_account_id: near_primitives::types::AccountId,
    pub call: super::CallFunction,
}

impl CliContract {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .call
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(contract_account_id) = &self.contract_account_id {
            args.push_front(contract_account_id.to_string());
        }
        args
    }
}

impl From<Contract> for CliContract {
    fn from(contract: Contract) -> Self {
        Self {
            contract_account_id: Some(contract.contract_account_id),
            call: Some(contract.call.into()),
        }
    }
}

impl Contract {
    fn from(
        item: CliContract,
        connection_config: Option<crate::common::ConnectionConfig>,
    ) -> color_eyre::eyre::Result<Self> {
        let contract_account_id: near_primitives::types::AccountId = match item.contract_account_id
        {
            Some(cli_contract_account_id) => match &connection_config {
                Some(network_connection_config) => match crate::common::get_account_state(
                    network_connection_config,
                    cli_contract_account_id.clone(),
                )? {
                    Some(_) => cli_contract_account_id,
                    None => {
                        println!("Account <{}> doesn't exist", cli_contract_account_id);
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
    ) -> color_eyre::eyre::Result<near_primitives::types::AccountId> {
        loop {
            let account_id: near_primitives::types::AccountId = Input::new()
                .with_prompt("What is the account ID of the contract?")
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
            receiver_id: self.contract_account_id.clone(),
            ..prepopulated_unsigned_transaction
        };
        self.call
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
