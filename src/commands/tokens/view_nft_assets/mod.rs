use color_eyre::eyre::Context;
use serde_json::json;

use crate::common::CallResultExt;
use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = ViewNftAssetsContext)]
pub struct ViewNftAssets {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the nft-contract account ID?
    nft_contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewNftAssetsContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewNftAssetsContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<ViewNftAssets as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let owner_account_id = previous_context.owner_account_id.clone();
            let nft_contract_account_id: near_primitives::types::AccountId =
                scope.nft_contract_account_id.clone().into();

            move |network_config, block_reference| {
                let args = serde_json::to_vec(&json!({
                        "account_id": owner_account_id.to_string(),
                    }))?;
                let call_result = network_config
                    .json_rpc_client()
                    .blocking_call_view_function(
                        &nft_contract_account_id,
                        "nft_tokens_for_owner",
                        args,
                        block_reference.clone(),
                    )
                    .wrap_err_with(||{
                        format!("Failed to fetch query for view method: 'nft_tokens_for_owner' (contract <{}> on network <{}>)",
                            nft_contract_account_id,
                            network_config.network_name
                        )
                    })?;
                call_result.print_logs();
                let serde_call_result: serde_json::Value = call_result.parse_result_from_json()?;

                eprintln!("\n{} account has NFT tokens:", owner_account_id);
                eprintln!("{}", serde_json::to_string_pretty(&serde_call_result)?);
                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            on_after_getting_block_reference_callback,
            interacting_with_account_ids: vec![
                scope.nft_contract_account_id.clone().into(),
                previous_context.owner_account_id,
            ],
        }))
    }
}

impl From<ViewNftAssetsContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewNftAssetsContext) -> Self {
        item.0
    }
}

impl ViewNftAssets {
    pub fn input_nft_contract_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the nft-contract account ID?",
        )
    }
}
