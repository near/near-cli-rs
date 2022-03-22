use dialoguer::Input;

mod initialize_mode;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct ContractFile {
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    next_action: self::initialize_mode::NextAction,
}

impl ContractFile {
    fn input_file_path(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::types::path_buf::PathBuf> {
        println!();
        let input_file_path: String = Input::new()
            .with_prompt("What is a file location of the contract?")
            .interact_text()?;
        let file_path = if cfg!(unix) {
            shellexpand::tilde(&input_file_path).as_ref().parse()?
        } else {
            input_file_path.parse()?
        };
        Ok(file_path)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let code = std::fs::read(&self.file_path.0.clone())
            .map_err(|err| color_eyre::Report::msg(format!("Failed to open file: {:?}", err)))?;
        let action = near_primitives::transaction::Action::DeployContract(
            near_primitives::transaction::DeployContractAction { code },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        self.next_action
            .process(unsigned_transaction, network_connection_config)
            .await
    }
}
