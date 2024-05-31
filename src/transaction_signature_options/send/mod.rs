use color_eyre::owo_colors::OwoColorize;
use tracing_indicatif::span_ext::IndicatifSpanExt;

use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SubmitContext)]
#[interactive_clap(output_context = SendContext)]
pub struct Send;

#[derive(Debug, Clone)]
pub struct SendContext;

impl SendContext {
    #[tracing::instrument(name = "Sending transaction ...", skip_all)]
    pub fn from_previous_context(
        previous_context: super::SubmitContext,
        _scope: &<Send as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let storage_message = (previous_context.on_before_sending_transaction_callback)(
            &previous_context.signed_transaction_or_signed_delegate_action,
            &previous_context.network_config,
        )
        .map_err(color_eyre::Report::msg)?;

        match previous_context.signed_transaction_or_signed_delegate_action {
            super::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                signed_transaction,
            ) => {
                let transaction_info = sending_signed_transaction(
                    &previous_context.network_config,
                    &signed_transaction,
                )?;

                crate::common::print_transaction_status(
                    &transaction_info,
                    &previous_context.network_config,
                )?;

                (previous_context.on_after_sending_transaction_callback)(
                    &transaction_info,
                    &previous_context.network_config,
                )
                .map_err(color_eyre::Report::msg)?;
            }
            super::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                signed_delegate_action,
            ) => {
                match sending_delegate_action(
                    signed_delegate_action,
                    previous_context.network_config
                        .meta_transaction_relayer_url
                        .expect("Internal error: Meta-transaction relayer URL must be Some() at this point"),
                ){
                    Ok(relayer_response) => {
                        if relayer_response.status().is_success() {
                            let response_text = relayer_response.text().map_err(color_eyre::Report::msg)?;
                            eprintln!("\nRelayer Response text: {}", response_text);
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
        }
        eprintln!("{storage_message}");
        Ok(Self)
    }
}

#[tracing::instrument(name = "Broadcasting transaction via RPC", skip_all)]
pub fn sending_signed_transaction(
    network_config: &crate::config::NetworkConfig,
    signed_transaction: &near_primitives::transaction::SignedTransaction,
) -> color_eyre::Result<near_primitives::views::FinalExecutionOutcomeView> {
    tracing::Span::current().pb_set_message(network_config.rpc_url.as_str());
    let retries_number = 5;
    let mut retries = (1..=retries_number).rev();
    let transaction_info = loop {
        let transaction_info_result = network_config.json_rpc_client().blocking_call(
            near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: signed_transaction.clone(),
            },
        );
        match transaction_info_result {
            Ok(response) => {
                break response;
            }
            Err(ref err) => match crate::common::rpc_transaction_error(err) {
                Ok(message) => {
                    if let Some(retries_left) = retries.next() {
                        sleep_after_error(
                            format!("{} (Previous attempt failed with error: `{}`. Will retry {} more times)",
                            network_config.rpc_url,
                            message.red(),
                            retries_left)
                        );
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
    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[tracing::instrument(name = "Broadcasting delegate action via a relayer url", skip_all)]
fn sending_delegate_action(
    signed_delegate_action: near_primitives::action::delegate::SignedDelegateAction,
    meta_transaction_relayer_url: url::Url,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    tracing::Span::current().pb_set_message(meta_transaction_relayer_url.as_str());
    let client = reqwest::blocking::Client::new();
    let json_payload = serde_json::json!({
        "signed_delegate_action": crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
            signed_delegate_action
        ).to_string()
    });
    client
        .post(meta_transaction_relayer_url)
        .json(&json_payload)
        .send()
}
