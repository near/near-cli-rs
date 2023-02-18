#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::GenerateKeypairContext)]
#[interactive_clap(output_context = crate::commands::ActionContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SaveKeypairToKeychain {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SaveKeypairToKeychainContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
    key_pair_properties: crate::common::KeyPairProperties,
    public_key: near_crypto::PublicKey,
}

impl SaveKeypairToKeychainContext {
    pub fn from_previous_context(
        previous_context: super::GenerateKeypairContext,
        _scope: &<SaveKeypairToKeychain as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.signer_account_id,
            permission: previous_context.permission,
            key_pair_properties: previous_context.key_pair_properties,
            public_key: previous_context.public_key,
        })
    }
}

impl From<SaveKeypairToKeychainContext> for crate::commands::ActionContext {
    fn from(item: SaveKeypairToKeychainContext) -> Self {
        Self {
            config: item.config,
            signer_account_id: item.signer_account_id.clone(),
            receiver_account_id: item.signer_account_id,
            actions: vec![near_primitives::transaction::Action::AddKey(
                near_primitives::transaction::AddKeyAction {
                    public_key: item.public_key,
                    access_key: near_primitives::account::AccessKey {
                        nonce: 0,
                        permission: item.permission,
                    },
                },
            )],
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_after_getting_network_callback: std::sync::Arc::new(|_actions, _network_config| {
                Ok(())
            }),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

impl interactive_clap::FromCli for SaveKeypairToKeychain {
    type FromCliContext = super::GenerateKeypairContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<
            <SaveKeypairToKeychain as interactive_clap::ToCli>::CliVariant,
        >,
        context: Self::FromCliContext,
    ) -> Result<Option<Self>, Self::FromCliError>
    where
        Self: Sized + interactive_clap::ToCli,
    {
        let new_context_scope = InteractiveClapContextScopeForSaveKeypairToKeychain {};
        let save_to_keychain_context = SaveKeypairToKeychainContext::from_previous_context(
            context.clone(),
            &new_context_scope,
        )?;
        let new_context = crate::commands::ActionContext::from(save_to_keychain_context);

        let optional_network = crate::network_for_transaction::NetworkForTransactionArgs::from_cli(
            optional_clap_variant.and_then(|clap_variant| match clap_variant.network_config {
                Some(
                    ClapNamedArgNetworkForTransactionArgsForSaveKeypairToKeychain::NetworkConfig(
                        cli_arg,
                    ),
                ) => Some(cli_arg),
                None => None,
            }),
            new_context.clone().into(),
        )?;
        let network_config = if let Some(network) = optional_network {
            network
        } else {
            return Ok(None);
        };

        let key_pair_properties_buf = serde_json::to_string(&context.key_pair_properties)?;
        crate::common::save_access_key_to_keychain(
            network_config.get_network_config(context.config.clone()),
            context.config.credentials_home_dir.clone(),
            &key_pair_properties_buf,
            &context.key_pair_properties.public_key_str,
            &new_context.receiver_account_id,
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        })?;

        Ok(Some(Self { network_config }))
    }
}

impl SaveKeypairToKeychain {
    pub async fn process(
        &self,
        config: crate::config::Config,
        key_pair_properties: crate::common::KeyPairProperties,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        let network_config = self.network_config.get_network_config(config.clone());
        // let key_pair_properties_buf = serde_json::to_string(&key_pair_properties)?;
        // crate::common::save_access_key_to_keychain(
        //     network_config,
        //     config.credentials_home_dir.clone(),
        //     &key_pair_properties_buf,
        //     &key_pair_properties.public_key_str,
        //     &prepopulated_unsigned_transaction.receiver_id,
        // )
        // .map_err(|err| {
        //     color_eyre::Report::msg(format!("Failed to save a file with access key: {}", err))
        // })?;
        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => crate::common::print_transaction_status(
                transaction_info,
                self.network_config.get_network_config(config),
            ),
            None => Ok(()),
        }
    }
}
