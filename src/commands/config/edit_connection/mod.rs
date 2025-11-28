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
        let Some(network_config) = config
            .network_connection
            .get_mut(&previous_context.connection_name)
        else {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "Network connection \"{}\" not found",
                &previous_context.connection_name
            ));
        };
        match scope.key.as_str() {
            "network_name" => network_config.network_name.clone_from(&scope.value),
            "rpc_url" => network_config.rpc_url = scope.value.parse()?,
            "rpc_api_key" => {
                network_config.rpc_api_key = if scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "wallet_url" => {
                network_config.wallet_url = scope.value.parse()?;
            }
            "explorer_transaction_url" => {
                network_config.explorer_transaction_url = scope.value.parse()?;
            }
            "linkdrop_account_id" => {
                network_config.linkdrop_account_id = if scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "near_social_db_contract_account_id" => {
                network_config.near_social_db_contract_account_id = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "faucet_url" => {
                network_config.faucet_url = if scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "meta_transaction_relayer_url" => {
                network_config.meta_transaction_relayer_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "fastnear_url" => {
                network_config.fastnear_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "staking_pools_factory_account_id" => {
                network_config.staking_pools_factory_account_id = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            "coingecko_url" => {
                network_config.coingecko_url = if &scope.value == "null" {
                    None
                } else {
                    Some(scope.value.parse()?)
                };
            }
            _ => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                    "Configuration key <{}> not found",
                    &scope.key
                ));
            }
        }

        eprintln!();
        config.write_config_toml()?;
        eprintln!(
            "Network connection \"{}\" was successfully updated with the new value for <{}>",
            &previous_context.connection_name, &scope.key
        );
        Ok(Self)
    }
}

impl Parameter {
    fn input_key(context: &EditConnectionContext) -> color_eyre::eyre::Result<Option<String>> {
        let variants = context
            .network_config
            .get_fields()?
            .iter()
            .map(|s| (s.clone(), s.clone(), ""))
            .collect::<Vec<_>>();

        match cliclack::select("Which setting do you want to change?")
            .items(&variants)
            .interact()
        {
            Ok(value) => Ok(Some(
                value.split_once(':').expect("Internal error").0.to_string(),
            )),
            Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn input_value(
        _context: &EditConnectionContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        match cliclack::input("Enter a new value for this parameter (if you want to remove an optional parameter, use \"null\"):")
            .default_input("null")
            .interact() {
                Ok(value) => Ok(Some(value)),
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => Ok(None),
                Err(err) => Err(err.into()),
            }
    }
}
