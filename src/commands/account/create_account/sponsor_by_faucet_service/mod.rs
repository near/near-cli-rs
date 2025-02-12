use tracing_indicatif::span_ext::IndicatifSpanExt;

pub mod add_key;
pub mod network;

#[derive(Clone)]
pub struct SponsorServiceContext {
    pub config: crate::config::Config,
    pub new_account_id: crate::types::account_id::AccountId,
    pub public_key: near_crypto::PublicKey,
    pub on_after_getting_network_callback: self::network::OnAfterGettingNetworkCallback,
    pub on_before_creating_account_callback: self::network::OnBeforeCreatingAccountCallback,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = NewAccountContext)]
pub struct NewAccount {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the new account ID?
    new_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    access_key_mode: add_key::AccessKeyMode,
}

#[derive(Clone)]
pub struct NewAccountContext {
    pub config: crate::config::Config,
    pub new_account_id: crate::types::account_id::AccountId,
    pub on_before_creating_account_callback: self::network::OnBeforeCreatingAccountCallback,
}

impl NewAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<NewAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let credentials_home_dir = previous_context.config.credentials_home_dir.clone();
        let on_before_creating_account_callback: self::network::OnBeforeCreatingAccountCallback =
            std::sync::Arc::new({
                move |network_config, new_account_id, public_key, storage_message| {
                    before_creating_account(
                        network_config,
                        new_account_id,
                        public_key,
                        &credentials_home_dir,
                        storage_message,
                    )
                }
            });

        Ok(Self {
            config: previous_context.config,
            new_account_id: scope.new_account_id.clone(),
            on_before_creating_account_callback,
        })
    }
}

impl NewAccount {
    fn input_new_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        super::fund_myself_create_account::NewAccount::input_new_account_id(context)
    }
}

#[tracing::instrument(name = "Receiving request via faucet service", skip_all)]
pub fn before_creating_account(
    network_config: &crate::config::NetworkConfig,
    new_account_id: &crate::types::account_id::AccountId,
    public_key: &near_crypto::PublicKey,
    credentials_home_dir: &std::path::Path,
    storage_message: String,
) -> crate::CliResult {
    let faucet_service_url = match &network_config.faucet_url {
        Some(url) => url,
        None => return Err(color_eyre::Report::msg(format!(
            "The <{}> network does not have a faucet (helper service) that can sponsor the creation of an account.",
            &network_config.network_name
        )))
    };
    tracing::Span::current().pb_set_message(faucet_service_url.as_str());
    tracing::info!(target: "near_teach_me", "{}", faucet_service_url.as_str());
    let mut data = std::collections::HashMap::new();
    data.insert("newAccountId", new_account_id.to_string());
    data.insert("newAccountPublicKey", public_key.to_string());

    let client = reqwest::blocking::Client::new();
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to create an account"
    );
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "HTTP POST {}",
        faucet_service_url.as_str()
    );
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "JSON Body:\n{}",
        crate::common::indent_payload(&format!("{:#?}", data))
    );

    let result = client.post(faucet_service_url.clone()).json(&data).send();

    print_account_creation_status(
        result,
        network_config,
        new_account_id,
        credentials_home_dir,
        storage_message,
    )
}

fn print_account_creation_status(
    result: Result<reqwest::blocking::Response, reqwest::Error>,
    network_config: &crate::config::NetworkConfig,
    new_account_id: &crate::types::account_id::AccountId,
    credentials_home_dir: &std::path::Path,
    storage_message: String,
) -> crate::CliResult {
    // eprintln!();
    match result {
        Ok(response) => {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response:\n{}",
                crate::common::indent_payload(&format!("{:#?}", response))
            );
            if response.status() >= reqwest::StatusCode::BAD_REQUEST {
                tracing::warn!(
                    parent: &tracing::Span::none(),
                    "WARNING!{}",
                    crate::common::indent_payload(&format!(
                        "\nThe new account <{new_account_id}> could not be created successfully.\n{storage_message}\n"
                    ))
                );
                return Err(color_eyre::Report::msg(format!(
                    "The faucet (helper service) server failed with status code <{}>",
                    response.status()
                )));
            }

            let account_creation_transaction = response
                .json::<near_jsonrpc_client::methods::tx::RpcTransactionResponse>()?
                .final_execution_outcome
                .map(|outcome| outcome.into_outcome())
                .ok_or_else(|| {
                    color_eyre::Report::msg(
                        "The faucet (helper service) server did not return a transaction response.",
                    )
                })?;
            match account_creation_transaction.status {
                near_primitives::views::FinalExecutionStatus::SuccessValue(ref value) => {
                    if value == b"false" {
                        tracing::warn!(
                            parent: &tracing::Span::none(),
                            "WARNING!{}",
                            crate::common::indent_payload(&format!(
                                "\nThe new account <{new_account_id}> could not be created successfully.\n{storage_message}\n"
                            ))
                        );
                    } else {
                        crate::common::update_used_account_list_as_signer(
                            credentials_home_dir,
                            new_account_id.as_ref(),
                        );
                        tracing::info!(
                            parent: &tracing::Span::none(),
                            "{}",
                            crate::common::indent_payload(&format!(
                                "\nNew account <{new_account_id}> created successfully.\n{storage_message}\n"
                            ))
                        );
                    }
                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "\n{}",
                        crate::common::indent_payload(&format!(
                            "Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                            id=account_creation_transaction.transaction_outcome.id,
                            path=network_config.explorer_transaction_url
                        ))
                    );
                }
                near_primitives::views::FinalExecutionStatus::NotStarted
                | near_primitives::views::FinalExecutionStatus::Started => unreachable!(),
                near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                    tracing::warn!(
                        parent: &tracing::Span::none(),
                        "WARNING!{}",
                        crate::common::indent_payload(&format!(
                            "\nThe new account <{new_account_id}> could not be created successfully.\n{storage_message}\n"
                        ))
                    );
                    match tx_execution_error {
                        near_primitives::errors::TxExecutionError::ActionError(action_error) => {
                            return crate::common::convert_action_error_to_cli_result(
                                &action_error,
                            );
                        }
                        near_primitives::errors::TxExecutionError::InvalidTxError(
                            invalid_tx_error,
                        ) => {
                            return crate::common::convert_invalid_tx_error_to_cli_result(
                                &invalid_tx_error,
                            );
                        }
                    }
                }
            }
            Ok(())
        }
        Err(err) => {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response:\n{}",
                crate::common::indent_payload(&err.to_string())
            );
            tracing::warn!(
                parent: &tracing::Span::none(),
                "WARNING!{}",
                crate::common::indent_payload(&format!(
                    "\nThe new account <{new_account_id}> could not be created successfully.\n{storage_message}\n"
                ))
            );
            Err(color_eyre::Report::msg(err.to_string()))
        }
    }
}
