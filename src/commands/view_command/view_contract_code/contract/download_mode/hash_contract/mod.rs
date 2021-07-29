// view a contract hash
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliContractHash {
    #[clap(subcommand)]
    selected_block_id: Option<super::super::super::block_id::CliBlockId>,
}

#[derive(Debug, Clone)]
pub struct ContractHash {
    pub selected_block_id: super::super::super::block_id::BlockId,
}

impl ContractHash {
    pub fn from(item: CliContractHash) -> Self {
        let selected_block_id: super::super::super::block_id::BlockId = match item.selected_block_id
        {
            Some(cli_block_id) => cli_block_id.into(),
            None => super::super::super::block_id::BlockId::choose_block_id(),
        };
        ContractHash { selected_block_id }
    }
}

impl ContractHash {
    pub async fn process(
        self,
        contract_id: String,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(contract_id, network_connection_config, None)
            .await
    }
}
