use inquire::ui::{Color, RenderConfig, Styled};
use inquire::{CustomType, MultiSelect, formatter::MultiOptionFormatter};

use crate::common::AccessKeyInfo;

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
    owner_account_id: near_kit::AccountId,
    public_keys: Vec<near_kit::PublicKey>,
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
                                near_kit::Action::DeleteKey(
                                    near_kit::DeleteKeyAction {
                                        public_key: crate::types::public_key::PublicKey::from(public_key).0,
                                    },
                                )
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

        let (access_key_list, errors) = crate::common::get_public_keys_from_network_config(
            &context.global_context,
            &context.owner_account_id,
        )?;

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
                .map(|list_option| list_option.value.to_string())
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

fn get_multi_select_render_config() -> RenderConfig<'static> {
    let mut render_config = crate::get_global_render_config();
    render_config.highlighted_option_prefix = Styled::new(">").with_fg(Color::DarkGreen);
    render_config.unhighlighted_option_prefix = Styled::new(" ").with_fg(Color::DarkGrey);
    render_config.scroll_up_prefix = Styled::new("↑").with_fg(Color::DarkGrey);
    render_config.scroll_down_prefix = Styled::new("↓").with_fg(Color::DarkGrey);
    render_config
}
