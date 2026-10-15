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
            let verbosity = previous_context.global_context.verbosity;

            move |outcome_view, network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    for action in outcome_view.transaction.actions.clone() {
                        if let near_primitives::views::ActionView::FunctionCall { method_name: _, args, gas: _, deposit: _ } = action
                            && let Ok(mt_ft_transfer) = serde_json::from_slice::<crate::types::mt_ft_properties::MtFtTransfer>(&args)
                                && let Ok(mt_ft_balance_str) = tokio::runtime::Runtime::new()
                                    .expect("Failed to create a new tokio runtime")
                                    .block_on(
                                        get_mt_ft_balance(
                                            network_config,
                                            &mt_contract,
                                            &token_id,
                                            &signer_account_id,
                                            near_primitives::types::BlockId::Hash(outcome_view.receipts_outcome.last().expect("MT-FT transfer should have at least one receipt outcome, but none was received").block_hash).into()
                                        )
                                    )
                                {
                                    let ft_transfer_amount = crate::types::ft_properties::FungibleToken::from_params_ft(
                                        mt_ft_transfer.amount,
                                        mt_ft_metadata.decimals,
                                        mt_ft_metadata.symbol.clone()
                                    );
                                    let remaining_balance = if let Ok(amount) = mt_ft_balance_str.parse::<u128>() {
                                        crate::types::ft_properties::FungibleToken::from_params_ft(
                                            amount,
                                            mt_ft_metadata.decimals,
                                            mt_ft_metadata.symbol.clone()
                                        ).to_string()
                                    } else {
                                        "N/A".to_string()
                                    };
                                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                                        tracing_indicatif::suspend_tracing_indicatif(|| eprintln!(
                                            "<{signer_account_id}> has successfully transferred {ft_transfer_amount} (MT-FT-contract: {mt_contract}, token ID: {token_id}) to <{receiver_account_id}>.\nRemaining balance: {remaining_balance}",
                                        ));
                                    }
                                    return Ok(());
                                }
                    }
                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe = verbosity {
                        tracing_indicatif::suspend_tracing_indicatif(|| eprintln!(
                            "<{signer_account_id}> has successfully transferred fungible tokens (MT-FT-contract: {mt_contract}, token ID: {token_id}) to <{receiver_account_id}>.",
                        ));
                    }
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
