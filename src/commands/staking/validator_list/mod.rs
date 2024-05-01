use prettytable::Table;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ValidatorListContext)]
pub struct ValidatorList {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct ValidatorListContext(crate::network::NetworkContext);

impl ValidatorListContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<ValidatorList as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(display_validators_info);
        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![],
            on_after_getting_network_callback,
        }))
    }
}

impl From<ValidatorListContext> for crate::network::NetworkContext {
    fn from(item: ValidatorListContext) -> Self {
        item.0
    }
}

#[tracing::instrument(name = "View the list of validators for delegation ...", skip_all)]
fn display_validators_info(network_config: &crate::config::NetworkConfig) -> crate::CliResult {
    let mut table = Table::new();
    table.set_titles(prettytable::row![Fg=>"#", "Validator Id", "Fee", "Delegators", "Stake"]);

    for (index, validator) in crate::common::get_validator_list(network_config)?
        .into_iter()
        .enumerate()
    {
        let fee = if let Some(fee) = validator.fee {
            format!("{:>6.2} %", fee.numerator * 100 / fee.denominator)
        } else {
            format!("{:>6}", "N/A")
        };
        let delegators = if let Some(num) = validator.delegators {
            format!("{:>8}", num)
        } else {
            format!("{:>8}", "N/A")
        };
        table.add_row(prettytable::row![
            Fg->index + 1,
            validator.validator_id,
            fee,
            delegators,
            near_token::NearToken::from_yoctonear(validator.stake),
        ]);
    }
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();
    let validators_url: url::Url = network_config.wallet_url.join("staking/validators")?;
    eprintln!(
        "This is not a complete list of validators. To see the full list of validators visit Explorer:\n{}\n",
        &validators_url.as_str()
    );
    Ok(())
}
