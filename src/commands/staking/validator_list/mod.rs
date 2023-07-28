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
            on_after_getting_network_callback,
        }))
    }
}

impl From<ValidatorListContext> for crate::network::NetworkContext {
    fn from(item: ValidatorListContext) -> Self {
        item.0
    }
}

fn display_validators_info(network_config: &crate::config::NetworkConfig) -> crate::CliResult {
    let mut table = Table::new();
    table.set_titles(prettytable::row![Fg=>"#", "Validator Id", "Fee", "Delegators", "Stake"]);

    for (index, validator) in crate::common::get_validator_list(network_config)?
        .into_iter()
        .enumerate()
    {
        table.add_row(prettytable::row![
            Fg->index + 1,
            validator.validator_id,
            format!("{:>6.2} %", validator.fee.numerator * 100 / validator.fee.denominator),
            validator.delegators,
            crate::common::NearBalance::from_yoctonear(validator.stake),
        ]);
    }
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();
    Ok(())
}
