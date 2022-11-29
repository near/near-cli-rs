use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};

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
    fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        let mut new_account_id: crate::types::account_id::AccountId = Input::new()
            .with_prompt("What is the new account ID?")
            .interact_text()?;

        let choose_input = vec![
            format!(
                "Yes, I want to check that <{}> account does not exist. (It is free of charge, and only requires Internet access)",
                new_account_id
            ),
            "No, I know that this account does not exist and I want to proceed.".to_string(),
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nDo you want to check the existence of the specified account so that you don’t waste tokens with sending a transaction that won't succeed?")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())?;
        let account_id = if let Some(0) = select_choose_input {
            loop {
                let network =
                    find_network_where_account_exist(context, new_account_id.clone().into())?;
                if let Some(network_config) = network {
                    println!(
                        "\nHeads up! You will only waste tokens if you proceed creating <{}> account on <{}> as the account already exists.",
                        &new_account_id, network_config.network_name
                    );
                    if !ask_if_different_account_id_wanted()? {
                        break new_account_id;
                    };
                } else if new_account_id.0.as_str().chars().count()
                    < MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
                    && new_account_id.0.is_top_level()
                {
                    println!(
                        "\nAccount <{}> has <{}> character count. Only the registrar account can create new top level accounts that are shorter than {} characters. Read more about it in nomicon: https://nomicon.io/DataStructures/Account#top-level-accounts",
                        &new_account_id,
                        &new_account_id.0.as_str().chars().count(),
                        MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH,
                    );
                    if !ask_if_different_account_id_wanted()? {
                        break new_account_id;
                    };
                } else {
                    let parent_account_id = new_account_id
                        .clone()
                        .get_owner_account_id_from_sub_account();
                    if !near_primitives::types::AccountId::from(parent_account_id.clone())
                        .is_top_level()
                    {
                        if find_network_where_account_exist(
                            context,
                            parent_account_id.clone().into(),
                        )?
                        .is_none()
                        {
                            println!("\nThe parent account <{}> does not yet exist. Therefore, you cannot create an account <{}>.",
                            &parent_account_id, &new_account_id);
                            if !ask_if_different_account_id_wanted()? {
                                break new_account_id;
                            };
                        } else {
                            break new_account_id;
                        }
                    } else {
                        break new_account_id;
                    }
                };
                new_account_id = Input::new()
                    .with_prompt("What is the new account ID?")
                    .interact_text()?;
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
        let initial_balance: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter the amount of the NEAR tokens you want to fund the new account with (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0.1 NEAR")
            .interact_text()?;
        Ok(initial_balance)
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
) -> color_eyre::eyre::Result<Option<crate::config::NetworkConfig>> {
    for network in context.0.networks.iter() {
        loop {
            match tokio::runtime::Runtime::new().unwrap().block_on(
                crate::common::get_account_state(
                    network.1.clone(),
                    new_account_id.clone(),
                    near_primitives::types::Finality::Final.into(),
                ),
            ) {
                Ok(optional_account_view) => {
                    if optional_account_view.is_some() {
                        return Ok(Some(network.1.clone()));
                    } else {
                        return Ok(None); // невірна обробка. Після переробки get_account_state() переробити. Перейти до перевірки наступної мережі
                    }
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(_)) => {
                    println!("\nAddress information not found: A host or server name was specified, or the network connection <{}> is missing. So now there is no way to check if <{}> exists.",
                        network.1.network_name, new_account_id
                    );

                    let choose_input = vec![
                        "Yes, I want to check the account ID again.",
                        "No, I want to keep using this account ID.",
                    ];
                    let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to check the account ID again on this network?")
                        .items(&choose_input)
                        .default(0)
                        .interact_on_opt(&Term::stderr())?;
                    if matches!(select_choose_input, Some(1)) {
                        break;
                    }
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(
                    near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                        near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                            ..
                        },
                    ),
                )) => {
                    break;
                }
                Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(_)) => {
                    println!(
                        "Unable to verify the existence of account <{}> on network <{}>",
                        new_account_id, network.1.network_name
                    );
                    break;
                }
            }
        }
    }
    Ok(None)
}

fn ask_if_different_account_id_wanted() -> color_eyre::eyre::Result<bool> {
    let choose_input = vec![
        "Yes, I want to enter a new name for account ID.",
        "No, I want to keep using this name for account ID.",
    ];
    let select_choose_input = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to enter a different name for the new account ID?")
        .items(&choose_input)
        .default(0)
        .interact_on_opt(&Term::stderr())?;
    if matches!(select_choose_input, Some(1)) {
        return Ok(false);
    }
    Ok(true)
}
