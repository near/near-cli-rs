use color_eyre::owo_colors::OwoColorize;
use inquire::{formatter::MultiOptionFormatter, CustomType, MultiSelect};

use crate::common::JsonRpcClientExt;
use crate::common::RpcQueryResponseExt;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::DeleteKeysCommandContext)]
#[interactive_clap(output_context = PublicKeyListContext)]
pub struct PublicKeyList {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the public keys you wish to delete (separated by comma):
    public_keys: crate::types::public_key_list::PublicKeyList,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct PublicKeyListContext {
    global_context: crate::GlobalContext,
    owner_account_id: near_primitives::types::AccountId,
    public_keys: Vec<near_crypto::PublicKey>,
}

impl PublicKeyListContext {
    pub fn from_previous_context(
        previous_context: super::DeleteKeysCommandContext,
        scope: &<PublicKeyList as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            owner_account_id: previous_context.owner_account_id,
            public_keys: scope.public_keys.clone().into(),
        })
    }
}

impl From<PublicKeyListContext> for crate::commands::ActionContext {
    fn from(item: PublicKeyListContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let owner_account_id = item.owner_account_id.clone();

                move |_network_config| {
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: owner_account_id.clone(),
                        receiver_id: owner_account_id.clone(),
                        actions: item
                            .public_keys
                            .clone()
                            .into_iter()
                            .map(|public_key| {
                                near_primitives::transaction::Action::DeleteKey(Box::new(
                                    near_primitives::transaction::DeleteKeyAction { public_key },
                                ))
                            })
                            .collect(),
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.owner_account_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl PublicKeyList {
    fn input_public_keys_manually(
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key_list::PublicKeyList>> {
        Ok(Some(
                CustomType::new("Enter a comma-separated list of public keys you want to delete (for example, ed25519:FAXX...RUQa, ed25519:FgVF...oSWJ, ...):")
                    .prompt()?,
            ))
    }

    pub fn input_public_keys(
        context: &super::DeleteKeysCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::public_key_list::PublicKeyList>> {
        if context.global_context.offline {
            return Self::input_public_keys_manually();
        }
        let mut access_key_list: Vec<AccessKeyInfo> = vec![];
        let mut processed_network: Vec<String> = vec![];
        let mut errors: Vec<String> = vec![];
        for (_, network_config) in context.global_context.config.network_connection.iter() {
            if processed_network.contains(&network_config.network_name) {
                continue;
            }
            match network_config
                .json_rpc_client()
                .blocking_call_view_access_key_list(
                    &context.owner_account_id,
                    near_primitives::types::Finality::Final.into(),
                ) {
                Ok(rpc_query_response) => {
                    let access_key_list_for_network = rpc_query_response.access_key_list_view()?;
                    access_key_list.extend(access_key_list_for_network.keys.iter().map(
                        |access_key_info_view| AccessKeyInfo {
                            public_key: access_key_info_view.public_key.clone(),
                            permission: access_key_info_view.access_key.permission.clone(),
                            network_name: network_config.network_name.clone(),
                        },
                    ));
                    processed_network.push(network_config.network_name.to_string());
                }
                Err(err) => {
                    errors.push(err.to_string());
                }
            }
        }

        if access_key_list.is_empty() {
            for error in errors {
                println!("WARNING! {error}");
            }
            println!("Automatic search of access keys for <{}> is not possible on [{}] network(s).\nYou can enter access keys to remove manually.",
                context.owner_account_id,
                context.global_context.config.network_names().join(", "));
            return Self::input_public_keys_manually();
        }

        let formatter: MultiOptionFormatter<'_, AccessKeyInfo> = &|a| {
            let public_key_list = a
                .iter()
                .map(|list_option| list_option.value.to_string())
                .collect::<Vec<_>>();
            format!("\n{}", public_key_list.join("\n"))
        };

        let selected_public_keys = MultiSelect::new(
            "Select the public keys you want to delete:",
            access_key_list,
        )
        .with_formatter(formatter)
        .with_validator(
            |list: &[inquire::list_option::ListOption<&AccessKeyInfo>]| {
                if list.is_empty() {
                    Ok(inquire::validator::Validation::Invalid(
                        inquire::validator::ErrorMessage::Custom(
                            "At least one key must be selected (use space to select)".to_string(),
                        ),
                    ))
                } else {
                    Ok(inquire::validator::Validation::Valid)
                }
            },
        )
        .prompt()?
        .iter()
        .map(|access_key_info| access_key_info.public_key.clone())
        .collect::<Vec<_>>();

        Ok(Some(selected_public_keys.into()))
    }
}

#[derive(Debug, Clone)]
struct AccessKeyInfo {
    public_key: near_crypto::PublicKey,
    permission: near_primitives::views::AccessKeyPermissionView,
    network_name: String,
}

impl std::fmt::Display for AccessKeyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.permission {
            near_primitives::views::AccessKeyPermissionView::FullAccess => {
                write!(
                    f,
                    "{} {}\t{}",
                    self.network_name.blue(),
                    self.public_key.yellow(),
                    "full access".yellow()
                )
            }
            near_primitives::views::AccessKeyPermissionView::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            } => {
                let allowance_message = match allowance {
                    Some(amount) => format!(
                        "with a remaining fee allowance of {}",
                        near_token::NearToken::from_yoctonear(*amount)
                    ),
                    None => "with no limit".to_string(),
                };
                if method_names.is_empty() {
                    write!(
                        f,
                        "{} {}\t{} {} {}",
                        self.network_name.blue(),
                        self.public_key.green(),
                        "call any function on".green(),
                        receiver_id.green(),
                        allowance_message.green()
                    )
                } else {
                    write!(
                        f,
                        "{} {}\t{} {:?} {} {} {}",
                        self.network_name.blue(),
                        self.public_key.green(),
                        "call".green(),
                        method_names.green(),
                        "function(s) on".green(),
                        receiver_id.green(),
                        allowance_message.green()
                    )
                }
            }
        }
    }
}
