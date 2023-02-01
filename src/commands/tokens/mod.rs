use strum::{EnumDiscriminants, EnumIter, EnumMessage};

// mod send_ft;
mod send_near;
// mod send_nft;
// mod view_ft_balance;
// mod view_near_balance;
// mod view_nft_assets;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = crate::commands::TransactionContext)]
pub struct TokensCommands {
    ///What is your account ID?
    owner_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    tokens_actions: TokensActions,
}

#[derive(Debug, Clone)]
struct TokensCommandsContext {
    config: crate::config::Config,
    owner_account_id: crate::types::account_id::AccountId,
}

impl TokensCommandsContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TokensCommands as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            config: previous_context.0,
            owner_account_id: scope.owner_account_id.clone(),
        }
    }
}

impl From<TokensCommandsContext> for crate::commands::TransactionContext {
    fn from(item: TokensCommandsContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.owner_account_id.clone(),
            receiver_account_id: item.owner_account_id,
            actions: vec![]
        }
    }
}


impl TokensCommands {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        self.tokens_actions
            .process(config, self.owner_account_id.clone().into())
            .await
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::TransactionContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select actions with tokens
pub enum TokensActions {
    #[strum_discriminants(strum(
        message = "send-near         - The transfer is carried out in NEAR tokens"
    ))]
    ///The transfer is carried out in NEAR tokens
    SendNear(self::send_near::SendNearCommand),
    // #[strum_discriminants(strum(
    //     message = "send-ft           - The transfer is carried out in FT tokens"
    // ))]
    // ///The transfer is carried out in FT tokens
    // SendFt(self::send_ft::SendFtCommand),
    // #[strum_discriminants(strum(
    //     message = "send-nft          - The transfer is carried out in NFT tokens"
    // ))]
    // ///The transfer is carried out in NFT tokens
    // SendNft(self::send_nft::SendNftCommand),
    // #[strum_discriminants(strum(message = "view-near-balance - View the balance of Near tokens"))]
    // ///View the balance of Near tokens
    // ViewNearBalance(self::view_near_balance::ViewNearBalance),
    // #[strum_discriminants(strum(message = "view-ft-balance   - View the balance of FT tokens"))]
    // ///View the balance of FT tokens
    // ViewFtBalance(self::view_ft_balance::ViewFtBalance),
    // #[strum_discriminants(strum(message = "view-nft-assets   - View the balance of NFT tokens"))]
    // ///View the balance of NFT tokens
    // ViewNftAssets(self::view_nft_assets::ViewNftAssets),
}

impl TokensActions {
    async fn process(
        &self,
        config: crate::config::Config,
        owner_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        match self {
            Self::SendNear(send_near_command) => {
                send_near_command.process(config, owner_account_id).await
            }
            // Self::ViewNearBalance(view_near_balance) => {
            //     view_near_balance.process(config, owner_account_id).await
            // }
            // Self::SendFt(send_ft_command) => {
            //     send_ft_command.process(config, owner_account_id).await
            // }
            // Self::SendNft(send_nft_command) => {
            //     send_nft_command.process(config, owner_account_id).await
            // }
            // Self::ViewFtBalance(view_ft_balance) => {
            //     view_ft_balance.process(config, owner_account_id).await
            // }
            // Self::ViewNftAssets(view_nft_assets) => {
            //     view_nft_assets.process(config, owner_account_id).await
            // }
        }
    }
}

#[derive(serde::Deserialize)]
struct FtMetadata {
    symbol: String,
    decimals: u64,
}

async fn params_ft_metadata(
    config: crate::config::Config,
    ft_contract_account_id: crate::types::account_id::AccountId,
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
) -> color_eyre::eyre::Result<FtMetadata> {
    let query_view_ft_metadata_response = network_config
        .get_network_config(config.clone())
        .json_rpc_client()
        .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
            block_reference: network_config.get_block_ref(),
            request: near_primitives::views::QueryRequest::CallFunction {
                account_id: ft_contract_account_id.clone().into(),
                method_name: "ft_metadata".to_string(),
                args: near_primitives::types::FunctionArgs::from(vec![]),
            },
        })
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to fetch query for view method: {:?}", err))
        })?;
    let call_result =
        if let near_jsonrpc_primitives::types::query::QueryResponseKind::CallResult(result) =
            query_view_ft_metadata_response.kind
        {
            result.result
        } else {
            return Err(color_eyre::Report::msg("Error call result".to_string()));
        };
    let ft_metadata: FtMetadata = serde_json::from_slice(&call_result).map_err(|err| {
        color_eyre::Report::msg(format!("Impossible to get FT metadata! Error: {}", err))
    })?;
    Ok(ft_metadata)
}
