use std::str::FromStr;

use inquire::{CustomType, Select, Text};

use crate::commands::account::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH;

mod add_key;
mod sign_as;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = NewAccountContext)]
pub struct NewAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter the amount for the account:
    initial_balance: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct NewAccountContext {
    config: crate::config::Config,
    offline: bool,
    new_account_id: crate::types::account_id::AccountId,
    initial_balance: crate::common::NearBalance,
}

impl NewAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<NewAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            offline: previous_context.offline,
            new_account_id: scope.new_account_id.clone(),
            initial_balance: scope.initial_balance.clone(),
        })
    }
}

impl NewAccount {
    pub fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let new_account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the new account ID?").prompt()?;

        if context.offline {
            return Ok(Some(new_account_id));
        }

        #[derive(derive_more::Display)]
        enum ConfirmOptions {
            #[display(
                fmt = "Yes, I want to check that <{}> account does not exist. (It is free of charge, and only requires Internet access)",
                account_id
            )]
            Yes {
                account_id: crate::types::account_id::AccountId,
            },
            #[display(fmt = "No, I know that this account does not exist and I want to proceed.")]
            No,
        }
        let select_choose_input =
            Select::new("\nDo you want to check the existence of the specified account so that you don’t waste tokens with sending a transaction that won't succeed?",
                vec![ConfirmOptions::Yes{account_id: new_account_id.clone()}, ConfirmOptions::No],
                )
                .prompt()?;
        let account_id = if let ConfirmOptions::Yes { mut account_id } = select_choose_input {
            loop {
                let network = crate::common::find_network_where_account_exist(
                    context,
                    account_id.clone().into(),
                );
                if let Some(network_config) = network {
                    eprintln!(
                        "\nHeads up! You will only waste tokens if you proceed creating <{}> account on <{}> as the account already exists.",
                        &account_id, network_config.network_name
                    );
                    if !crate::common::ask_if_different_account_id_wanted()? {
                        break account_id;
                    };
                } else if account_id.0.as_str().chars().count()
                    < MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
                    && account_id.0.is_top_level()
                {
                    eprintln!(
                        "\nAccount <{}> has <{}> character count. Only the registrar account can create new top level accounts that are shorter than {} characters. Read more about it in nomicon: https://nomicon.io/DataStructures/Account#top-level-accounts",
                        &account_id,
                        &account_id.0.as_str().chars().count(),
                        MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH,
                    );
                    if !crate::common::ask_if_different_account_id_wanted()? {
                        break account_id;
                    };
                } else {
                    let parent_account_id =
                        account_id.clone().get_parent_account_id_from_sub_account();
                    if !near_primitives::types::AccountId::from(parent_account_id.clone())
                        .is_top_level()
                    {
                        if crate::common::find_network_where_account_exist(
                            context,
                            parent_account_id.clone().into(),
                        )
                        .is_none()
                        {
                            eprintln!("\nThe parent account <{}> does not yet exist. Therefore, you cannot create an account <{}>.",
                                &parent_account_id, &account_id);
                            if !crate::common::ask_if_different_account_id_wanted()? {
                                break account_id;
                            };
                        } else {
                            break account_id;
                        }
                    } else {
                        break account_id;
                    }
                };
                account_id = CustomType::new("What is the new account ID?").prompt()?;
            }
        } else {
            new_account_id
        };
        Ok(Some(account_id))
    }

    fn input_initial_balance(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        eprintln!();
        match crate::common::NearBalance::from_str(&Text::new("Enter the amount of the NEAR tokens you want to fund the new account with (example: 10NEAR or 0.5near or 10000yoctonear):")
            .with_initial_value("0.1 NEAR")
            .prompt()?
            ) {
                Ok(initial_balance) => Ok(Some(initial_balance)),
                Err(err) => Err(color_eyre::Report::msg(
                    err,
                ))
            }
    }
}

#[derive(Clone)]
pub struct AccountPropertiesContext {
    pub config: crate::config::Config,
    pub offline: bool,
    pub account_properties: AccountProperties,
    pub on_before_sending_transaction_callback:
        crate::transaction_signature_options::OnBeforeSendingTransactionCallback,
}

#[derive(Debug, Clone)]
pub struct AccountProperties {
    pub new_account_id: crate::types::account_id::AccountId,
    pub public_key: near_crypto::PublicKey,
    pub initial_balance: crate::common::NearBalance,
}
