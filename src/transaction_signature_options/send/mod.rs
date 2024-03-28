use crate::common::JsonRpcClientExt;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(input_context = super::SubmitContext)]
#[interactive_clap(output_context = SendContext)]
pub struct Send;

#[derive(Debug, Clone)]
pub struct SendContext;

impl SendContext {
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

                // eprintln!("Transaction sent ..."); // long-spinner download iterator
                let retries_number = 5;
                let mut retries_left = (0..retries_number).rev();
                let transaction_info = loop {
                    let pb = indicatif::ProgressBar::new_spinner();
                    pb.enable_steady_tick(std::time::Duration::from_millis(120));
                    pb.set_style(
                        indicatif::ProgressStyle::with_template("{spinner:.blue} {msg}")
                            .unwrap()
                            .tick_strings(&[
                                "▹▹▹▹▹",
                                "▸▹▹▹▹",
                                "▹▸▹▹▹",
                                "▹▹▸▹▹",
                                "▹▹▹▸▹",
                                "▹▹▹▹▸",
                                "▪▪▪▪▪",
                            ]),
                    );
                    if retries_left.len() < retries_number {
                        pb.set_message(format!(
                            "Sending a transaction ({} retries left) ...",
                            retries_left.len() + 1
                        ));
                    } else {
                        pb.set_message("Sending a transaction ...");
                    }
                    std::thread::sleep(std::time::Duration::from_secs(5));

                    let transaction_info_result = previous_context.network_config.json_rpc_client()
                    .blocking_call(
                        near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest{
                            signed_transaction: signed_transaction.clone()
                        }
                    );
                    match transaction_info_result {
                        Ok(response) => {
                            pb.finish_with_message("Your transaction has been sent successfully.");
                            break response;
                        }
                        Err(ref err) => match crate::common::rpc_transaction_error(err) {
                            Ok(_) => {
                                if retries_left.next().is_some() {
                                    std::thread::sleep(std::time::Duration::from_millis(100));
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
