use base64::Engine as _;
use color_eyre::eyre::Context;
use inquire::CustomType;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendFtCallCommandContext)]
#[interactive_clap(output_context = AmountFtContext)]
pub struct AmountFt {
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
    signer_account_id: near_kit::AccountId,
    receiver_account_id: near_kit::AccountId,
    ft_contract: crate::types::ft_properties::FtContract,
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
}

impl AmountFtContext {
    pub fn from_previous_context(
        previous_context: super::SendFtCallCommandContext,
        scope: &<AmountFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let ft_transfer_amount =
            if let crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount =
                scope.ft_transfer_amount
            {
                crate::types::ft_properties::FungibleTokenTransferAmount::MaxAmount
            } else {
                scope
                    .ft_transfer_amount
                    .normalize(&previous_context.ft_contract.ft_metadata)?
            };

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            ft_contract: previous_context.ft_contract,
            ft_transfer_amount,
        })
    }
}

impl AmountFt {
    fn input_ft_transfer_amount(
        context: &super::SendFtCallCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::ft_properties::FungibleTokenTransferAmount>>
    {
        let ft_metadata = context.ft_contract.ft_metadata.clone();

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
    signer_account_id: near_kit::AccountId,
    ft_contract_account_id: near_kit::AccountId,
    receiver_account_id: near_kit::AccountId,
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
            .wrap_err_with(|| format!("Failed to read msg from file: {:?}", &scope.file_path))?
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

            move |network_config| {
                let amount_ft = if let crate::types::ft_properties::FungibleTokenTransferAmount::ExactAmount(ft) = &ft_transfer_amount {
                    ft.to_ft_amount()
                } else {
                    crate::commands::tokens::send_ft::get_ft_balance_for_account(
                        network_config,
                        &signer_account_id,
                        &ft_contract_account_id,
                        near_kit::Finality::Final.into(),
                    )?
                };

                super::get_prepopulated_transaction(
                    network_config,
                    &ft_contract_account_id,
                    &receiver_account_id,
                    &signer_account_id,
                    &amount_ft,
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
        let receiver_account_id = previous_context.receiver_account_id.clone();
        let verbosity = previous_context.global_context.verbosity;

        move |outcome_view, network_config| {
            if outcome_view.is_success() {
                for action in outcome_view.transaction.actions.clone() {
                    if let near_kit::ActionView::FunctionCall { method_name: _, args, gas: _, deposit: _ } = action
                        && let Ok(args_bytes) = base64::engine::general_purpose::STANDARD.decode(&args)
                        && let Ok(ft_transfer_call) = serde_json::from_slice::<crate::types::ft_properties::FtTransferCall>(&args_bytes)
                            && let Ok(ft_balance) = crate::commands::tokens::send_ft::get_ft_balance_for_account(
                                network_config,
                                &signer_account_id,
                                &ft_contract_account_id,
                                near_kit::BlockReference::at_hash(outcome_view.receipts_outcome.last().expect("FT transfer call should have at least one receipt outcome, but none was received").block_hash)
                            ) {
                                let ft_transfer_amount = near_kit::FtAmount::new(
                                    ft_transfer_call.amount,
                                    ft_balance.decimals(),
                                    ft_balance.symbol(),
                                );
                                if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                                    tracing_indicatif::suspend_tracing_indicatif(|| eprintln!(
                                        "<{signer_account_id}> has successfully called ft_transfer_call, transferring {ft_transfer_amount} (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.\nRemaining balance: {ft_balance}",
                                    ));
                                }
                                return Ok(());
                            }
                }
                if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                    tracing_indicatif::suspend_tracing_indicatif(|| eprintln!(
                        "<{signer_account_id}> has successfully called ft_transfer_call (FT-contract: {ft_contract_account_id}) to <{receiver_account_id}>.",
                    ));
                }
            }
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
