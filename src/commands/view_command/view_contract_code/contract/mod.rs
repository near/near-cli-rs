use dialoguer::Input;

mod download_mode;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliSendTo {
    /// Specify a contract
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

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Contract(cli_sender) => {
                let sender = Contract::from(cli_sender);
                Self::Contract(sender)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Contract(Default::default()))
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            SendTo::Contract(sender) => sender.process(network_connection_config).await,
        }
    }
}

/// Specify a contract
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContract {
    pub contract_id: Option<near_primitives::types::AccountId>,
    #[clap(subcommand)]
    download_mode: Option<self::download_mode::CliDownloadMode>,
}

#[derive(Debug, Clone)]
pub struct Contract {
    pub contract_id: near_primitives::types::AccountId,
    pub download_mode: self::download_mode::DownloadMode,
}

impl CliContract {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .download_mode
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(contract_id) = &self.contract_id {
            args.push_front(contract_id.to_string());
        };
        args
    }
}

impl From<Contract> for CliContract {
    fn from(contract: Contract) -> Self {
        Self {
            contract_id: Some(contract.contract_id),
            download_mode: Some(contract.download_mode.into()),
        }
    }
}

impl From<CliContract> for Contract {
    fn from(item: CliContract) -> Self {
        let contract_id: near_primitives::types::AccountId = match item.contract_id {
            Some(cli_contract_id) => cli_contract_id,
            None => Contract::input_contract_id(),
        };
        let download_mode = match item.download_mode {
            Some(cli_download_mode) => {
                self::download_mode::DownloadMode::from(cli_download_mode, &contract_id.to_string())
            }
            None => self::download_mode::DownloadMode::choose_download_mode(&contract_id.to_string()),
        };
        Self {
            contract_id,
            download_mode,
        }
    }
}

impl Contract {
    pub fn input_contract_id() -> near_primitives::types::AccountId {
        println!();
        Input::new()
            .with_prompt("What contract do you need to view?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.download_mode
            .process(self.contract_id, network_connection_config)
            .await
    }
}
