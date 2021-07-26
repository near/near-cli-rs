use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a receiver
    Contract(CliReceiver),
}

#[derive(Debug)]
pub enum SendTo {
    Contract(Receiver),
}

impl SendTo {
    pub fn from(
        item: CliSendTo,
        connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<Self> {
        match item {
            CliSendTo::Contract(cli_receiver) => {
                let receiver = Receiver::from(cli_receiver, connection_config)?;
                Ok(Self::Contract(receiver))
            }
        }
    }
}

impl SendTo {
    pub fn send_to(
        connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<Self> {
        Self::from(CliSendTo::Contract(Default::default()), connection_config)
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            SendTo::Contract(receiver) => receiver.process(network_connection_config).await,
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
pub struct CliReceiver {
    contract_account_id: Option<String>,
    #[clap(subcommand)]
    call: Option<super::CliCallFunction>,
}

#[derive(Debug)]
pub struct Receiver {
    pub contract_account_id: String,
    pub call: super::CallFunction,
}

impl Receiver {
    fn from(
        item: CliReceiver,
        connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<Self> {
        let contract_account_id: String = match item.contract_account_id {
            Some(cli_contract_account_id) => {
                let contract_code_hash: near_primitives::hash::CryptoHash =
                    match crate::common::check_account_id(
                        connection_config.clone(),
                        cli_contract_account_id.clone(),
                    )? {
                        Some(account_view) => account_view.code_hash,
                        None => near_primitives::hash::CryptoHash::default(),
                    };
                if contract_code_hash == near_primitives::hash::CryptoHash::default() {
                    println!(
                        "Contract code is not deployed to this account <{}>.",
                        cli_contract_account_id
                    );
                    Receiver::input_contract_account_id(connection_config)?
                } else {
                    cli_contract_account_id
                }
            }
            None => Receiver::input_contract_account_id(connection_config)?,
        };
        let call = match item.call {
            Some(cli_call) => cli_call.into(),
            None => super::CallFunction::choose_call_function(),
        };
        Ok(Self {
            contract_account_id,
            call,
        })
    }
}

impl Receiver {
    fn input_contract_account_id(
        connection_config: crate::common::ConnectionConfig,
    ) -> color_eyre::eyre::Result<String> {
        loop {
            let contract_account_id: String = Input::new()
                .with_prompt("What is the account ID of the contract?")
                .interact_text()
                .unwrap();
            let contract_code_hash: near_primitives::hash::CryptoHash =
                match crate::common::check_account_id(
                    connection_config.clone(),
                    contract_account_id.clone(),
                )? {
                    Some(account_view) => account_view.code_hash,
                    None => near_primitives::hash::CryptoHash::default(),
                };
            if contract_code_hash == near_primitives::hash::CryptoHash::default() {
                println!(
                    "Contract code is not deployed to this account <{}>.",
                    contract_account_id
                )
            } else {
                break Ok(contract_account_id);
            }
        }
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.call
            .process(network_connection_config, self.contract_account_id)
            .await
    }
}
