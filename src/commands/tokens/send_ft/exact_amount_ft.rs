use color_eyre::eyre::ContextCompat;
use inquire::CustomType;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::SendFtCommandContext)]
#[interactive_clap(output_context = ExactAmountFtContext)]
pub struct ExactAmountFt {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter an amount FT to transfer:
    amount_ft: crate::types::ft_properties::FungibleToken,
    #[interactive_clap(named_arg)]
    /// Enter gas for function call
    prepaid_gas: super::transaction_formation::PrepaidGas,
}

#[derive(Debug, Clone)]
pub struct ExactAmountFtContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub ft_contract_account_id: near_primitives::types::AccountId,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub transfer_amount_option: super::TransferAmountFtDiscriminants,
    pub amount_ft: Option<crate::types::ft_properties::FungibleToken>,
}

impl ExactAmountFtContext {
    pub fn from_previous_context(
        previous_context: super::SendFtCommandContext,
        scope: &<ExactAmountFt as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
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

        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            ft_contract_account_id: previous_context.ft_contract_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            transfer_amount_option: super::TransferAmountFtDiscriminants::ExactAmount,
            amount_ft: Some(scope.amount_ft.normalize(&ft_metadata)?),
        })
    }
}

impl ExactAmountFt {
    fn input_amount_ft(
        context: &super::SendFtCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::ft_properties::FungibleToken>> {
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
            CustomType::<crate::types::ft_properties::FungibleToken>::new(&format!(
                "Enter an FT amount to transfer (example: 10 {symbol} or 0.5 {symbol}):",
                symbol = ft_metadata.symbol
            ))
            .with_validator(move |ft: &crate::types::ft_properties::FungibleToken| {
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
