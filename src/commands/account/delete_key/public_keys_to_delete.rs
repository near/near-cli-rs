use color_eyre::owo_colors::OwoColorize;
use inquire::ui::{Color, RenderConfig, Styled};
use inquire::{CustomType, MultiSelect, formatter::MultiOptionFormatter};

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
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
            sign_as_delegate_action: false,
            on_sending_delegate_action_callback: None,
        }
    }
}

impl PublicKeyList {
    fn input_public_keys_manually()
    -> color_eyre::eyre::Result<Option<crate::types::public_key_list::PublicKeyList>> {
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
                    access_key_list.extend(access_key_list_for_network.keys.iter().filter_map(
                        |access_key_info_view| {
                            // In 2.13 the access-key list returns a `PublicKeyHandle`.
                            // ML-DSA-65 keys are stored on-chain only as a hash, so the
                            // full public key needed to build a DeleteKey action can't be
                            // recovered here; `full_pubkey()` returns `None` for them and
                            // they are skipped from the interactive picker.
                            access_key_info_view
                                .public_key
                                .full_pubkey()
                                .map(|public_key| AccessKeyInfo {
                                    public_key,
                                    permission: access_key_info_view.access_key.permission.clone(),
                                    network_name: network_config.network_name.clone(),
                                })
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
            println!(
                "Automatic search of access keys for <{}> is not possible on [{}] network(s).\nYou can enter access keys to remove manually.",
                context.owner_account_id,
                context.global_context.config.network_names().join(", ")
            );
            return Self::input_public_keys_manually();
        }

        let formatter: MultiOptionFormatter<'_, AccessKeyInfo> = &|a| {
            let public_key_list = a
                .iter()
                .map(|list_option| list_option.value.detailed_description())
                .collect::<Vec<_>>();
            public_key_list.join("\n").to_string()
        };

        let selected_public_keys = MultiSelect::new(
            "Select the public keys you want to delete:",
            access_key_list,
        )
        .with_render_config(get_multi_select_render_config())
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

impl AccessKeyInfo {
    fn permission_summary(&self) -> String {
        match &self.permission {
            near_primitives::views::AccessKeyPermissionView::FullAccess => {
                "full access".to_string()
            }
            near_primitives::views::AccessKeyPermissionView::FunctionCall {
                receiver_id, ..
            } => format!("call → {receiver_id}"),
            near_primitives::views::AccessKeyPermissionView::GasKeyFunctionCall {
                receiver_id,
                ..
            } => format!("gas → {receiver_id}"),
            near_primitives::views::AccessKeyPermissionView::GasKeyFullAccess { .. } => {
                "gas: full access".to_string()
            }
        }
    }

    fn permission_details(&self) -> String {
        match &self.permission {
            near_primitives::views::AccessKeyPermissionView::FullAccess => {
                "full access".to_string()
            }
            near_primitives::views::AccessKeyPermissionView::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            } => {
                let allowance_message = match allowance {
                    Some(amount) => format!("with a remaining fee allowance of {amount}"),
                    None => "with no limit".to_string(),
                };
                if method_names.is_empty() {
                    format!("call any function on {receiver_id} {allowance_message}")
                } else {
                    format!(
                        "call {method_names:?} function(s) on {receiver_id} {allowance_message}"
                    )
                }
            }
            near_primitives::views::AccessKeyPermissionView::GasKeyFunctionCall {
                balance,
                receiver_id,
                method_names,
                ..
            } => {
                let methods = if method_names.is_empty() {
                    "any methods".to_string()
                } else {
                    format!("{method_names:?}")
                };
                format!(
                    "gas key for function calls (on {receiver_id} {methods}, balance: {})",
                    balance.exact_amount_display()
                )
            }
            near_primitives::views::AccessKeyPermissionView::GasKeyFullAccess {
                balance,
                num_nonces,
            } => format!(
                "gas key with full access (balance: {}, nonces: {num_nonces})",
                balance.exact_amount_display()
            ),
        }
    }

    fn detailed_description(&self) -> String {
        let details = self.permission_details();
        match &self.permission {
            near_primitives::views::AccessKeyPermissionView::FullAccess => format!(
                "{} {}  {}",
                self.network_name.blue(),
                self.public_key.yellow(),
                details.yellow()
            ),
            near_primitives::views::AccessKeyPermissionView::FunctionCall { .. } => format!(
                "{} {}  {}",
                self.network_name.blue(),
                self.public_key.green(),
                details.green()
            ),
            near_primitives::views::AccessKeyPermissionView::GasKeyFunctionCall { .. }
            | near_primitives::views::AccessKeyPermissionView::GasKeyFullAccess { .. } => format!(
                "{} {}  {}",
                self.network_name.blue(),
                self.public_key.cyan(),
                details.cyan()
            ),
        }
    }
}

impl std::fmt::Display for AccessKeyInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = self.permission_summary();
        match &self.permission {
            near_primitives::views::AccessKeyPermissionView::FullAccess => write!(
                f,
                "{} {}  {}",
                self.network_name.blue(),
                self.public_key.yellow(),
                summary.yellow()
            ),
            near_primitives::views::AccessKeyPermissionView::FunctionCall { .. } => write!(
                f,
                "{} {}  {}",
                self.network_name.blue(),
                self.public_key.green(),
                summary.green()
            ),
            near_primitives::views::AccessKeyPermissionView::GasKeyFunctionCall { .. }
            | near_primitives::views::AccessKeyPermissionView::GasKeyFullAccess { .. } => write!(
                f,
                "{} {}  {}",
                self.network_name.blue(),
                self.public_key.cyan(),
                summary.cyan()
            ),
        }
    }
}

fn get_multi_select_render_config() -> RenderConfig<'static> {
    let mut render_config = crate::get_global_render_config();
    render_config.highlighted_option_prefix = Styled::new(">").with_fg(Color::DarkGreen);
    render_config.unhighlighted_option_prefix = Styled::new(" ").with_fg(Color::DarkGrey);
    render_config.scroll_up_prefix = Styled::new("↑").with_fg(Color::DarkGrey);
    render_config.scroll_down_prefix = Styled::new("↓").with_fg(Color::DarkGrey);
    render_config
}

#[cfg(test)]
mod tests {
    use super::AccessKeyInfo;

    fn function_call_key() -> AccessKeyInfo {
        AccessKeyInfo {
            public_key: "ed25519:DyZNbPPVPFjKobtEQxM8tNcdnA5E7WNsrpmNfkvqoyeV"
                .parse()
                .unwrap(),
            permission: near_primitives::views::AccessKeyPermissionView::FunctionCall {
                allowance: Some(near_token::NearToken::from_near(1)),
                receiver_id: "dev.everything.near".to_string(),
                method_names: vec!["__fastdata_kv".to_string()],
            },
            network_name: "mainnet".to_string(),
        }
    }

    #[test]
    fn picker_row_is_compact_and_contains_no_cursor_control_characters() {
        let picker_row = function_call_key().to_string();

        assert!(picker_row.contains("call → dev.everything.near"));
        assert!(!picker_row.contains("__fastdata_kv"));
        for control_character in ['\t', '\n', '\r'] {
            assert!(!picker_row.contains(control_character));
        }
    }

    #[test]
    fn submitted_answer_retains_permission_details() {
        let details = function_call_key().detailed_description();

        assert!(details.contains("__fastdata_kv"));
        assert!(details.contains("dev.everything.near"));
        assert!(details.contains("1.00 NEAR"));
        assert!(!details.contains('\t'));
    }
}
