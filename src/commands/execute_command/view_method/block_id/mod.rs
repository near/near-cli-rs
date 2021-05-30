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

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this contract at final block"))]
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this contract at block heigt"))]
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View this contract at block hash"))]
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
        contract_account_id: String,
        selected_server_url: url::Url,
        method_name: String,
        args: Vec<u8>,
    ) -> crate::CliResult {
        println!();
        match self {
            Self::AtBlockHeight(block_id_height) => {
                block_id_height
                    .process(selected_server_url, contract_account_id, method_name, args)
                    .await
            }
            Self::AtBlockHash(block_id_hash) => {
                block_id_hash
                    .process(selected_server_url, contract_account_id, method_name, args)
                    .await
            }
            Self::AtFinalBlock => {
                self.at_final_block(selected_server_url, contract_account_id, method_name, args)
                    .await
            }
        }
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    async fn at_final_block(
        self,
        selected_server_url: url::Url,
        contract_account_id: String,
        method_name: String,
        args: Vec<u8>,
    ) -> crate::CliResult {
        let args: near_primitives::types::FunctionArgs =
            near_primitives::types::FunctionArgs::from(args);
        let query_view_method_response = self
            .rpc_client(&selected_server_url.as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::CallFunction {
                    account_id: contract_account_id,
                    method_name,
                    args,
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to fetch query for view method: {:?}", err))
            })?;
        let call_result =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
                query_view_method_response.kind
            {
                result.result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };
        let call_result_str = String::from_utf8(call_result).unwrap();
        let serde_call_result: serde_json::Value = serde_json::from_str(&call_result_str)
            .map_err(|err| color_eyre::Report::msg(format!("serde json: {:?}", err)))?;
        println!("--------------");
        println!();
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_call_result).unwrap()
        );
        Ok(())
    }
}
