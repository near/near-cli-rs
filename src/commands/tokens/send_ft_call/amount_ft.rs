use color_eyre::eyre::Context;
use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::FtContractContext)]
#[interactive_clap(output_context = AmountFtContext)]
pub struct AmountFt {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount FT to transfer:
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
    #[interactive_clap(named_arg)]
    /// Enter a memo for transfer (optional):
    memo: FtTransferCallParams,
}

#[derive(Debug, Clone)]
pub struct AmountFtContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    ft_contract: crate::types::ft_properties::FtContract,
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
}

impl AmountFtContext {
    pub fn from_previous_context(
        previous_context: super::FtContractContext,
        scope: &<AmountFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let ft_transfer_amount =
            if let crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount =
                scope.ft_transfer_amount
            {
                crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount
            } else if previous_context.global_context.offline
                && previous_context.ft_contract.ft_metadata.symbol.is_empty()
            {
                return Err(color_eyre::Report::msg(
                    "Cannot enter an exact FT amount to transfer in offline mode when FT metadata does not contain symbol information, because the prompt for entering FT amount will not contain token symbol information, and the amount of decimals in the entered amount will not be validated against the decimals specified in FT metadata.",
                ));
            } else {
                scope
                    .ft_transfer_amount
                    .normalize(&previous_context.ft_contract.ft_metadata)?
            };

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            ft_contract: previous_context.ft_contract,
            ft_transfer_amount,
        })
    }
}

impl AmountFt {
    pub fn input_receiver_account_id(
        context: &super::FtContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }

    fn input_ft_transfer_amount(
        context: &super::FtContractContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::ft_properties::FungibleTokenTransferAmount>>
    {
        let ft_metadata = context.ft_contract.ft_metadata.clone();

        if context.global_context.offline && ft_metadata.symbol.is_empty() {
            return Err(color_eyre::Report::msg(format!(
                "FT metadata for contract <{}> does not contain symbol information. Cannot enter an exact FT amount to transfer in offline mode when FT metadata does not contain symbol information, because the prompt for entering FT amount will not contain token symbol information, and the amount of decimals in the entered amount will not be validated against the decimals specified in FT metadata.",
                context.ft_contract.ft_contract_account_id
            )));
        }

        Ok(Some(
            CustomType::<crate::types::ft_properties::FungibleTokenTransferAmount>::new(&format!(
                "Enter an FT amount to transfer (example: 10 {symbol} or 0.5 {symbol} or \"all\" to transfer the entire amount of fungible tokens from your account):",
                symbol = ft_metadata.symbol
            ))
            .with_validator(move |ft: &crate::types::ft_properties::FungibleTokenTransferAmount| {
                match ft.normalize(&ft_metadata) {
                    Err(err) => Ok(inquire::validator::Validation::Invalid(
                        inquire::validator::ErrorMessage::Custom(err.to_string()),
                    )),
                    Ok(_) => Ok(inquire::validator::Validation::Valid),
                }
            })
            .with_formatter(&|ft| ft.to_string())
            .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = AmountFtContext)]
#[interactive_clap(output_context = FtTransferCallParamsContext)]
pub struct FtTransferCallParams {
    /// Enter a memo for transfer (optional):
    memo: String,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_interactive_input)]
    gas: Option<crate::common::NearGas>,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_interactive_input)]
    deposit: Option<crate::types::near_token::NearToken>,
    #[interactive_clap(subcommand)]
    /// How would you like to provide the msg for the receiving contract?
    msg_type: MsgType,
}

#[derive(Debug, Clone)]
pub struct FtTransferCallParamsContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    ft_metadata: crate::types::ft_properties::FtMetadata,
    receiver_account_id: near_primitives::types::AccountId,
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
    memo: String,
    gas: crate::common::NearGas,
    deposit: crate::types::near_token::NearToken,
}

