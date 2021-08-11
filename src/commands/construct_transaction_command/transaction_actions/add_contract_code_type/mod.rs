use dialoguer::Input;
use std::io::Read;

mod initialize_mode;

/// add contract file
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContractFile {
    file_path: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    contract_mode: Option<self::initialize_mode::CliContractMode>,
}

#[derive(Debug, Clone)]
pub struct ContractFile {
    pub file_path: std::path::PathBuf,
    contract_mode: self::initialize_mode::ContractMode,
}

impl CliContractFile {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .contract_mode
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(file_path) = &self.file_path {
            args.push_front(file_path.as_path().display().to_string());
        }
        args
    }
}

impl From<ContractFile> for CliContractFile {
    fn from(contract_file: ContractFile) -> Self {
        Self {
            file_path: Some(contract_file.file_path),
            contract_mode: Some(contract_file.contract_mode.into()),
        }
    }
}

impl ContractFile {
    pub fn from(
        item: CliContractFile,
        connection_config: Option<crate::common::ConnectionConfig>,
        sender_account_id: near_primitives::types::AccountId,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path = match item.file_path {
            Some(cli_file_path) => cli_file_path,
            None => ContractFile::input_file_path(),
        };
        let contract_mode = match item.contract_mode {
            Some(cli_contract_mode) => self::initialize_mode::ContractMode::from(
                cli_contract_mode,
                connection_config,
                sender_account_id,
            )?,
            None => self::initialize_mode::ContractMode::choose_contract_mode(
                connection_config,
                sender_account_id,
            )?,
        };
        Ok(ContractFile {
            file_path,
            contract_mode,
        })
    }
}

impl ContractFile {
    fn input_file_path() -> std::path::PathBuf {
        println!();
        let input_file_path: String = Input::new()
            .with_prompt("What is a file location of the contract?")
            .interact_text()
            .unwrap();
        let mut path = std::path::PathBuf::new();
        path.push(input_file_path);
        println!("path: {:?}", &path);
        path
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let mut code = Vec::new();
        std::fs::File::open(&self.file_path.clone())
            .map_err(|err| color_eyre::Report::msg(format!("Failed to open file: {:?}", err)))?
            .read_to_end(&mut code)
            .map_err(|err| color_eyre::Report::msg(format!("Failed to read file: {:?}", err)))?;
        let action = near_primitives::transaction::Action::DeployContract(
            near_primitives::transaction::DeployContractAction { code },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.contract_mode
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
