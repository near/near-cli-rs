use dialoguer::{console::Term, theme::ColorfulTheme, Input, Select};
use serde_json::json;
use std::str::FromStr;

use crate::commands::account::MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH;

mod add_key;

#[derive(Debug, Clone, Default)]
pub struct AccountProperties {
    pub new_account_id: Option<crate::types::account_id::AccountId>,
    pub public_key: crate::types::public_key::PublicKey,
    pub initial_balance: crate::common::NearBalance,
    pub key_pair_properties: Option<crate::common::KeyPairProperties>,
    pub storage: Option<self::add_key::autogenerate_new_keypair::SaveModeDiscriminants>,
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
            "Yes, I want to check the existence of such an account.",
            "No, I want to use this name for account_id.",
        ];
        let select_choose_input = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("\nDo you want to check the existence of such an account so that you don’t send a transaction and don’t waste tokens in vain?")
            .items(&choose_input)
            .default(0)
            .interact_on_opt(&Term::stderr())?;
        match select_choose_input {
            Some(0) => loop {
                let mut network_config = context.0.networks.iter().next().unwrap().1;

                let optional_account_view = 'block: {
                    for network in context.0.networks.iter() {
                        let optional_account_view = tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(crate::common::get_account_state(
                                network.1.clone(),
                                new_account_id.clone().into(),
                                near_primitives::types::Finality::Final.into(),
                            ))?;
                        if optional_account_view.is_some() {
                            network_config = network.1;
                            break 'block optional_account_view;
                        }
                    }
                    None
                };
                if optional_account_view.is_some() {
                    println!(
                        "\nAccount <{}> already exists in network <{}>.",
                        &new_account_id, network_config.network_name
                    );
                    let choose_input = vec![
                        "Yes, I want to enter a new name for account_id.",
                        "No, I want to use this name for account_id.",
                    ];
                    let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to enter a new name for account_id?")
                        .items(&choose_input)
                        .default(0)
                        .interact_on_opt(&Term::stderr())?;
                    if matches!(select_choose_input, Some(1)) {
                        break Ok(new_account_id);
                    }
                } else if new_account_id.to_string().chars().count()
                    < MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
                    && !new_account_id.to_string().contains('.')
                {
                    println!(
                    "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
                    &new_account_id, &new_account_id.to_string().chars().count()
                );
                    let choose_input = vec![
                        "Yes, I want to enter a new name for account_id.",
                        "No, I want to use this name for account_id.",
                    ];
                    let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                        .with_prompt("Do you want to enter a new name for account_id?")
                        .items(&choose_input)
                        .default(0)
                        .interact_on_opt(&Term::stderr())?;
                    if matches!(select_choose_input, Some(1)) {
                        break Ok(new_account_id);
                    }
                } else {
                    break Ok(new_account_id);
                }
                new_account_id = Input::new()
                    .with_prompt("What is the new account ID?")
                    .interact_text()?;
            },
            Some(1) => Ok(new_account_id),
            _ => unreachable!("Error"),
        }
    }

    fn input_initial_balance(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        let initial_balance: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0.1 NEAR")
            .interact_text()?;
        Ok(initial_balance)
    }

    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let account_properties = AccountProperties {
            new_account_id: Some(self.new_account_id.clone()),
            initial_balance: self.initial_balance.clone(),
            ..Default::default()
        };
        self.access_key_mode
            .process(config, account_properties)
            .await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::commands::account::create_account::CreateAccountContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignerAccountId {
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    ///What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub fn from_cli(
        optional_clap_variant: Option<<SignerAccountId as interactive_clap::ToCli>::CliVariant>,
        context: crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let signer_account_id: crate::types::account_id::AccountId = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.signer_account_id)
        {
            Some(cli_signer_account_id) => cli_signer_account_id,
            None => {
                let owner_account_id = context
                    .new_account_id
                    .clone()
                    .get_owner_account_id_from_sub_account();
                if !owner_account_id.0.is_top_level() {
                    owner_account_id
                } else {
                    Self::input_signer_account_id(&context)?
                }
            }
        };
        let network_config = crate::network_for_transaction::NetworkForTransactionArgs::from_cli(
            optional_clap_variant.and_then(|clap_variant| {
                clap_variant.network_config.map(
                    |ClapNamedArgNetworkForTransactionArgsForSignerAccountId::NetworkConfig(
                        cli_network_config,
                    )| cli_network_config,
                )
            }),
            (context.config,),
        )?;
        let network_config = if let Some(value) = network_config {
            value
        } else {
            return Ok(None);
        };
        Ok(Some(Self {
            signer_account_id,
            network_config,
        }))
    }

    fn input_signer_account_id(
        context: &crate::commands::account::create_account::CreateAccountContext,
    ) -> color_eyre::eyre::Result<crate::types::account_id::AccountId> {
        loop {
            let signer_account_id: crate::types::account_id::AccountId = Input::new()
                .with_prompt("What is the signer account ID?")
                .interact_text()?;
            let top_level_new_account_id_string = context.new_account_id.to_string();
            let top_level_new_account_id_str = top_level_new_account_id_string
                .rsplit_once('.')
                .map_or("mainnet", |s| if s.1 == "near" { "mainnet" } else { s.1 });
            let network_config = context
                .config
                .networks
                .get(top_level_new_account_id_str)
                .expect("Impossible to get network config!");
            let optional_account_view = tokio::runtime::Runtime::new().unwrap().block_on(
                crate::common::get_account_state(
                    network_config.clone(),
                    signer_account_id.clone().into(),
                    near_primitives::types::Finality::Final.into(),
                ),
            )?;
            if optional_account_view.is_none() {
                println!(
                    "\nThe account <{}> does not yet exist on the network <{}>.",
                    &signer_account_id, network_config.network_name
                );
                let choose_input = vec![
                    "Yes, I want to enter a new name for signer_account_id.",
                    "No, I want to use this name for signer_account_id.",
                ];
                let select_choose_input = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Do you want to enter a new name for signer_account_id?")
                    .items(&choose_input)
                    .default(0)
                    .interact_on_opt(&Term::stderr())?;
                if matches!(select_choose_input, Some(1)) {
                    break Ok(signer_account_id);
                }
            } else {
                break Ok(signer_account_id);
            }
        }
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: AccountProperties,
    ) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());

        if crate::common::get_account_state(
            network_config.clone(),
            self.signer_account_id.clone().into(),
            near_primitives::types::Finality::Final.into(),
        )
        .await?
        .is_none()
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nSigner account <{}> does not yet exist on the network <{}>.",
                self.signer_account_id,
                network_config.network_name
            ));
        };

        let new_account_id = account_properties
            .clone()
            .new_account_id
            .expect("Impossible to get account_id!");
        if new_account_id.to_string().chars().count() < MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH
            && !new_account_id.to_string().contains('.')
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount <{}> has <{}> character count. Only REGISTRAR_ACCOUNT_ID account can create new top level accounts that are shorter than MIN_ALLOWED_TOP_LEVEL_ACCOUNT_LENGTH (32) characters.",
                &new_account_id, &new_account_id.to_string().chars().count()
            ));
        }
        if crate::common::get_account_state(
            network_config.clone(),
            new_account_id.clone().into(),
            near_primitives::types::Finality::Final.into(),
        )
        .await?
        .is_some()
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount <{}> already exists in network <{}>.",
                &new_account_id,
                network_config.network_name
            ));
        };

        let args = json!({
            "new_account_id": new_account_id.clone().to_string(),
            "new_public_key": account_properties.public_key.to_string()
        })
        .to_string()
        .into_bytes();

        let linkdrop_account_id = network_config
            .clone()
            .linkdrop_account_id
            .expect("Impossible to get linkdrop_account_id!");

        let (actions, receiver_id) = if new_account_id
            .clone()
            .0
            .is_sub_account_of(&self.signer_account_id.0)
        {
            (
                vec![
                    near_primitives::transaction::Action::CreateAccount(
                        near_primitives::transaction::CreateAccountAction {},
                    ),
                    near_primitives::transaction::Action::Transfer(
                        near_primitives::transaction::TransferAction {
                            deposit: account_properties.initial_balance.to_yoctonear(),
                        },
                    ),
                    near_primitives::transaction::Action::AddKey(
                        near_primitives::transaction::AddKeyAction {
                            public_key: near_crypto::PublicKey::from_str(
                                &account_properties.public_key.to_string(),
                            )?,
                            access_key: near_primitives::account::AccessKey {
                                nonce: 0,
                                permission:
                                    near_primitives::account::AccessKeyPermission::FullAccess,
                            },
                        },
                    ),
                ],
                new_account_id.clone().into(),
            )
        } else if !new_account_id
            .clone()
            .0
            .is_sub_account_of(&self.signer_account_id.0)
            && new_account_id.to_string().split('.').count() > 2
        {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nSigner account <{}> does not have permission to create account <{}>.",
                self.signer_account_id,
                new_account_id
            ));
        } else {
            (
                vec![near_primitives::transaction::Action::FunctionCall(
                    near_primitives::transaction::FunctionCallAction {
                        method_name: "create_account".to_string(),
                        args,
                        gas: crate::common::NearGas::from_str("30 TeraGas")
                            .unwrap()
                            .inner,
                        deposit: account_properties.initial_balance.to_yoctonear(),
                    },
                )],
                linkdrop_account_id.clone(),
            )
        };

        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.0.clone(),
            public_key: account_properties.clone().public_key.into(),
            nonce: 0,
            receiver_id,
            block_hash: Default::default(),
            actions,
        };
        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => match transaction_info.status {
                near_primitives::views::FinalExecutionStatus::SuccessValue(ref value) => {
                    if value == b"false" {
                        println!(
                            "The new account <{}> could not be created successfully.",
                            new_account_id
                        );
                    } else {
                        println!("New account <{}> created successfully.", new_account_id);

                        let storage = account_properties
                            .storage
                            .expect("Impossible to get storage!");
                        match storage {
                            #[cfg(target_os = "macos")]
                            add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToMacosKeychain => {
                                add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_macos_keychain(
                                    network_config,
                                    account_properties,
                                )
                                .await?
                            }
                            add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToKeychain => {
                                add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_keychain(
                                    config.clone(),
                                    network_config,
                                    account_properties,
                                )
                                .await?
                            }
                            add_key::autogenerate_new_keypair::SaveModeDiscriminants::PrintToTerminal => {
                                add_key::autogenerate_new_keypair::SaveMode::print_access_key_to_terminal(
                                    account_properties,
                                )
                            }
                        }
                    }
                    println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                                id=transaction_info.transaction_outcome.id,
                                path=self.network_config.get_network_config(config).explorer_transaction_url
                            );
                    Ok(())
                }
                _ => crate::common::print_transaction_status(
                    transaction_info,
                    self.network_config.get_network_config(config),
                ),
            },
            None => Ok(()),
        }
    }
}
