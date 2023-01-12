use inquire::{CustomType, Select, Text};
use std::str::FromStr;

use crate::commands::account::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH;

mod add_key;
mod sign_as;

#[derive(Debug, Clone)]
pub struct AccountProperties {
    pub new_account_id: near_primitives::types::AccountId,
    pub public_key: near_crypto::PublicKey,
    pub initial_balance: crate::common::NearBalance,
}

#[derive(Debug, Clone)]
pub struct StorageProperties {
    pub key_pair_properties: crate::common::KeyPairProperties,
    pub storage: self::add_key::autogenerate_new_keypair::SaveModeDiscriminants,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = CreateAccountContext)]
pub struct NewAccount {
    #[interactive_clap(skip_default_input_arg)]
    ///What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    ///Enter the amount for the account
    initial_balance: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

#[derive(Debug, Clone)]
struct NewAccountContext {
    config: crate::config::Config,
    new_account_id: crate::types::account_id::AccountId,
}

impl NewAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<NewAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> Self {
        Self {
            config: previous_context.0,
            new_account_id: scope.new_account_id.clone(),
        }
    }
}

impl From<NewAccountContext> for crate::commands::account::create_account::CreateAccountContext {
    fn from(item: NewAccountContext) -> Self {
        Self {
            config: item.config,
            new_account_id: item.new_account_id,
        }
    }
}

impl NewAccount {
    pub fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let new_account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the new account ID?").prompt()?;

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
                let network = find_network_where_account_exist(context, account_id.clone().into());
                if let Some(network_config) = network {
                    println!(
                        "\nHeads up! You will only waste tokens if you proceed creating <{}> account on <{}> as the account already exists.",
                        &account_id, network_config.network_name
                    );
                    if !ask_if_different_account_id_wanted()? {
                        break account_id;
                    };
                } else if account_id.0.as_str().chars().count()
                    < MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
                    && account_id.0.is_top_level()
                {
                    println!(
                        "\nAccount <{}> has <{}> character count. Only the registrar account can create new top level accounts that are shorter than {} characters. Read more about it in nomicon: https://nomicon.io/DataStructures/Account#top-level-accounts",
                        &account_id,
                        &account_id.0.as_str().chars().count(),
                        MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH,
                    );
                    if !ask_if_different_account_id_wanted()? {
                        break account_id;
                    };
                } else {
                    let parent_account_id =
                        account_id.clone().get_parent_account_id_from_sub_account();
                    if !near_primitives::types::AccountId::from(parent_account_id.clone())
                        .is_top_level()
                    {
                        if find_network_where_account_exist(
                            context,
                            parent_account_id.clone().into(),
                        )
                        .is_none()
                        {
                            println!("\nThe parent account <{}> does not yet exist. Therefore, you cannot create an account <{}>.",
                            &parent_account_id, &account_id);
                            if !ask_if_different_account_id_wanted()? {
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
        Ok(account_id)
    }

    fn input_initial_balance(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        match crate::common::NearBalance::from_str(&Text::new("Enter the amount of the NEAR tokens you want to fund the new account with (example: 10NEAR or 0.5near or 10000yoctonear).")
            .with_initial_value("0.1 NEAR")
            .prompt()?
            ) {
                Ok(initial_balance) => Ok(initial_balance),
                Err(err) => Err(color_eyre::Report::msg(
                    err,
                ))
            }
    }

    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let account_properties = AccountProperties {
            new_account_id: self.new_account_id.clone().into(),
            initial_balance: self.initial_balance.clone(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
        };
        self.access_key_mode
            .process(config, account_properties)
            .await
    }
}

fn find_network_where_account_exist(
    context: &crate::GlobalContext,
    new_account_id: near_primitives::types::AccountId,
) -> Option<crate::config::NetworkConfig> {
    for network in context.0.networks.iter() {
        if tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(crate::common::get_account_state(
                network.1.clone(),
                new_account_id.clone(),
                near_primitives::types::Finality::Final.into(),
            ))
            .is_ok()
        {
            return Some(network.1.clone());
        }
    }
    None
}

fn ask_if_different_account_id_wanted() -> color_eyre::eyre::Result<bool> {
    #[derive(strum_macros::Display, PartialEq)]
    enum ConfirmOptions {
        #[strum(to_string = "Yes, I want to enter a new name for account ID.")]
        Yes,
        #[strum(to_string = "No, I want to keep using this name for account ID.")]
        No,
    }
    let select_choose_input = Select::new(
        "Do you want to enter a different name for the new account ID?",
        vec![ConfirmOptions::Yes, ConfirmOptions::No],
    )
    .prompt()?;
    Ok(select_choose_input == ConfirmOptions::Yes)
}
