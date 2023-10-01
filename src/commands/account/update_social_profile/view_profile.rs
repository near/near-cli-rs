use color_eyre::eyre::WrapErr;

use crate::common::{CallResultExt, JsonRpcClientExt};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::UpdateSocialProfileContext)]
#[interactive_clap(output_context = AccountContext)]
pub struct Account {
    #[interactive_clap(skip_default_input_arg)]
    /// What is your account ID?
    account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_view_at_block::NetworkViewAtBlockArgs,
}

#[derive(Clone)]
pub struct AccountContext(crate::network_view_at_block::ArgsForViewContext);

impl AccountContext {
    pub fn from_previous_context(
        previous_context: super::UpdateSocialProfileContext,
        scope: &<Account as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let account_id: near_primitives::types::AccountId = scope.account_id.clone().into();
        let on_after_getting_block_reference_callback: crate::network_view_at_block::OnAfterGettingBlockReferenceCallback =
            std::sync::Arc::new({
                let account_id = account_id.clone();
                move |network_config, block_reference| {
                    let contract_account_id = (previous_context.get_contract_account_id)(network_config)?;

                    let social_db = network_config
                        .json_rpc_client()
                        .blocking_call_view_function(
                            &contract_account_id,
                            "get",
                            serde_json::json!({
                                "keys": vec![format!("{account_id}/profile/**")],
                            })
                            .to_string()
                            .into_bytes(),
                            block_reference.clone(),
                        )
                        .wrap_err_with(|| {format!("Failed to fetch query for view method: 'get {account_id}/profile/**'")})?
                        .parse_result_from_json::<crate::types::socialdb_types::SocialDb>()
                        .wrap_err_with(|| {
                            format!("Failed to parse view function call return value for {account_id}/profile.")
                        })?;

                    print_profile(social_db.accounts.get(&account_id));

                    Ok(())
                }
            });

        Ok(Self(crate::network_view_at_block::ArgsForViewContext {
            config: previous_context.global_context.config,
            interacting_with_account_ids: vec![account_id],
            on_after_getting_block_reference_callback,
        }))
    }
}

impl From<AccountContext> for crate::network_view_at_block::ArgsForViewContext {
    fn from(item: AccountContext) -> Self {
        item.0
    }
}

impl Account {
    pub fn input_account_id(
        context: &super::UpdateSocialProfileContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is your account ID?",
        )
    }
}

fn print_profile(optional_account_profile: Option<&crate::types::socialdb_types::AccountProfile>) {
    if let Some(account_profile) = optional_account_profile {
        eprintln!("{{");
        if let Some(name) = &account_profile.profile.name {
            eprintln!("  \"name\": \"{name}\",");
        }
        if let Some(image) = &account_profile.profile.image {
            eprintln!("  \"image\": {{");
            if let Some(url) = &image.url {
                eprintln!("    \"url\": \"{url}\",");
            }
            if let Some(ipfs_cid) = &image.ipfs_cid {
                eprintln!("    \"ipfs_cid\": \"{ipfs_cid}\",");
            }
            eprintln!("  }},");
        }
        if let Some(background_image) = &account_profile.profile.background_image {
            eprintln!("  \"background_image\": {{");
            if let Some(url) = &background_image.url {
                eprintln!("    \"url\": \"{url}\",");
            }
            if let Some(ipfs_cid) = &background_image.ipfs_cid {
                eprintln!("    \"ipfs_cid\": \"{ipfs_cid}\",");
            }
            eprintln!("  }},");
        }
        if let Some(description) = &account_profile.profile.description {
            eprintln!(
                "  \"description\": \"{}\",",
                description.replace('\n', "\\n")
            );
        }
        if let Some(linktree) = &account_profile.profile.linktree {
            eprintln!("  \"linktree\": {{");
            for (key, optional_value) in linktree.iter() {
                if let Some(value) = &optional_value {
                    eprintln!("    \"{key}\": \"{value}\",");
                }
            }
            eprintln!("  }},")
        }
        if let Some(tags) = &account_profile.profile.tags {
            eprintln!("  \"tags\": {{");
            for (key, value) in tags.iter() {
                eprintln!("    \"{key}\": \"{value}\",");
            }
            eprintln!("  }},")
        }
        eprintln!("}}");
    } else {
        eprintln!("{{}}");
    }
}
