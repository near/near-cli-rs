#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SenderContext)]
pub struct ContractHash {
    #[interactive_clap(subcommand)]
    pub selected_block_id: super::super::super::block_id::BlockId,
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
