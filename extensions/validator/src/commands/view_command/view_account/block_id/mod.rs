use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod block_id_hash;
mod block_id_height;

#[derive(Debug, Clone, clap::Clap)]
pub enum CliBlockId {
    /// Specify a block ID final to view this account
    AtFinalBlock,
    /// Specify a block ID height to view this account
    AtBlockHeight(self::block_id_height::CliBlockIdHeight),
    /// Specify a block ID hash to view this account
    AtBlockHash(self::block_id_hash::CliBlockIdHash),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum BlockId {
    #[strum_discriminants(strum(message = "View this account at final block"))]
    AtFinalBlock,
    #[strum_discriminants(strum(message = "View this account at block heigt"))]
    AtBlockHeight(self::block_id_height::BlockIdHeight),
    #[strum_discriminants(strum(message = "View this account at block hash"))]
    AtBlockHash(self::block_id_hash::BlockIdHash),
}

impl CliBlockId {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::AtFinalBlock => {
                let mut args = std::collections::VecDeque::new();
                args.push_front("at-final-block".to_owned());
                args
            }
            Self::AtBlockHeight(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("at-block-height".to_owned());
                args
            }
            Self::AtBlockHash(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("at-block-hash".to_owned());
                args
            }
        }
    }
}

impl From<BlockId> for CliBlockId {
    fn from(block_id: BlockId) -> Self {
        match block_id {
            BlockId::AtFinalBlock => Self::AtFinalBlock,
            BlockId::AtBlockHeight(block_id_height) => Self::AtBlockHeight(block_id_height.into()),
            BlockId::AtBlockHash(block_id_hash) => Self::AtBlockHash(block_id_hash.into()),
        }
    }
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
        sender_account_id: near_primitives::types::AccountId,
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
                self.display_account_info(sender_account_id.clone(), &network_connection_config)
                    .await?;
                self.display_access_key_list(sender_account_id.clone(), &network_connection_config)
                    .await?;
                Ok(())
            }
        }
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    async fn display_account_info(
        &self,
        account_id: near_primitives::types::AccountId,
        network_connection_config: &crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccount {
                    account_id: account_id.clone(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view account: {:?}",
                    err
                ))
            })?;
        let account_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };

        println!(
            "Account details for '{}' at block #{} ({})\n\
            Native account balance: {}\n\
            Validator stake: {}\n\
            Storage used by the account: {} bytes",
            account_id,
            query_view_method_response.block_height,
            query_view_method_response.block_hash,
            crate::common::NearBalance::from_yoctonear(account_view.amount),
            crate::common::NearBalance::from_yoctonear(account_view.locked),
            account_view.storage_usage
        );
        if account_view.code_hash == near_primitives::hash::CryptoHash::default() {
            println!("Contract code is not deployed to this account.");
        } else {
            println!(
                "Contract code SHA-256 checksum (hex): {}",
                hex::encode(account_view.code_hash.as_ref())
            );
        }
        Ok(())
    }

    async fn display_access_key_list(
        &self,
        account_id: near_primitives::types::AccountId,
        network_connection_config: &crate::common::ConnectionConfig,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(network_connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                    account_id: account_id.clone(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view key list: {:?}",
                    err
                ))
            })?;
        let access_key_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };

        println!("Number of access keys: {}", access_key_view.keys.len());
        for (index, access_key) in access_key_view.keys.iter().enumerate() {
            let permissions_message = match &access_key.access_key.permission {
                near_primitives::views::AccessKeyPermissionView::FullAccess => {
                    "full access".to_owned()
                }
                near_primitives::views::AccessKeyPermissionView::FunctionCall {
                    allowance,
                    receiver_id,
                    method_names,
                } => {
                    let allowance_message = match allowance {
                        Some(amount) => format!(
                            "with an allowance of {}",
                            crate::common::NearBalance::from_yoctonear(*amount)
                        ),
                        None => format!("with no limit"),
                    };
                    format!(
                        "only do {:?} function calls on {} {}",
                        method_names, receiver_id, allowance_message
                    )
                }
            };
            println!(
                "{: >4}. {} (nonce: {}) is granted to {}",
                index + 1,
                access_key.public_key,
                access_key.access_key.nonce,
                permissions_message
            );
        }
        Ok(())
    }
}
