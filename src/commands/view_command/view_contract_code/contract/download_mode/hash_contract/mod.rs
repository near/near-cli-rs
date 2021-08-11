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

impl CliContractHash {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let args = self
            .selected_block_id
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        args
    }
}

impl From<ContractHash> for CliContractHash {
    fn from(contract_hash: ContractHash) -> Self {
        Self {
            selected_block_id: Some(contract_hash.selected_block_id.into()),
        }
    }
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
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        self.selected_block_id
            .process(contract_id, network_connection_config, None)
            .await
    }
}
