use async_recursion::async_recursion;
use dialoguer::Input;

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
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct ContractFile {
    pub file_path: std::path::PathBuf,
    pub next_action: Box<super::NextAction>,
}

impl interactive_clap::ToCli for ContractFile {
    type CliVariant = CliContractFile;
}

impl CliContractFile {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
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
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl ContractFile {
    pub fn from_cli(
        optional_clap_variant: Option<<ContractFile as interactive_clap::ToCli>::CliVariant>,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let file_path = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.file_path)
        {
            Some(cli_file_path) => cli_file_path,
            None => Self::input_file_path(&context)?,
        };
        let skip_next_action: super::NextAction =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.next_action) {
                Some(cli_skip_action) => {
                    super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
                }
                None => super::NextAction::choose_variant(context)?,
            };
        Ok(Self {
            file_path,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl ContractFile {
    fn input_file_path(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<std::path::PathBuf> {
        println!();
        let input_file_path: String = Input::new()
            .with_prompt("What is a file location of the contract?")
            .interact_text()?;
        let file_path = shellexpand::tilde(&input_file_path).as_ref().into();
        Ok(file_path)
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let code = std::fs::read(&self.file_path.clone())
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
        match *self.next_action {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