impl FtTransferCallParamsContext {
    pub fn from_previous_context(
        previous_context: AmountFtContext,
        scope: &<FtTransferCallParams as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let gas = scope.gas.unwrap_or(near_gas::NearGas::from_tgas(100));
        let deposit = scope
            .deposit
            .unwrap_or(crate::types::near_token::NearToken::from_yoctonear(1));

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract_account_id: previous_context.ft_contract.ft_contract_account_id,
            ft_metadata: previous_context.ft_contract.ft_metadata,
            receiver_account_id: previous_context.receiver_account_id,
            ft_transfer_amount: previous_context.ft_transfer_amount,
            memo: scope.memo.trim().to_string(),
            gas,
            deposit,
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = FtTransferCallParamsContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// How would you like to provide the msg for the receiving contract?
pub enum MsgType {
    #[strum_discriminants(strum(message = "msg-args   - Pass the msg string directly"))]
    /// Pass the msg string directly
    MsgArgs(MsgArgs),
    #[strum_discriminants(strum(message = "msg-file   - Read msg from a file"))]
    /// Read msg from a file
    MsgFile(MsgFile),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = FtTransferCallParamsContext)]
#[interactive_clap(output_context = MsgArgsContext)]
pub struct MsgArgs {
    /// Enter the msg for the receiving contract:
    msg: String,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct MsgArgsContext(crate::commands::ActionContext);

impl MsgArgsContext {
    pub fn from_previous_context(
        previous_context: FtTransferCallParamsContext,
        scope: &<MsgArgs as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let msg = scope.msg.clone();
        Ok(Self(build_action_context(previous_context, msg)?))
    }
}

impl From<MsgArgsContext> for crate::commands::ActionContext {
    fn from(item: MsgArgsContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = FtTransferCallParamsContext)]
#[interactive_clap(output_context = MsgFileContext)]
pub struct MsgFile {
    /// Enter the path to the file containing the msg for the receiving contract:
    file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct MsgFileContext(crate::commands::ActionContext);

impl MsgFileContext {
    pub fn from_previous_context(
        previous_context: FtTransferCallParamsContext,
        scope: &<MsgFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let msg = std::fs::read_to_string(&scope.file_path)
            .wrap_err_with(|| format!("Failed to read msg from file: {:?}", scope.file_path))?
            .trim()
            .to_string();
        Ok(Self(build_action_context(previous_context, msg)?))
    }
}

impl From<MsgFileContext> for crate::commands::ActionContext {
    fn from(item: MsgFileContext) -> Self {
        item.0
    }
}

fn build_action_context(
    previous_context: FtTransferCallParamsContext,
    msg: String,
) -> color_eyre::eyre::Result<crate::commands::ActionContext> {
    let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
        std::sync::Arc::new({
            let signer_account_id = previous_context.signer_account_id.clone();
            let ft_contract_account_id = previous_context.ft_contract_account_id.clone();
            let receiver_account_id = previous_context.receiver_account_id.clone();
            let ft_transfer_amount = previous_context.ft_transfer_amount.clone();
            let memo = previous_context.memo.clone();
            let msg = msg.clone();
            let gas = previous_context.gas;
            let deposit = previous_context.deposit;
            let verbosity = previous_context.global_context.verbosity;

            move |network_config| {
                if !crate::common::validate_receiver_account_id(
                    network_config,
                    &receiver_account_id,
                    verbosity,
                    previous_context.global_context.offline,
                )? {
                    return Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: signer_account_id.clone(),
                        receiver_id: ft_contract_account_id.clone(),
                        actions: vec![],
                    });
                }

                let amount_ft = if let crate::types::ft_properties::FungibleTokenTransferAmount::ExactAmount(ft) = &ft_transfer_amount {
                    ft
                } else {
                    if previous_context.global_context.offline {
                        return Err(color_eyre::Report::msg(
                    "It is not possible to calculate the full amount of fungible tokens in your account offline."
                        ));
                    }
                    &crate::commands::tokens::send_ft::get_ft_balance_for_account(
                        network_config,
                        &signer_account_id,
                        &ft_contract_account_id,
                        near_primitives::types::Finality::Final.into(),
                    )?
                };

                super::get_prepopulated_transaction(
                    previous_context.global_context.offline,
                    network_config,
                    &ft_contract_account_id,
                    &receiver_account_id,
                    &signer_account_id,
                    amount_ft,
                    &memo,
                    &msg,
                    deposit,
                    gas,
                )
            }
        });

    let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
        let signer_account_id = previous_context.signer_account_id.clone();
        let ft_contract_account_id = previous_context.ft_contract_account_id.clone();
        let ft_metadata = previous_context.ft_metadata.clone();
        let receiver_account_id = previous_context.receiver_account_id.clone();
        let requested_amount = previous_context.ft_transfer_amount.clone();
        let verbosity = previous_context.global_context.verbosity;

