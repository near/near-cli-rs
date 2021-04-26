use dialoguer::Input;

mod download_mode;


#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify a contract
    Contract(CliContract),
}

#[derive(Debug)]
pub enum SendTo {
    Contract(Contract),
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
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        match self {
            SendTo::Contract(sender) => {
                sender
                    .process(selected_server_url)
                    .await
            }
        }
    }
}

/// Specify a contract
#[derive(Debug, Default, clap::Clap)]
pub struct CliContract {
    pub contract_id: Option<String>,
    #[clap(subcommand)]
    download_mode: Option<self::download_mode::CliDownloadMode>,
}

#[derive(Debug)]
pub struct Contract {
    pub contract_id: String,
    pub download_mode: self::download_mode::DownloadMode,
}

impl From<CliContract> for Contract {
    fn from(item: CliContract) -> Self {
        let contract_id: String = match item.contract_id {
            Some(cli_contract_id) => cli_contract_id,
            None => Contract::input_contract_id(),
        };
        let download_mode = match item.download_mode {
            Some(cli_download_mode) => self::download_mode::DownloadMode::from(cli_download_mode, &contract_id),
            None => self::download_mode::DownloadMode::choose_download_mode(&contract_id)
        };
        Self {
            contract_id,
            download_mode,
        }
    }
}

impl Contract {
    pub fn input_contract_id() -> String {
        println!();
        Input::new()
            .with_prompt("What contract do you need to view?")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        self.download_mode
            .process(self.contract_id, selected_server_url)
            .await
    }
}
