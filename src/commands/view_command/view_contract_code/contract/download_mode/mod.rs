use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod download_contract;
mod hash_contract;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[interactive_clap(context = crate::common::SignerContext)]
///To view contract code you will need to choose next action
pub enum DownloadMode {
    #[strum_discriminants(strum(message = "Download a contract file"))]
    /// Download a contract file
    Download(self::download_contract::ContractFile),
    #[strum_discriminants(strum(message = "View a contract hash"))]
    /// View a contract hash
    Hash(self::hash_contract::ContractHash),
}

impl DownloadMode {
    pub async fn process(
        self,
        contract_id: near_primitives::types::AccountId,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        match self {
            DownloadMode::Download(contract_file) => {
                contract_file
                    .process(contract_id, network_connection_config)
                    .await
            }
            DownloadMode::Hash(contract_hash) => {
                contract_hash
                    .process(contract_id, network_connection_config)
                    .await
            }
        }
    }
}
