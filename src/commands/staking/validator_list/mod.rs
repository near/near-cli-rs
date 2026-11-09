mod format_validator_stake;

use crate::common::{StakingPoolInfo, ValidatorStatus, get_validator_list};
use format_validator_stake::format_validator_stake;
use prettytable::Table;

fn format_validator_fee(validator: &StakingPoolInfo) -> String {
    if let Some(fee) = validator.fee {
        format!("{:>6.2} %", fee.numerator * 100 / fee.denominator)
    } else {
        format!("{:>6}", "N/A")
    }
}

fn format_validator_delegators(validator: &StakingPoolInfo) -> String {
    if let Some(delegator_count) = validator.delegators {
        format!("{delegator_count:>8}")
    } else {
        format!("{:>8}", "N/A")
    }
}

pub fn format_validator_status(validator: &StakingPoolInfo) -> &'static str {
    match validator.status {
        ValidatorStatus::Active => "Active",
        ValidatorStatus::Leaving => "Leaving",
        ValidatorStatus::Joining => "Joining",
        ValidatorStatus::Proposal => "Proposal",
    }
}

#[tracing::instrument(name = "View the list of validators for delegation ...", skip_all)]
fn print_validator_list(network_config: &crate::config::NetworkConfig) -> crate::CliResult {
    tracing::info!(target: "near_teach_me", "View the list of validators for delegation ...");

    let validators = get_validator_list(network_config)?;

    // Print validators as a table
    let mut table = Table::new();

    table.set_titles(
        prettytable::row![Fg=>"#", "Validator Id", "Status", "Fee", "Delegators", "Stake"],
    );
    table.extend(
        validators
            .into_iter()
            .enumerate()
            .map(|(index, validator)| {
                prettytable::row![
                    Fg->index + 1,
                    validator.validator_id,
                    format_validator_status(&validator),
                    format_validator_fee(&validator),
                    format_validator_delegators(&validator),
                    r->format_validator_stake(&validator),
                ]
            }),
    );
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();

    Ok(())
}

#[derive(Clone)]
pub struct ValidatorListContext(crate::network::NetworkContext);

impl ValidatorListContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        _scope: &<ValidatorList as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(print_validator_list);

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

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = ValidatorListContext)]
pub struct ValidatorList {
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}
