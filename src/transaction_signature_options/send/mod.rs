use color_eyre::owo_colors::OwoColorize;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SubmitContext)]
#[interactive_clap(output_context = SendContext)]
pub struct Send;

#[derive(Debug, Clone)]
pub struct SendContext;

impl SendContext {
    #[tracing::instrument(skip_all, name = "Sending transaction ...")]
    pub fn from_previous_context(
        previous_context: super::SubmitContext,
        _scope: &<Send as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut storage_message = String::new();

        match previous_context.signed_transaction_or_signed_delegate_action {
            super::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                signed_transaction,
            ) => {
                (previous_context.on_before_sending_transaction_callback)(
                    &signed_transaction,
                    &previous_context.network_config,
                    &mut storage_message,
                )
                .map_err(color_eyre::Report::msg)?;

                let retries_number = 5;
                let mut retries = (0..retries_number).rev();
                let transaction_info = loop {
                    let transaction_info_result =
                        tokio::runtime::Runtime::new()
                            .unwrap()
                            .block_on(sending_transaction(
                                format!("{}", previous_context.network_config.rpc_url),
                                &previous_context.network_config.json_rpc_client(),
                                signed_transaction.clone(),
                            ));
                    match transaction_info_result {
                        Ok(response) => {
                            break response;
                        }
                        Err(ref err) => match crate::common::rpc_transaction_error(err) {
                            Ok(message) => {
                                let retr = retries.next();
                                if let Some(retries_left) = retries.next() {
                                    sleep_after_error(
                                        format!("{} (Previous attempt failed with error: `{}`. Will retry {} more times)",
                                        previous_context.network_config.rpc_url,
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

                crate::common::print_transaction_status(
                    &transaction_info,
                    &previous_context.network_config,
                )?;

                (previous_context.on_after_sending_transaction_callback)(
                    &transaction_info,
                    &previous_context.network_config,
                )
                .map_err(color_eyre::Report::msg)?;

                eprintln!("{storage_message}");
            }
            super::SignedTransactionOrSignedDelegateAction::SignedDelegateAction(
                signed_delegate_action,
            ) => {
                let client = reqwest::blocking::Client::new();
                let json_payload = serde_json::json!({
                    "signed_delegate_action": crate::types::signed_delegate_action::SignedDelegateActionAsBase64::from(
                        signed_delegate_action
                    ).to_string()
                });
                match client
                    .post(
                        previous_context
                            .network_config
                            .meta_transaction_relayer_url
                            .expect("Internal error: Meta-transaction relayer URL must be Some() at this point"),
                    )
                    .json(&json_payload)
                    .send()
                {
                    Ok(relayer_response) => {
                        if relayer_response.status().is_success() {
                            let response_text = relayer_response.text()
                                .map_err(color_eyre::Report::msg)?;
                            println!("Relayer Response text: {}", response_text);
                        } else {
                            println!(
                                "Request failed with status code: {}",
                                relayer_response.status()
                            );
                        }
                    }
                    Err(report) => {
                        return Err(
                            color_eyre::Report::msg(report),
                        )
                    }
                }
                eprintln!("{storage_message}");
            }
        }
        Ok(Self)
    }
}

#[tracing::instrument(
    skip_all,
    name = "Waiting 5 seconds before broadcasting transaction via RPC"
)]
fn sleep_after_error(additional_message_for_name: String) {
    tracing::Span::current().pb_set_message(&additional_message_for_name);
    std::thread::sleep(std::time::Duration::from_secs(5));
}

#[tracing::instrument(name = "Broadcasting transaction via RPC", skip_all)]
async fn sending_transaction(
    additional_message_for_name: String,
    json_rpc_client: &near_jsonrpc_client::JsonRpcClient,
    signed_transaction: near_primitives::transaction::SignedTransaction,
) -> Result<
    near_primitives::views::FinalExecutionOutcomeView,
    near_jsonrpc_client::errors::JsonRpcError<
        near_jsonrpc_primitives::types::transactions::RpcTransactionError,
    >,
> {
    tracing::Span::current().pb_set_message(&additional_message_for_name);
    json_rpc_client
        .call(
            near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                signed_transaction: signed_transaction.clone(),
            },
        )
        .await
}
