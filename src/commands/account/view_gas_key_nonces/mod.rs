use color_eyre::eyre::WrapErr;

use crate::common::{RpcResultExt, block_on};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ViewGasKeyNoncesContext)]
pub struct ViewGasKeyNonces {
    #[interactive_clap(skip_default_input_arg)]
    /// What Account ID does the gas key belong to?
    account_id: crate::types::account_id::AccountId,
    /// Enter the public key of the gas key:
    public_key: crate::types::public_key::PublicKey,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct ViewGasKeyNoncesContext(crate::network_view_at_block::ArgsForViewContext);

impl ViewGasKeyNoncesContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<ViewGasKeyNonces as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback = std::sync::Arc::new({
            let account_id: near_kit::AccountId = scope.account_id.clone().into();
            let public_key: near_kit::PublicKey = scope.public_key.clone().into();

            move |network_config, block_reference| {
                let gas_key_nonces = block_on(
                    network_config.client().rpc().view_gas_key_nonces(
                        &account_id,
                        &public_key,
                        block_reference.clone(),
                    ),
                )
                    .into_eyre()
                    .wrap_err_with(|| {
                        format!(
                            "Failed to fetch the gas key nonces for {} on account <{}>",
                            public_key, account_id
                        )
                    })?;

                crate::common::display_gas_key_nonces(&public_key, &gas_key_nonces.nonces);
                Ok(())
            }
        });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![scope.account_id.clone().into()],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<ViewGasKeyNoncesContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: ViewGasKeyNoncesContext) -> Self {
        item.0
    }
}

impl ViewGasKeyNonces {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "What Account ID does the gas key belong to?",
        )
    }
}
