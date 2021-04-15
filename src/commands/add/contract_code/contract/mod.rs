use dialoguer::{theme::ColorfulTheme, Input, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};
use std::io::Read;

mod initialize_mode;


#[derive(Debug, clap::Clap)]
pub enum CliContract {
    /// Add a contract file
    ContractFile(CliContractFile),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum Contract {
    #[strum_discriminants(strum(message = "Add a contract file"))]
    ContractFile(ContractFile),
}

impl From<CliContract> for Contract {
    fn from(item: CliContract) -> Self {
        match item {
            CliContract::ContractFile(cli_contract_file) => {
                Contract::ContractFile(cli_contract_file.into())
            }
        }
    }
}

impl Contract {
    pub fn choose_contract() -> Self {
        println!();
        let variants = ContractDiscriminants::iter().collect::<Vec<_>>();
        let contracts = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_contract = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(
                "To deploy contract code you will need to choose next action"
            )
            .items(&contracts)
            .default(0)
            .interact()
            .unwrap();
        let cli_contract = match variants[selected_contract] {
            ContractDiscriminants::ContractFile => CliContract::ContractFile(Default::default()),
        };
        Self::from(cli_contract)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {
        match self {
            Contract::ContractFile(contract_file) => {
                contract_file
                    .process(prepopulated_unsigned_transaction, selected_server_url)
                    .await
            }
        }
    }
}

/// add contract file
#[derive(Debug, Default, clap::Clap)]
pub struct CliContractFile {
    file_path: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    next_action: Option<self::initialize_mode::CliNextAction>
}

#[derive(Debug)]
pub struct ContractFile {
    pub file_path: std::path::PathBuf,
    next_action: self::initialize_mode::NextAction
}

impl From<CliContractFile> for ContractFile {
    fn from(item: CliContractFile) -> Self {
        let file_path = match item.file_path {
            Some(cli_file_path) => cli_file_path,
            None => ContractFile::input_file_path()
        };
        let next_action = match item.next_action {
            Some(cli_next_action) => self::initialize_mode::NextAction::from(cli_next_action),
            None => self::initialize_mode::NextAction::choose_next_action()
        };
        ContractFile {
            file_path,
            next_action
        }
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
        selected_server_url: Option<url::Url>,
    ) -> crate::CliResult {

        let mut f = std::fs::File::open(&self.file_path.clone())
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to open file: {:?}",
                    err
                ))
            })?;
        let mut code = Vec::new();
        f.read_to_end(&mut code)
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to read file: {:?}",
                    err
                ))
            })?;
        let action = near_primitives::transaction::Action::DeployContract(
            near_primitives::transaction::DeployContractAction {
                code
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.next_action
            .process(unsigned_transaction, selected_server_url)
            .await
    }
}
