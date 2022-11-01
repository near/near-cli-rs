use dialoguer::Input;
use serde_json::json;
use std::str::FromStr;

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
#[interactive_clap(context = crate::GlobalContext)]
pub struct NewAccount {
    ///What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    ///Enter the amount for the account
    initial_balance: crate::common::NearBalance,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

impl NewAccount {
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
#[interactive_clap(context = crate::GlobalContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SignerAccountId {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_from_cli_arg)]
    #[interactive_clap(skip_default_input_arg)]
    signer_account_id: Option<crate::types::account_id::AccountId>,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub fn from_cli(
        optional_clap_variant: Option<<SignerAccountId as interactive_clap::ToCli>::CliVariant>,
        context: crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<Self>> {
        let signer_account_id: Option<crate::types::account_id::AccountId> = optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.signer_account_id);
        let network_config = crate::network_for_transaction::NetworkForTransactionArgs::from_cli(
            optional_clap_variant.and_then(|clap_variant| {
                clap_variant.network_config.map(
                    |ClapNamedArgNetworkForTransactionArgsForSignerAccountId::NetworkConfig(
                        cli_network_config,
                    )| cli_network_config,
                )
            }),
            context,
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

    pub async fn process(
        &self,
        config: crate::config::Config,
        account_properties: AccountProperties,
    ) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        let mut new_account_id = account_properties
            .clone()
            .new_account_id
            .expect("Impossible to get account_id!");
        let account_id = loop {
            if (crate::common::get_account_state(
                network_config.clone(),
                new_account_id.clone().into(),
                near_primitives::types::Finality::Final.into(),
            )
            .await?)
                .is_some()
            {
                println!("Account <{}> already exists", new_account_id);
            } else {
                break new_account_id;
            }
            new_account_id = Input::new()
                .with_prompt("Enter a new account name")
                .interact_text()?;
        };

        let signer_id: near_primitives::types::AccountId = if self.signer_account_id.is_none() {
            let account_id_str = account_id.clone().to_string();
            let signer_account_id = if account_id_str.split('.').count() > 2 {
                account_id.clone().get_owner_account_id_from_sub_account()
            } else {
                Input::new()
                    .with_prompt("Enter a signer account name")
                    .interact_text()?
            };
            signer_account_id.into()
        } else {
            self.signer_account_id
                .clone()
                .expect("Impossible to get signer_account_id!")
                .into()
        };

        let args = json!({
            "new_account_id": account_id.clone().to_string(),
            "new_public_key": account_properties.public_key.to_string()
        })
        .to_string()
        .into_bytes();

        let linkdrop_account_id = network_config
            .clone()
            .linkdrop_account_id
            .expect("Impossible to get linkdrop_account_id!");
        let (actions, receiver_id) = if account_id.clone().0.is_sub_account_of(&signer_id) {
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
                account_id.clone().into(),
            )
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
            signer_id,
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
            Some(transaction_info) => {
                if !matches!(
                    transaction_info.status,
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_)
                ) {
                    return crate::common::print_transaction_status(
                        transaction_info,
                        self.network_config.get_network_config(config),
                    );
                }
                println!("New account <{}> created successfully.", account_id);

                if account_properties.storage.is_some() {
                    let new_account_properties = AccountProperties {
                        new_account_id: Some(account_id),
                        ..account_properties
                    };
                    let storage = account_properties
                        .storage
                        .expect("Impossible to get storage!");
                    match storage {
                        #[cfg(target_os = "macos")]
                        add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToMacosKeychain => {
                            add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_macos_keychain(
                                network_config,
                                new_account_properties,
                            )
                            .await?
                        }
                        add_key::autogenerate_new_keypair::SaveModeDiscriminants::SaveToKeychain => {
                            add_key::autogenerate_new_keypair::SaveMode::save_access_key_to_keychain(
                                config.clone(),
                                network_config,
                                new_account_properties,
                            )
                            .await?
                        }
                        add_key::autogenerate_new_keypair::SaveModeDiscriminants::PrintToTerminal => {
                            add_key::autogenerate_new_keypair::SaveMode::print_access_key_to_terminal(
                                new_account_properties,
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
            None => Ok(()),
        }
    }
}
