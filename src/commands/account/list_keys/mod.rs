use color_eyre::eyre::Context;

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewListKeysContext)]
pub struct ViewListKeys {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID do you need to view?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewListKeysContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewListKeysContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewListKeys as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();

        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            move |network_config, block_reference| {
                let access_key_list = network_config
                    .json_rpc_client()
                    .blocking_call_view_access_key_list(
                        &account_id,
                        block_reference.clone(),
                    )
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch query AccessKeyList for {}",
                            &account_id
                        )
                    })?
                    .access_key_list_view()?;

                crate::common::display_access_key_list(&access_key_list.keys);
                Ok(())
            }
        });
        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            account_id: scope.account_id.clone(),
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewListKeysContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewListKeysContext) -> Self {
        item.0
    }
}

impl ViewListKeys {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What Account ID do you need to view?",
        )
    }
}
