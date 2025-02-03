use color_eyre::eyre::ContextCompat;
use inquire::{CustomType, Text};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendFtCommandContext)]
#[interactive_clap(output_context = AmountFtContext)]
pub struct AmountFt {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount FT to transfer:
    ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
    #[interactive_clap(skip_default_input_arg)]
    /// Enter a memo for transfer (optional):
    memo: Option<String>,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: super::preparation_ft_transfer::PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct AmountFtContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub ft_contract_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub ft_transfer_amount: crate::types::ft_properties::FungibleTokenTransferAmount,
    pub memo: Option<String>,
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
            memo: scope.memo.as_ref().and_then(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }),
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

    fn input_memo(
        _context: &super::SendFtCommandContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let input = Text::new("Enter a memo for transfer (optional):").prompt()?;
        Ok(if input.trim().is_empty() {
            None
        } else {
            Some(input)
        })
    }
}
