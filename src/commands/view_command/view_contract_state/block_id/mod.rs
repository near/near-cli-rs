use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, clap::Clap)]
pub enum CliBlockId {
    /// Specify a block ID final to view this contract
    AtFinalBlock,
    /// Specify a block ID height to view this contract
    AtBlockHeight(self::block_id_height::CliBlockIdHeight),
    /// Specify a block ID hash to view this contract
    AtBlockHash(self::block_id_hash::CliBlockIdHash),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum BlockId {
    #[strum_discriminants(strum(message = "View state this contract at final block"))]
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View state this contract at block heigt"))]
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View state this contract at block hash"))]
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl From<CliBlockId> for BlockId {
    fn from(item: CliBlockId) -> Self {
        match item {
            CliBlockId::AtFinalBlock => Self::AtFinalBlock,
            CliBlockId::AtBlockHeight(cli_block_id_height) => {
                Self::AtBlockHeight(cli_block_id_height.into())
            }
            CliBlockId::AtBlockHash(cli_block_id_hash) => {
                Self::AtBlockHash(cli_block_id_hash.into())
            }
        }
    }
}

impl BlockId {
    pub fn choose_block_id() -> Self {
        println!();
        let variants = BlockIdDiscriminants::iter().collect::<Vec<_>>();
        let blocks = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your action")
            .items(&blocks)
            .default(0)
            .interact()
            .unwrap();
        let cli_block_id = match variants[selection] {
            BlockIdDiscriminants::AtFinalBlock => CliBlockId::AtFinalBlock,
            BlockIdDiscriminants::AtBlockHeight => CliBlockId::AtBlockHeight(Default::default()),
            BlockIdDiscriminants::AtBlockHash => CliBlockId::AtBlockHash(Default::default()),
        };
        Self::from(cli_block_id)
    }

    pub async fn process(
        self,
        sender_account_id: String,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(sender_account_id, network_connection_config)
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(sender_account_id, network_connection_config)
                    .await
            }
            Self::AtFinalBlock => {
                self.at_final_block(sender_account_id, network_connection_config)
                    .await
            }
        }
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    async fn at_final_block(
        self,
        sender_account_id: String,
        network_connection_config: crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewState {
                    account_id: sender_account_id,
                    prefix: near_primitives::types::StoreKey::from(vec![]),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view account: {:?}",
                    err
                ))
            })?;
        let call_access_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewState(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        println!(
            "\nContract state (values):\n{:#?}\n",
            &call_access_view.values
        );
        println!(
            "\nContract state (proof):\n{:#?}\n",
            &call_access_view.proof
        );
        Ok(())
    }
}
