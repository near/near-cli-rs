mod receiver_validation;

use receiver_validation::ReceiverValidation;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = SendNearCommandContext)]
#[interactive_clap(skip_default_from_cli)]
pub struct SendNearCommand {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    /// How many NEAR Tokens do you want to transfer? (example: 10 NEAR or 0.5 NEAR or 10000 yoctonear)
    amount_in_near: crate::types::near_token::NearToken,
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    /// Receiver validation: check or skip. Defaults to the existing quiet/offline behavior.
    receiver_validation: Option<ReceiverValidation>,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SendNearCommandContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    amount_in_near: crate::types::near_token::NearToken,
    receiver_validation: Option<ReceiverValidation>,
    interactive: bool,
}

impl SendNearCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendNearCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            amount_in_near: scope.amount_in_near,
            receiver_validation: scope.receiver_validation,
            interactive: false,
        })
    }
}

impl From<SendNearCommandContext> for crate::commands::ActionContext {
    fn from(item: SendNearCommandContext) -> Self {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = item.signer_account_id.clone();
                let receiver_account_id = item.receiver_account_id.clone();
                let verbosity = item.global_context.verbosity;

                move |network_config| {
                    if !receiver_validation::validate(
                        network_config,
                        &receiver_account_id,
                        verbosity,
                        item.global_context.offline,
                        item.receiver_validation,
                        item.interactive,
                    )? {
                        return Ok(crate::commands::PrepopulatedTransaction {
                            signer_id: signer_account_id.clone(),
                            receiver_id: receiver_account_id.clone(),
                            actions: vec![],
                        });
                    }
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: receiver_account_id.clone(),
                        actions: vec![near_primitives::transaction::Action::Transfer(
                            near_primitives::transaction::TransferAction {
                                deposit: item.amount_in_near.into(),
                            },
                        )],
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![item.signer_account_id, item.receiver_account_id],
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

impl SendNearCommand {
    pub fn input_receiver_account_id(
        context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }
}

impl interactive_clap::FromCli for SendNearCommand {
    type FromCliContext = super::TokensCommandsContext;
    type FromCliError = color_eyre::eyre::Error;

    fn from_cli(
        optional_clap_variant: Option<<Self as interactive_clap::ToCli>::CliVariant>,
        context: Self::FromCliContext,
    ) -> interactive_clap::ResultFromCli<
        <Self as interactive_clap::ToCli>::CliVariant,
        Self::FromCliError,
    > {
        use interactive_clap::ResultFromCli;
        let mut cli = optional_clap_variant.unwrap_or_default();
        // Only offer a new dialog when the command still needs interactive input.
        let interactive = needs_interactive_input(&cli)
            && !matches!(context.global_context.verbosity, crate::Verbosity::Quiet);
        if cli.receiver_account_id.is_none() {
            cli.receiver_account_id = match Self::input_receiver_account_id(&context) {
                Ok(Some(value)) => Some(value),
                Ok(None) => return ResultFromCli::Cancel(Some(cli)),
                Err(err) => return ResultFromCli::Err(Some(cli), err),
            };
        }
        if cli.amount_in_near.is_none() {
            cli.amount_in_near = match Self::input_amount_in_near(&context) {
                Ok(Some(value)) => Some(value),
                Ok(None) => return ResultFromCli::Cancel(Some(cli)),
                Err(err) => return ResultFromCli::Err(Some(cli), err),
            };
        }
        if interactive && !context.global_context.offline && cli.receiver_validation.is_none() {
            cli.receiver_validation = match receiver_validation::input() {
                Ok(Some(value)) => Some(value),
                Ok(None) => return ResultFromCli::Cancel(Some(cli)),
                Err(err) => return ResultFromCli::Err(Some(cli), err),
            };
        }
        let scope = InteractiveClapContextScopeForSendNearCommand {
            receiver_account_id: cli
                .receiver_account_id
                .clone()
                .expect("receiver was entered"),
            amount_in_near: cli.amount_in_near.expect("amount was entered"),
            receiver_validation: cli.receiver_validation,
        };
        let mut new_context = match SendNearCommandContext::from_previous_context(context, &scope) {
            Ok(value) => value,
            Err(err) => return ResultFromCli::Err(Some(cli), err),
        };
        new_context.interactive = interactive;
        use ClapNamedArgNetworkForTransactionArgsForSendNearCommand::NetworkConfig;
        let network = cli.network_config.take().map(|NetworkConfig(value)| value);
        match <crate::network_for_transaction::NetworkForTransactionArgs as interactive_clap::FromCli>::from_cli(network, new_context.into()) {
            ResultFromCli::Ok(value) => {
                cli.network_config = Some(NetworkConfig(value));
                ResultFromCli::Ok(cli)
            }
            ResultFromCli::Cancel(value) => {
                cli.network_config = value.map(NetworkConfig);
                ResultFromCli::Cancel(Some(cli))
            }
            ResultFromCli::Back => ResultFromCli::Back,
            ResultFromCli::Err(value, err) => {
                cli.network_config = value.map(NetworkConfig);
                ResultFromCli::Err(Some(cli), err)
            }
        }
    }
}

fn needs_interactive_input(cli: &CliSendNearCommand) -> bool {
    use clap::CommandFactory;
    use interactive_clap::ToCliArgs;
    let mut command = CliSendNearCommand::command();
    let args = std::iter::once("send-near".to_owned()).chain(cli.to_cli_args());
    match command.try_get_matches_from_mut(args) {
        Ok(matches) => command_needs_interactive_input(&command, &matches),
        Err(_) => true,
    }
}

fn command_needs_interactive_input(command: &clap::Command, matches: &clap::ArgMatches) -> bool {
    // interactive-clap makes positional arguments and subcommands optional so it
    // can prompt for missing values. Follow the entire selected command chain,
    // including signer credentials and submit/output arguments.
    let missing_argument = command.get_arguments().any(|arg| {
        // These long options also always prompt when omitted. Other signing
        // options (nonce, block hash, etc.) resolve automatically when online.
        let prompts = arg.is_positional()
            || arg.get_long() == Some("seed-phrase-hd-path")
            || (command.get_name() == "sign-later"
                && matches!(
                    arg.get_long(),
                    Some("signer-public-key" | "nonce" | "block-hash")
                ));
        prompts && !matches.contains_id(arg.get_id().as_str())
    });
    missing_argument
        || match matches.subcommand() {
            Some((name, submatches)) => command_needs_interactive_input(
                command
                    .find_subcommand(name)
                    .expect("parsed subcommand exists"),
                submatches,
            ),
            None => command.get_subcommands().next().is_some(),
        }
}

#[cfg(test)]
mod tests;
