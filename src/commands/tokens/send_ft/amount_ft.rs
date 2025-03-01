use color_eyre::eyre::ContextCompat;
use inquire::{CustomType, Text};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendFtCommandContext)]
#[interactive_clap(output_context = AmountFtContext)]
pub struct AmountFt {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount FT to transfer:
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
    #[interactive_clap(named_arg)]
    /// Enter a memo for transfer (optional):
    memo: FtTransferParams,
}

#[derive(Debug, Clone)]
pub struct AmountFtContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    ft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
}

impl AmountFtContext {
    pub fn from_previous_context(
        previous_context: super::SendFtCommandContext,
        scope: &<AmountFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let ft_transfer_amount =
            if let crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount =
                scope.ft_transfer_amount
            {
                crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount
            } else {
                let network_config = crate::common::find_network_where_account_exist(
                    &previous_context.global_context,
                    previous_context.ft_contract_account_id.clone(),
                )
                .wrap_err_with(|| {
                    format!(
                        "Contract <{}> does not exist in networks",
                        previous_context.ft_contract_account_id
                    )
                })?;
                let ft_metadata = crate::types::ft_properties::params_ft_metadata(
                    previous_context.ft_contract_account_id.clone(),
                    &network_config,
                    near_primitives::types::Finality::Final.into(),
                )?;
                scope.ft_transfer_amount.normalize(&ft_metadata)?
            };

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract_account_id: previous_context.ft_contract_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            ft_transfer_amount,
        })
    }
}

impl AmountFt {
    fn input_ft_transfer_amount(
        context: &super::SendFtCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::ft_properties::FungibleTokenTransferAmount>>
    {
        let network_config = crate::common::find_network_where_account_exist(
            &context.global_context,
            context.ft_contract_account_id.clone(),
        )
        .wrap_err_with(|| {
            format!(
                "Contract <{}> does not exist in networks",
                context.ft_contract_account_id
            )
        })?;

        let ft_metadata = crate::types::ft_properties::params_ft_metadata(
            context.ft_contract_account_id.clone(),
            &network_config,
            near_primitives::types::Finality::Final.into(),
        )?;
        eprintln!();

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
#[interactive_clap(output_context = FtTransferParamsContext)]
pub struct FtTransferParams {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter a memo for transfer (optional):
    memo: Option<String>,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_interactive_input)]
    gas: Option<crate::common::NearGas>,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_interactive_input)]
    deposit: Option<crate::types::near_token::NearToken>,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Clone)]
pub struct FtTransferParamsContext(crate::commands::ActionContext);

impl FtTransferParamsContext {
    pub fn from_previous_context(
        previous_context: AmountFtContext,
        scope: &<FtTransferParams as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let signer_account_id = previous_context.signer_account_id.clone();
                let ft_contract_account_id = previous_context.ft_contract_account_id.clone();
                let receiver_account_id = previous_context.receiver_account_id.clone();
                let ft_transfer_amount = previous_context.ft_transfer_amount.clone();
                let memo = scope.memo.clone();
                let gas = scope.gas.unwrap_or(near_gas::NearGas::from_tgas(100));
                let deposit = scope.deposit.unwrap_or(crate::types::near_token::NearToken::from_yoctonear(1));

                move |network_config| {
                    let amount_ft = if let crate::types::ft_properties::FungibleTokenTransferAmount::ExactAmount(ft) = &ft_transfer_amount {
                        ft.clone()
                    } else {
                        super::get_ft_balance_for_account(
                            network_config,
                            &signer_account_id,
                            &ft_contract_account_id,
                            near_primitives::types::Finality::Final.into()
                        )?
                    };

                    super::get_prepopulated_transaction(
                        network_config,
                        &ft_contract_account_id,
                        &receiver_account_id,
                        &signer_account_id,
                        &amount_ft,
                        &memo,
                        &deposit,
                        &gas
                    )
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            let signer_account_id = previous_context.signer_account_id.clone();
            let ft_contract_account_id = previous_context.ft_contract_account_id.clone();
            let receiver_account_id = previous_context.receiver_account_id.clone();

            move |outcome_view, network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    for action in outcome_view.transaction.actions.clone() {
                        if let near_primitives::views::ActionView::FunctionCall { method_name: _, args, gas: _, deposit: _ } = action {
                            if let Ok(ft_transfer) = serde_json::from_slice::<crate::types::ft_properties::FtTransfer>(&args) {
                                if let Ok(ft_balance) = super::get_ft_balance_for_account(
                                    network_config,
                                    &signer_account_id,
                                    &ft_contract_account_id,
                                    near_primitives::types::BlockId::Hash(outcome_view.receipts_outcome.last().expect("FT transfer should have at least one receipt outcome, but none was received").block_hash).into()
                                ) {
                                    let ft_transfer_amount = crate::types::ft_properties::FungibleToken::from_params_ft(
                                        ft_transfer.amount,
                                        ft_balance.decimals(),
                                        ft_balance.symbol().to_string()
                                    );
                                    let info_str = format!(
                                        "<{signer_account_id}> has successfully transferred {ft_transfer_amount} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.\nRemaining balance: {ft_balance}",
                                    );
                                    tracing::info!(
                                        parent: &tracing::Span::none(),
                                        "\n{}",
                                        crate::common::indent_payload(&info_str)
                                    );
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
                let info_str = format!(
                    "<{signer_account_id}> has successfully transferred fungible tokens (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.",
                );
                tracing::info!(
                    parent: &tracing::Span::none(),
                    "\n{}",
                    crate::common::indent_payload(&info_str)
                );
                Ok(())
            }
        });

        Ok(Self(crate::commands::ActionContext {
            global_context: previous_context.global_context,
            interacting_with_account_ids: vec![
                previous_context.ft_contract_account_id,
                previous_context.signer_account_id,
                previous_context.receiver_account_id,
            ],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback,
        }))
    }
}

impl From<FtTransferParamsContext> for crate::commands::ActionContext {
    fn from(item: FtTransferParamsContext) -> Self {
        item.0
    }
}

impl FtTransferParams {
    fn input_memo(_context: &AmountFtContext) -> color_eyre::eyre::Result<Option<String>> {
        let input = Text::new("Enter a memo for transfer (optional):").prompt()?;
        Ok(if input.trim().is_empty() {
            None
        } else {
            Some(input)
        })
    }
}