        move |outcome_view, network_config| {
            if let crate::Verbosity::Quiet = verbosity {
                return Ok(());
            }

            let ft_transfer_call = outcome_view.transaction.actions.iter().find_map(|action| {
                if let near_primitives::views::ActionView::FunctionCall {
                    method_name, args, ..
                } = action
                    && method_name == "ft_transfer_call"
                {
                    serde_json::from_slice::<crate::types::ft_properties::FtTransferCall>(args)
                        .ok()
                } else {
                    None
                }
            });
            let receiver_callback_status = outcome_view
                .receipts_outcome
                .iter()
                .find(|receipt| receipt.outcome.executor_id == receiver_account_id)
                .map(|receipt| &receipt.outcome.status);
            // A failure after the initial FT receipt may happen after its transfer was committed.
            let initial_ft_transfer_failed = matches!(
                &outcome_view.transaction_outcome.outcome.status,
                near_primitives::views::ExecutionStatusView::Failure(_)
            ) || outcome_view
                .transaction_outcome
                .outcome
                .receipt_ids
                .iter()
                .filter_map(|receipt_id| {
                    outcome_view
                        .receipts_outcome
                        .iter()
                        .find(|receipt| receipt.id == *receipt_id)
                })
                .any(|receipt| {
                    receipt.outcome.executor_id == ft_contract_account_id
                        && matches!(
                            &receipt.outcome.status,
                            near_primitives::views::ExecutionStatusView::Failure(_)
                        )
                });
            let transferred_amount = match &outcome_view.status {
                // The resolver's successful result reports the amount retained by the receiver.
                // See NEP-141: https://github.com/near/NEPs/blob/master/neps/nep-0141.md?plain=1#L292-L331
                near_primitives::views::FinalExecutionStatus::SuccessValue(return_value) => {
                    ft_transfer_call.as_ref().and_then(|ft_transfer_call| {
                        parse_ft_transfer_call_transferred_amount(
                            return_value,
                            ft_transfer_call.amount,
                        )
                    })
                }
                // Report zero only when the initial FT-contract execution failed; a later failure
                // may occur after the transfer was committed, so its amount is unknown.
                near_primitives::views::FinalExecutionStatus::Failure(_) => {
                    initial_ft_transfer_failed.then_some(0)
                }
                // The outcome is not final, so the transferred amount cannot yet be determined.
                near_primitives::views::FinalExecutionStatus::NotStarted
                | near_primitives::views::FinalExecutionStatus::Started => None,
            };
            let failure_reason = match &outcome_view.status {
                near_primitives::views::FinalExecutionStatus::Failure(error) => {
                    Some(format!("{error:?}"))
                }
                near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                    receiver_callback_status.and_then(|status| match status {
                        near_primitives::views::ExecutionStatusView::Failure(error) => {
                            Some(format!("{error:?}"))
                        }
                        _ => None,
                    })
                }
                near_primitives::views::FinalExecutionStatus::NotStarted
                | near_primitives::views::FinalExecutionStatus::Started => None,
            };
            // Find the FT-contract receipt with the final transaction result to query the balance after resolution.
            let remaining_balance_block_hash = outcome_view
                .receipts_outcome
                .iter()
                .find(|receipt| {
                    receipt.outcome.executor_id == ft_contract_account_id
                        && match (&outcome_view.status, &receipt.outcome.status) {
                            (
                                near_primitives::views::FinalExecutionStatus::SuccessValue(
                                    final_value,
                                ),
                                near_primitives::views::ExecutionStatusView::SuccessValue(
                                    receipt_value,
                                ),
                            ) => final_value == receipt_value,
                            (
                                near_primitives::views::FinalExecutionStatus::Failure(
                                    final_error,
                                ),
                                near_primitives::views::ExecutionStatusView::Failure(
                                    receipt_error,
                                ),
                            ) => final_error == receipt_error,
                            _ => false,
                        }
                })
                .map(|receipt| receipt.block_hash);
            let remaining_balance = remaining_balance_block_hash
                .and_then(|block_hash| {
                    crate::commands::tokens::send_ft::get_ft_balance_for_account(
                        network_config,
                        &signer_account_id,
                        &ft_contract_account_id,
                        near_primitives::types::BlockId::Hash(block_hash).into(),
                    )
                    .ok()
                })
                .map(|balance| balance.to_string())
                .unwrap_or_else(|| "N/A".to_string());

