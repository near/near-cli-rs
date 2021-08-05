use dialoguer::Input;

// download contract file
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContractFile {
    file_path: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    selected_block_id: Option<super::super::super::block_id::CliBlockId>,
}

#[derive(Debug, Clone)]
pub struct ContractFile {
    pub file_path: Option<std::path::PathBuf>,
    pub selected_block_id: super::super::super::block_id::BlockId,
}

impl CliContractFile {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .selected_block_id
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(file_path) = &self.file_path {
            args.push_front(file_path.as_path().display().to_string());
        };
        args
    }
}

impl From<ContractFile> for CliContractFile {
    fn from(contract_file: ContractFile) -> Self {
        Self {
            file_path: contract_file.file_path,
            selected_block_id: Some(contract_file.selected_block_id.into()),
        }
    }
}

impl ContractFile {
    pub fn from(item: CliContractFile, contract_id: &str) -> Self {
        let file_path = match item.file_path {
            Some(cli_file_path) => Some(cli_file_path),
            None => ContractFile::input_file_path(contract_id),
        };
        let selected_block_id: super::super::super::block_id::BlockId = match item.selected_block_id
        {
            Some(cli_block_id) => cli_block_id.into(),
            None => super::super::super::block_id::BlockId::choose_block_id(),
        };
        ContractFile {
            file_path,
            selected_block_id,
        }
    }
}

impl ContractFile {
    fn input_file_path(contract_id: &str) -> Option<std::path::PathBuf> {
        println!();
        let input_file_path: String = Input::new()
            .with_prompt("Where to download the contract file?")
            .with_initial_text(format!("{}.wasm", contract_id))
            .interact_text()
            .unwrap();
        Some(input_file_path.into())
    }

    pub async fn process(
        self,
        contract_id: String,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(contract_id, network_connection_config, self.file_path)
            .await
    }
}
