use color_eyre::eyre::Context;
use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::types::mt_ft_inventory::get_mt_ft_balance;
use crate::types::mt_ft_properties::{IntentsTokenId, MtFtMetadata};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::IntentContractIdContext)]
#[interactive_clap(output_context = AmountMtFtContext)]
pub struct AmountMtFt {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount MT-FT to transfer:
    mt_ft_transfer_amount: crate::types::ft_properties::FungibleToken,
    #[interactive_clap(named_arg)]
    /// Enter a memo for transfer (optional):
    memo: MtFtTransferCallParams,
}

#[derive(Debug, Clone)]
pub struct AmountMtFtContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    mt_contract: near_primitives::types::AccountId,
    token_id: IntentsTokenId,
    mt_ft_metadata: MtFtMetadata,
    receiver_account_id: near_primitives::types::AccountId,
    mt_ft_transfer_amount: crate::types::ft_properties::FungibleToken,
}

impl AmountMtFtContext {
    pub fn from_previous_context(
        previous_context: super::IntentContractIdContext,
        scope: &<AmountMtFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mt_ft_transfer_amount = scope
            .mt_ft_transfer_amount
            .clone()
            .normalize(&previous_context.mt_ft_metadata.clone().into())?;

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            mt_contract: previous_context.mt_contract,
            token_id: previous_context.token_id,
            mt_ft_metadata: previous_context.mt_ft_metadata,
            receiver_account_id: scope.receiver_account_id.clone().into(),
            mt_ft_transfer_amount,
        })
    }
}

