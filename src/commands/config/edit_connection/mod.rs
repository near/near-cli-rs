use std::str::FromStr;

use inquire::{Select, Text};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = EditConnectionContext)]
pub struct EditConnection {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the network connection name?
    connection_name: String,
    #[interactive_clap(subargs)]
    parameter: Parameter,
}

#[derive(Debug, Clone)]
pub struct EditConnectionContext {
    global_context: crate::GlobalContext,
    connection_name: String,
    network_config: crate::config::NetworkConfig,
}

impl EditConnectionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<EditConnection as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context
            .config
            .network_connection
            .get(&scope.connection_name)
            .unwrap_or_else(|| {
                panic!(
                    "Network connection \"{}\" not found",
                    &scope.connection_name
                )
            })
            .clone();

        Ok(Self {
            global_context: previous_context,
            connection_name: scope.connection_name.clone(),
            network_config,
        })
    }
}

impl EditConnection {
    fn input_connection_name(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        crate::common::input_network_name(&context.config, &[])
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = EditConnectionContext)]
#[interactive_clap(output_context = ParameterContext)]
pub struct Parameter {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Which parameter do you want to update?
    key: String,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter a new value for this parameter:
    value: String,
}

#[derive(Debug, Clone)]
pub struct ParameterContext;

impl ParameterContext {
    pub fn from_previous_context(
        previous_context: EditConnectionContext,
        scope: &<Parameter as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.global_context.config;
        if let Some(network_config) = config
            .network_connection
            .get_mut(&previous_context.connection_name)
        {
            if scope.key.contains("network_name") {
                network_config.network_name.clone_from(&scope.value)
            } else if scope.key.contains("rpc_url") {
                network_config.rpc_url = scope.value.clone().parse()?;
            } else if scope.key.contains("rpc_api_key") {
                network_config.rpc_api_key = if &scope.value == "null" {
                    None
                } else {
                    Some(crate::types::api_key::ApiKey::from_str(&scope.value)?)
                };
            } else if scope.key.contains("wallet_url") {
                network_config.wallet_url = scope.value.clone().parse()?;
            } else if scope.key.contains("explorer_transaction_url") {
                network_config.explorer_transaction_url = scope.value.clone().parse()?;
            } else if scope.key.contains("linkdrop_account_id") {
                network_config.linkdrop_account_id = if &scope.value == "null" {
                    None
                } else {
                    Some(near_primitives::types::AccountId::from_str(&scope.value)?)
                };
            } else if scope.key.contains("near_social_db_contract_account_id") {
                network_config.near_social_db_contract_account_id = if &scope.value == "null" {
                    None
                } else {
                    Some(near_primitives::types::AccountId::from_str(&scope.value)?)
                };
            } else if scope.key.contains("faucet_url") {
                network_config.faucet_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.clone().parse()?)
                };
            } else if scope.key.contains("meta_transaction_relayer_url") {
                network_config.meta_transaction_relayer_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.clone().parse()?)
                };
            } else if scope.key.contains("fastnear_url") {
                network_config.fastnear_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.clone().parse()?)
                };
            } else if scope.key.contains("staking_pools_factory_account_id") {
                network_config.staking_pools_factory_account_id = if &scope.value == "null" {
                    None
                } else {
                    Some(near_primitives::types::AccountId::from_str(&scope.value)?)
                };
            } else if scope.key.contains("coingecko_url") {
                network_config.coingecko_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.clone().parse()?)
                };
            } else {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                    "Parameter <{}> not found",
                    &scope.key
                ));
            }
        } else {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Network connection \"{}\" not found",
                &previous_context.connection_name
            ));
        };

        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Parameter <{}> successfully updated for Network connection \"{}\"",
            &scope.key, &previous_context.connection_name
        );
        Ok(Self)
    }
}

impl Parameter {
    fn input_key(context: &EditConnectionContext) -> color_eyre::eyre::Result<Option<String>> {
        let variants = context.network_config.get_fields()?;

        let select_submit =
            Select::new("Whitch of the parametrs do you want to change?", variants).prompt();
        match select_submit {
            Ok(value) => Ok(Some(
                value.split_once(':').expect("Internal error").0.to_string(),
            )),
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn input_value(
        _context: &EditConnectionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let value: String =
            Text::new("Enter a new value for this parameter (If you want to remove this optional parameter, leave \"null\"):")
                .with_initial_value("null")
                .prompt()?;
        Ok(Some(value))
    }
}