            let requested_amount = match &requested_amount {
                    crate::types::ft_properties::FungibleTokenTransferAmount::ExactAmount(amount) => {
                        Some(amount.clone())
                    }
                    crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount => {
                        ft_transfer_call
                            .as_ref()
                            .map(|ft_transfer_call| {
                                crate::types::ft_properties::FungibleToken::from_params_ft(
                                    ft_transfer_call.amount,
                                    ft_metadata.decimals,
                                    ft_metadata.symbol.clone(),
                                )
                            })
                    }
            };
            let transfer_message = match transferred_amount {
                    Some(amount)
                        if let Some(requested_amount) = &requested_amount
                            && amount < requested_amount.amount() =>
                    {
                        let transferred_amount =
                            crate::types::ft_properties::FungibleToken::from_params_ft(
                                amount,
                                ft_metadata.decimals,
                                ft_metadata.symbol.clone(),
                            );
                        let returned_amount =
                            crate::types::ft_properties::FungibleToken::from_params_ft(
                                requested_amount.amount() - amount,
                                ft_metadata.decimals,
                                ft_metadata.symbol.clone(),
                            );
                        format!(
                            "<{signer_account_id}> has transferred {transferred_amount} of the requested {requested_amount} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>; {returned_amount} was returned to the sender."
                        )
                    }
                    Some(amount) if amount > 0 => {
                        let transferred_amount =
                            crate::types::ft_properties::FungibleToken::from_params_ft(
                                amount,
                                ft_metadata.decimals,
                                ft_metadata.symbol.clone(),
                            );
                        format!(
                            "<{signer_account_id}> has successfully transferred {transferred_amount} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>."
                        )
                    }
                    Some(_) => match &requested_amount {
                        Some(requested_amount) => format!(
                            "<{signer_account_id}> could not transfer {requested_amount} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>. Reason: {}.",
                            failure_reason
                                .as_deref()
                                .unwrap_or("the full amount was returned to the sender")
                        ),
                        None => format!(
                            "<{signer_account_id}> could not complete ft_transfer_call (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>. Reason: {}.",
                            failure_reason
                                .as_deref()
                                .unwrap_or("the full amount was returned to the sender")
                        ),
                    },
                    None => {
                        let failure_reason = failure_reason
                            .as_deref()
                            .map(|reason| format!(" Reason: {reason}"))
                            .unwrap_or_default();
                        match &requested_amount {
                            Some(requested_amount) => format!(
                                "<{signer_account_id}> could not determine the transferred amount of {requested_amount} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.{failure_reason}"
                            ),
                            None => format!(
                                "<{signer_account_id}> could not determine the transferred amount (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.{failure_reason}"
                            ),
                        }
                    }
            };

            tracing_indicatif::suspend_tracing_indicatif(|| eprintln!(
                "{transfer_message}\nRemaining balance: {remaining_balance}",
            ));
            Ok(())
        }
    });

    Ok(crate::commands::ActionContext {
        global_context: previous_context.global_context,
        interacting_with_account_ids: vec![
            previous_context.ft_contract_account_id,
            previous_context.signer_account_id,
            previous_context.receiver_account_id,
        ],
        get_prepopulated_transaction_after_getting_network_callback,
        on_before_signing_callback: std::sync::Arc::new(
            |_prepopulated_unsigned_transaction, _network_config| Ok(()),
        ),
        on_before_sending_transaction_callback: std::sync::Arc::new(
            |_signed_transaction, _network_config| Ok(String::new()),
        ),
        on_after_sending_transaction_callback,
        sign_as_delegate_action: false,
        on_sending_delegate_action_callback: None,
    })
}

fn parse_ft_transfer_call_transferred_amount(
    resolver_return_value: &[u8],
    requested_amount: u128,
) -> Option<u128> {
    // ft_resolve_transfer returns the amount kept by the receiver; ft_on_transfer returns the unused amount.
    let transferred_amount = serde_json::from_slice::<String>(resolver_return_value)
        .ok()?
        .parse::<u128>()
        .ok()?;
    (transferred_amount <= requested_amount).then_some(transferred_amount)
}

#[cfg(test)]
mod tests {
    use super::parse_ft_transfer_call_transferred_amount;

    #[test]
    fn parses_transferred_amount_from_resolver_result() {
        assert_eq!(
            parse_ft_transfer_call_transferred_amount(br#""25""#, 100),
            Some(25)
        );
    }

    #[test]
    fn returns_zero_when_resolver_reports_a_full_refund() {
        assert_eq!(
            parse_ft_transfer_call_transferred_amount(br#""0""#, 100),
            Some(0)
        );
    }

    #[test]
    fn returns_requested_amount_when_resolver_reports_full_acceptance() {
        assert_eq!(
            parse_ft_transfer_call_transferred_amount(br#""100""#, 100),
            Some(100)
        );
    }

    #[test]
    fn rejects_invalid_or_inconsistent_resolver_results() {
        assert_eq!(parse_ft_transfer_call_transferred_amount(b"", 100), None);
        assert_eq!(
            parse_ft_transfer_call_transferred_amount(br#""not-a-number""#, 100),
            None
        );
        assert_eq!(
            parse_ft_transfer_call_transferred_amount(br#""101""#, 100),
            None
        );
    }
}
