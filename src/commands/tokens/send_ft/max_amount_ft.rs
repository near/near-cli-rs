#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendFtCommandContext)]
#[interactive_clap(output_context = MaxAmountFtContext)]
pub struct MaxAmountFt {
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: super::amount_ft::PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct MaxAmountFtContext(super::amount_ft::AmountFtContext);

impl MaxAmountFtContext {
    pub fn from_previous_context(
        previous_context: super::SendFtCommandContext,
        _scope: &<MaxAmountFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(super::amount_ft::AmountFtContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract_account_id: previous_context.ft_contract_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            transfer_amount_option: super::TransferAmountFtDiscriminants::MaxAmount,
            amount_ft: None,
        }))
    }
}

impl From<MaxAmountFtContext> for super::amount_ft::AmountFtContext {
    fn from(item: MaxAmountFtContext) -> Self {
        item.0
    }
}
