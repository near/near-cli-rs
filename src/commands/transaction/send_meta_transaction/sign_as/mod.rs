use color_eyre::owo_colors::OwoColorize;
use inquire::Select;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SignedMetaTransactionContext)]
#[interactive_clap(output_context = RelayerAccountIdContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct RelayerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the relayer account ID?
    relayer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct RelayerAccountIdContext(crate::commands::ActionContext);

impl RelayerAccountIdContext {
    pub fn from_previous_context(
        previous_context: super::SignedMetaTransactionContext,
        scope: &<RelayerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_id: near_primitives::types::AccountId =
                    scope.relayer_account_id.clone().into();
                let signed_delegate_action = previous_context.signed_delegate_action.clone();

                move |_network_config| {
                    let actions = vec![signed_delegate_action.clone().into()];

                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_id.clone(),
                        receiver_id: signed_delegate_action.delegate_action.sender_id.clone(),
                        actions,
                    })
                }
            });

        let on_before_signing_callback: crate::commands::OnBeforeSigningCallback =
            std::sync::Arc::new({
                move |prepopulated_unsigned_transaction, _network_config| {
                    prepopulated_unsigned_transaction.actions =
                        vec![near_primitives::transaction::Action::Delegate(Box::new(
                            previous_context.signed_delegate_action.clone(),
                        ))];
                    Ok(())
                }
            });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![scope.relayer_account_id.clone().into()],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome, _network_config| Ok(()),
            ),
            sign_as_delegate_action: false,
            on_sending_delegate_action_callback: None,
        }))
    }
}

impl From<RelayerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: RelayerAccountIdContext) -> Self {
        item.0
    }
}

impl RelayerAccountId {
    fn input_relayer_account_id(
        context: &super::SignedMetaTransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the relayer account ID?",
        )
    }
}

impl interactive_clap::FromCli for RelayerAccountId {
    type FromCliContext = super::SignedMetaTransactionContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    > {
        use ClapNamedArgNetworkForTransactionArgsForRelayerAccountId::NetworkConfig;
        use interactive_clap::ResultFromCli;
        let mut cli = optional_clap_variant.unwrap_or_default();
        // Explicit sign-as values and offline commands have always bypassed this prompt.
        let check_relayer = cli.relayer_account_id.is_none() && !context.global_context.offline;
        if cli.relayer_account_id.is_none() {
            cli.relayer_account_id = match Self::input_relayer_account_id(&context) {
                Ok(Some(account)) => Some(account),
                Ok(None) => return ResultFromCli::Cancel(Some(cli)),
                Err(err) => return ResultFromCli::Err(Some(cli), err),
            };
        }

        // Resolve the connection before checking the interactively entered account.
        let mut network_cli = match cli.network_config.take() {
            Some(NetworkConfig(network)) => network,
            None => Default::default(),
        };
        if network_cli.network_name.is_none() {
            network_cli.network_name = match crate::common::input_network_name(
                &context.global_context.config,
                &[cli
                    .relayer_account_id
                    .clone()
                    .expect("relayer is set")
                    .into()],
            ) {
                Ok(Some(name)) => Some(name),
                Ok(None) => {
                    cli.network_config = Some(NetworkConfig(network_cli.clone()));
                    return ResultFromCli::Cancel(Some(cli));
                }
                Err(err) => {
                    cli.network_config = Some(NetworkConfig(network_cli.clone()));
                    return ResultFromCli::Err(Some(cli), err);
                }
            };
        }
        cli.network_config = Some(NetworkConfig(network_cli.clone()));
        if check_relayer {
            let network_name = network_cli.network_name.as_ref().expect("network is set");
            let Some(network) = context
                .global_context
                .config
                .network_connection
                .get(network_name)
            else {
                return ResultFromCli::Err(
                    Some(cli),
                    color_eyre::eyre::eyre!("Failed to get network config!"),
                );
            };
            loop {
                let account = cli.relayer_account_id.clone().expect("relayer is set");
                match crate::common::is_account_exist_on_network(network, &account.clone().into()) {
                    Ok(true) => break,
                    Err(err) => return ResultFromCli::Err(Some(cli), err),
                    Ok(false) => {}
                }
                tracing::warn!(
                    "{}",
                    format!(
                        "The account <{account}> does not exist on network <{}>.",
                        network.network_name
                    )
                    .red()
                );
                let enter_another = match Select::new(
                    "Do you want to enter another relayer account id?",
                    vec![
                        "Yes, I want to enter a new account name.",
                        "No, I want to use this account name.",
                    ],
                )
                .prompt()
                {
                    Ok(choice) => choice.starts_with("Yes"),
                    Err(err) => return ResultFromCli::Err(Some(cli), err.into()),
                };
                if !enter_another {
                    break;
                }
                match Self::input_relayer_account_id(&context) {
                    // Update the reconstructed command immediately, even if a later step cancels.
                    Ok(Some(account)) => cli.relayer_account_id = Some(account),
                    Ok(None) => return ResultFromCli::Cancel(Some(cli)),
                    Err(err) => return ResultFromCli::Err(Some(cli), err),
                }
            }
        }
        let scope = InteractiveClapContextScopeForRelayerAccountId {
            relayer_account_id: cli.relayer_account_id.clone().expect("relayer is set"),
        };
        let action_context = match RelayerAccountIdContext::from_previous_context(context, &scope) {
            Ok(context) => context,
            Err(err) => return ResultFromCli::Err(Some(cli), err),
        };
        match crate::network_for_transaction::NetworkForTransactionArgs::from_cli(
            Some(network_cli),
            action_context.into(),
        ) {
            ResultFromCli::Ok(network) => {
                cli.network_config = Some(NetworkConfig(network));
                ResultFromCli::Ok(cli)
            }
            ResultFromCli::Cancel(network) => {
                cli.network_config = network.map(NetworkConfig);
                ResultFromCli::Cancel(Some(cli))
            }
            ResultFromCli::Back => ResultFromCli::Back,
            ResultFromCli::Err(network, err) => {
                cli.network_config = network.map(NetworkConfig);
                ResultFromCli::Err(Some(cli), err)
            }
        }
    }
}