impl AmountMtFt {
    pub fn input_receiver_account_id(
        context: &super::IntentContractIdContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the receiver account ID?",
        )
    }

    fn input_mt_ft_transfer_amount(
        context: &super::IntentContractIdContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::ft_properties::FungibleToken>> {
        let mt_ft_metadata = context.mt_ft_metadata.clone();

        Ok(Some(
            CustomType::<crate::types::ft_properties::FungibleToken>::new(&format!(
                "Enter an MT-FT amount to transfer (example: 10 {symbol} or 0.5 {symbol}):",
                symbol = mt_ft_metadata.symbol
            ))
            .with_validator(move |ft: &crate::types::ft_properties::FungibleToken| {
                match ft.normalize(&mt_ft_metadata.clone().into()) {
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
#[interactive_clap(input_context = AmountMtFtContext)]
#[interactive_clap(output_context = MtFtTransferCallParamsContext)]
pub struct MtFtTransferCallParams {
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

#[derive(Clone)]
pub struct MtFtTransferCallParamsContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    mt_contract: near_primitives::types::AccountId,
    token_id: IntentsTokenId,
    mt_ft_metadata: MtFtMetadata,
    receiver_account_id: near_primitives::types::AccountId,
    mt_ft_transfer_amount: crate::types::ft_properties::FungibleToken,
    memo: String,
    gas: crate::common::NearGas,
    deposit: crate::types::near_token::NearToken,
}

impl MtFtTransferCallParamsContext {
    pub fn from_previous_context(
        previous_context: AmountMtFtContext,
        scope: &<MtFtTransferCallParams as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let gas = scope.gas.unwrap_or(near_gas::NearGas::from_tgas(100));
        let deposit = scope
            .deposit
            .unwrap_or(crate::types::near_token::NearToken::from_yoctonear(1));

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            mt_contract: previous_context.mt_contract,
            token_id: previous_context.token_id,
            mt_ft_metadata: previous_context.mt_ft_metadata,
            receiver_account_id: previous_context.receiver_account_id,
            mt_ft_transfer_amount: previous_context.mt_ft_transfer_amount,
            memo: scope.memo.trim().to_string(),
            gas,
            deposit,
        })
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = MtFtTransferCallParamsContext)]
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
#[interactive_clap(input_context = MtFtTransferCallParamsContext)]
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
        previous_context: MtFtTransferCallParamsContext,
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
#[interactive_clap(input_context = MtFtTransferCallParamsContext)]
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
        previous_context: MtFtTransferCallParamsContext,
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
    previous_context: MtFtTransferCallParamsContext,
    msg: String,
) -> color_eyre::eyre::Result<crate::commands::ActionContext> {
    let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = previous_context.signer_account_id.clone();
                let mt_contract = previous_context.mt_contract.clone();
                let receiver_account_id = previous_context.receiver_account_id.clone();
                let amount = previous_context.mt_ft_transfer_amount.clone();
                let token_id = previous_context.token_id.clone();
                let memo = previous_context.memo.clone();
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
                            receiver_id: receiver_account_id.clone(),
                            actions: vec![],
                        });
                    }

                    super::get_prepopulated_transaction(
                        &mt_contract,
                        &receiver_account_id,
                        token_id.clone().to_string(),
                        &signer_account_id,
                        &amount,
                        &memo,
                        &msg,
                        deposit,
                        gas
                    )
                }
            });

    let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer_account_id = previous_context.signer_account_id.clone();
            let mt_contract = previous_context.mt_contract.clone();
            let mt_ft_metadata = previous_context.mt_ft_metadata.clone();
            let token_id = previous_context.token_id.clone();
            let receiver_account_id = previous_context.receiver_account_id.clone();
            let requested_amount = previous_context.mt_ft_transfer_amount.amount();
            let verbosity = previous_context.global_context.verbosity;

            move |outcome_view, network_config| {
                let receiver_callback_status = outcome_view
                    .receipts_outcome
                    .iter()
                    .find(|receipt| receipt.outcome.executor_id == receiver_account_id)
                    .map(|receipt| &receipt.outcome.status);
                let transferred_amount = match &outcome_view.status {
                    near_primitives::views::FinalExecutionStatus::SuccessValue(return_value) => {
                        let receiver_callback_succeeded =
                            receiver_callback_status.and_then(|status| match status {
                                near_primitives::views::ExecutionStatusView::SuccessValue(_) => {
                                    Some(true)
                                }
                                near_primitives::views::ExecutionStatusView::Failure(_) => {
                                    Some(false)
                                }
                                near_primitives::views::ExecutionStatusView::Unknown
                                | near_primitives::views::ExecutionStatusView::SuccessReceiptId(_) => {
                                    None
                                }
                            });
                        parse_mt_transfer_call_transferred_amount(
                            return_value,
                            requested_amount,
                            receiver_callback_succeeded,
                        )
                    }
                    near_primitives::views::FinalExecutionStatus::Failure(_) => Some(0),
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
                // Find the MT-contract receipt with the final transaction result to query the balance after resolution.
                let remaining_balance_block_hash = outcome_view
                    .receipts_outcome
                    .iter()
                    .find(|receipt| {
                        receipt.outcome.executor_id == mt_contract
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
                        tokio::runtime::Runtime::new()
                            .expect("Failed to create a new tokio runtime")
                            .block_on(get_mt_ft_balance(
                                network_config,
                                &mt_contract,
                                &token_id,
                                &signer_account_id,
                                near_primitives::types::BlockId::Hash(block_hash).into(),
                            ))
                            .ok()
                    })
                    .and_then(|balance| balance.parse::<u128>().ok())
                    .map(|amount| {
                        crate::types::ft_properties::FungibleToken::from_params_ft(
                            amount,
                            mt_ft_metadata.decimals,
                            mt_ft_metadata.symbol.clone(),
                        )
                        .to_string()
                    })
                    .unwrap_or_else(|| "N/A".to_string());

                if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                    let requested_amount = crate::types::ft_properties::FungibleToken::from_params_ft(
                        requested_amount,
                        mt_ft_metadata.decimals,
                        mt_ft_metadata.symbol.clone(),
                    );
                    let transfer_message = match transferred_amount {
                        Some(amount) if amount > 0 => {
                            let transferred_amount = crate::types::ft_properties::FungibleToken::from_params_ft(
                                amount,
                                mt_ft_metadata.decimals,
                                mt_ft_metadata.symbol.clone(),
                            );
                            format!(
                                "<{signer_account_id}> has successfully transferred {transferred_amount} (MT-FT-contract: {mt_contract}, token ID: {token_id}) to <{receiver_account_id}>."
                            )
                        }
                        Some(_) => {
                            format!(
                                "<{signer_account_id}> could not transfer {requested_amount} (MT-FT-contract: {mt_contract}, token ID: {token_id}) to <{receiver_account_id}>. Reason: {}.",
                                failure_reason.as_deref().unwrap_or("the full amount was returned to the sender")
                            )
                        }
                        None => {
                            format!(
                                "<{signer_account_id}> could not determine the transferred amount of {requested_amount} (MT-FT-contract: {mt_contract}, token ID: {token_id}) to <{receiver_account_id}>."
                            )
                        }
                    };

                    tracing_indicatif::suspend_tracing_indicatif(|| eprintln!(
                        "{transfer_message}\nRemaining balance: {remaining_balance}",
                    ));
                }
                Ok(())
            }
        });

    Ok(crate::commands::ActionContext {
        global_context: previous_context.global_context,
        interacting_with_account_ids: vec![
            previous_context.mt_contract,
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

fn parse_mt_transfer_call_transferred_amount(
    return_value: &[u8],
    requested_amount: u128,
    receiver_callback_succeeded: Option<bool>,
) -> Option<u128> {
    let amounts = serde_json::from_slice::<Vec<String>>(return_value).ok()?;
    let [returned_amount] = amounts.as_slice() else {
        return None;
    };
    let returned_amount = returned_amount.parse::<u128>().ok()?;

    if returned_amount == 0 {
        return match receiver_callback_succeeded {
            Some(true) => Some(requested_amount),
            Some(false) => Some(0),
            None => None,
        };
    }

    requested_amount.checked_sub(returned_amount)
}

#[cfg(test)]
mod tests {
    use super::parse_mt_transfer_call_transferred_amount;

    #[test]
    fn calculates_transferred_amount_when_resolver_returns_a_refund() {
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["25"]"#, 100, None),
            Some(75)
        );
    }

    #[test]
    fn treats_zero_return_as_full_transfer_when_receiver_callback_succeeds() {
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["0"]"#, 100, Some(true)),
            Some(100)
        );
    }

    #[test]
    fn treats_zero_return_as_no_transfer_when_receiver_callback_fails() {
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["0"]"#, 100, Some(false)),
            Some(0)
        );
    }

    #[test]
    fn does_not_infer_transfer_amount_when_receiver_callback_result_is_unknown() {
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["0"]"#, 100, None),
            None
        );
    }

    #[test]
    fn treats_a_full_refund_as_no_transfer() {
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["100"]"#, 100, None),
            Some(0)
        );
    }

    #[test]
    fn does_not_infer_transfer_amount_from_invalid_or_inconsistent_result() {
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(b"", 100, None),
            None
        );
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["not-a-number"]"#, 100, None),
            None
        );
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["100","0"]"#, 100, None),
            None
        );
        assert_eq!(
            parse_mt_transfer_call_transferred_amount(br#"["101"]"#, 100, None),
            None
        );
    }
}
