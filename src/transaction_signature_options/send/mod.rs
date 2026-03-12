use color_eyre::owo_colors::OwoColorize;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SubmitContext)]
#[interactive_clap(output_context = SendContext)]
pub struct Send {
    /// Wait until the transaction reaches a specific execution status before returning (default: final)
    #[interactive_clap(long)]
    #[interactive_clap(skip_interactive_input)]
    wait_until: Option<crate::types::tx_execution_status::TxExecutionStatus>,
}

#[derive(Debug, Clone)]
pub struct SendContext;

impl SendContext {
    #[tracing::instrument(name = "Sending transaction ...", skip_all)]
    pub fn from_previous_context(
        previous_context: super::SubmitContext,
        scope: &<Send as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        tracing::info!(target: "near_teach_me", "Sending transaction ...");

        let wait_until: near_primitives::views::TxExecutionStatus = scope
            .wait_until
            .clone()
            .or_else(|| previous_context.network_config.tx_wait_until.clone())
            .map(|s| s.into())
            .unwrap_or(near_primitives::views::TxExecutionStatus::Final);

        let storage_message = (previous_context.on_before_sending_transaction_callback)(
            &previous_context.signed_transaction_or_signed_delegate_action,
            &previous_context.network_config,
        )
        .map_err(color_eyre::Report::msg)?;

        match previous_context.signed_transaction_or_signed_delegate_action {
            super::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                signed_transaction,
            ) => {
                match sending_signed_transaction(
                    &previous_context.network_config,
                    &signed_transaction,
                    wait_until.clone(),
                )? {
                    Some(transaction_info) => {
                        crate::common::print_transaction_status(
                            &transaction_info,
                            &previous_context.network_config,
                            previous_context.global_context.verbosity,
                        )?;

                        (previous_context.on_after_sending_transaction_callback)(
                            &transaction_info,
                            &previous_context.network_config,
                        )
                        .map_err(color_eyre::Report::msg)?;
                    }
                    None => {
                        let tx_hash = signed_transaction.get_hash();
                        eprintln!(
                            "\nTransaction sent successfully (wait level: {wait_until:?})."
                        );
                        eprintln!("Transaction ID: {tx_hash}");
                        eprintln!(
                            "To see the transaction in the transaction explorer, please open this url in your browser:\n{}{}\n",
                            previous_context.network_config.explorer_transaction_url,
                            tx_hash,
                        );
                    }
                }
            }
            super::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                signed_delegate_action,
            ) if previous_context
                .network_config
                .meta_transaction_relayer_url
                .is_some() =>
            {
                match sending_delegate_action(
                    signed_delegate_action,
                    previous_context.network_config
                        .meta_transaction_relayer_url
                        .expect("Internal error: Meta-transaction relayer URL must be Some() at this point"),
                ){
                    Ok(relayer_response) => {
                        if relayer_response.status().is_success() {
                            let response_text = relayer_response.text().map_err(color_eyre::Report::msg)?;
                            eprintln!("\nRelayer Response text: {response_text}");
                        } else {
                            eprintln!(
                                "\nRequest failed with status code: {}",
                                relayer_response.status()
                            );
                        }
                    }
                    Err(report) => return Err(color_eyre::Report::msg(report)),
                };
            }
            super::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(..) => {
                // Fallback to `display` command when `meta_transaction_relayer_url` is not configured.
                super::display::DisplayContext::from_previous_context(
                    previous_context.clone(),
                    &super::display::InteractiveClapContextScopeForDisplay {},
                )?;
            }
        }
        if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
            previous_context.global_context.verbosity
        {
            tracing_indicatif::suspend_tracing_indicatif(|| eprintln!("{storage_message}"));
        }

        Ok(Self)
    }
}

#[tracing::instrument(name = "Broadcasting transaction via RPC", skip_all)]
pub fn sending_signed_transaction(
    network_config: &crate::config::NetworkConfig,
    signed_transaction: &near_primitives::transaction::SignedTransaction,
    wait_until: near_primitives::views::TxExecutionStatus,
) -> color_eyre::Result<Option<near_primitives::views::FinalExecutionOutcomeView>> {
    tracing::Span::current().pb_set_message(network_config.rpc_url.as_str());
    tracing::info!(target: "near_teach_me", "Broadcasting transaction via RPC {}", network_config.rpc_url.as_str());

    let retries_number = 5;
    let mut retries = (1..=retries_number).rev();
    let transaction_info = loop {
        let request = near_jsonrpc_client::methods::send_tx::RpcSendTransactionRequest {
            signed_transaction: signed_transaction.clone(),
            wait_until: wait_until.clone(),
        };

        tracing::info!(
            target: "near_teach_me",
            parent: &tracing::Span::none(),
            "I am making HTTP call to NEAR JSON RPC to send a transaction, learn more https://docs.near.org/api/rpc/transactions#send-tx"
        );

        let transaction_info_result = network_config
            .json_rpc_client()
            .blocking_call(request)
            .inspect(crate::common::teach_me_call_response);
        match transaction_info_result {
            Ok(response) => {
                break response
                    .final_execution_outcome
                    .map(|outcome| outcome.into_outcome());
            }
            Err(ref err) => match crate::common::rpc_transaction_error(err) {
                Ok(message) => {
                    if let Some(retries_left) = retries.next() {
                        sleep_after_error(format!(
                            "{} (Previous attempt failed with error: `{}`. Will retry {} more times)",
                            network_config.rpc_url,
                            message.red(),
                            retries_left
                        ));
                    } else {
                        return Err(color_eyre::eyre::eyre!(err.to_string()));
                    }
                }
                Err(report) => return Err(color_eyre::Report::msg(report)),
            },
        };
    };

    Ok(transaction_info)
}

#[tracing::instrument(
    name = "Waiting 5 seconds before broadcasting transaction via RPC",
    skip_all
)]
pub fn sleep_after_error(additional_message_for_name: String) {
    tracing::Span::current().pb_set_message(&additional_message_for_name);
    tracing::info!(target: "near_teach_me", "Waiting 5 seconds before broadcasting transaction via RPC {additional_message_for_name}");
    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[tracing::instrument(name = "Broadcasting delegate action via a relayer url", skip_all)]
fn sending_delegate_action(
    signed_delegate_action: near_primitives::action::delegate::SignedDelegateAction,
    meta_transaction_relayer_url: url::Url,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    tracing::Span::current().pb_set_message(meta_transaction_relayer_url.as_str());
    tracing::info!(target: "near_teach_me", "Broadcasting delegate action via a relayer url {}", meta_transaction_relayer_url.as_str());

    let client = reqwest::blocking::Client::new();
    let request_payload = serde_json::json!({
        "signed_delegate_action": crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
            signed_delegate_action
        ).to_string()
    });
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to broadcast a transaction, learn more https://docs.near.org/concepts/abstraction/relayers"
    );
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "HTTP POST {}",
        meta_transaction_relayer_url.as_str()
    );
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "JSON Body:\n{}",
        crate::common::indent_payload(&format!("{request_payload:#}"))
    );

    let response = client
        .post(meta_transaction_relayer_url.clone())
        .json(&request_payload)
        .send()?;

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "JSON RPC Response:\n{}",
        crate::common::indent_payload(&format!("{response:#?}"))
    );

    Ok(response)
}
