use inquire::{CustomType, Select};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = AddNetworkConnectionContext)]
pub struct AddNetworkConnection {
    #[interactive_clap(long)]
    /// What is the NEAR network? (e.g. mainnet, testnet, shardnet)
    network_name: String,
    #[interactive_clap(long)]
    /// What is the connection name? (e.g. pagoda-mainnet)
    connection_name: String,
    #[interactive_clap(long)]
    /// What is the RPC endpoint?
    rpc_url: crate::types::url::Url,
    #[interactive_clap(long)]
    /// What is the wallet endpoint?
    wallet_url: crate::types::url::Url,
    #[interactive_clap(long)]
    /// What is the transaction explorer endpoint?
    explorer_transaction_url: crate::types::url::Url,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    rpc_api_key: Option<crate::types::api_key::ApiKey>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    linkdrop_account_id: Option<crate::types::account_id::AccountId>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    near_social_db_contract_account_id: Option<crate::types::account_id::AccountId>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    faucet_url: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    meta_transaction_relayer_url: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    stake_delegators_api: Option<String>,
    #[interactive_clap(long)]
    staking_pools_factory_account_id: crate::types::account_id::AccountId,
}

#[derive(Debug, Clone)]
pub struct AddNetworkConnectionContext;

impl AddNetworkConnectionContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<AddNetworkConnection as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let mut config = previous_context.config;
        config.network_connection.insert(
            scope.connection_name.clone(),
            crate::config::NetworkConfig {
                network_name: scope.network_name.clone(),
                rpc_url: scope.rpc_url.clone().into(),
                wallet_url: scope.wallet_url.clone().into(),
                explorer_transaction_url: scope.explorer_transaction_url.0.clone(),
                rpc_api_key: scope.rpc_api_key.clone(),
                linkdrop_account_id: scope
                    .linkdrop_account_id
                    .clone()
                    .map(|linkdrop_account_id| linkdrop_account_id.into()),
                near_social_db_contract_account_id: scope
                    .near_social_db_contract_account_id
                    .clone()
                    .map(|near_social_db_contract_account_id| {
                        near_social_db_contract_account_id.into()
                    }),
                faucet_url: scope.faucet_url.clone().map(|faucet_url| faucet_url.into()),
                meta_transaction_relayer_url: scope
                    .meta_transaction_relayer_url
                    .clone()
                    .map(|meta_transaction_relayer_url| meta_transaction_relayer_url.into()),
                stake_delegators_api: scope.stake_delegators_api.clone(),
                staking_pools_factory_account_id: scope
                    .staking_pools_factory_account_id
                    .clone()
                    .into(),
            },
        );
        eprintln!();
        crate::common::write_config_toml(config)?;
        eprintln!(
            "Network connection \"{}\" was successfully added to config.toml",
            &scope.connection_name
        );
        Ok(Self)
    }
}

impl AddNetworkConnection {
    fn input_rpc_api_key(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::api_key::ApiKey>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, the RPC endpoint requires API key")]
            Yes,
            #[strum(to_string = "No, the RPC endpoint does not require API key")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to input an API key?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let api_key: crate::types::api_key::ApiKey =
                CustomType::new("Enter an API key").prompt()?;
            Ok(Some(api_key))
        } else {
            Ok(None)
        }
    }

    fn input_linkdrop_account_id(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, and I want to enter the name of the account hosting the program \"linkdrop\""
            )]
            Yes,
            #[strum(to_string = "I dont know")]
            No,
        }
        let select_choose_input = Select::new(
            "Is there a \"linkdrop\" program on this network?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the name of the account that hosts the \"linkdrop\" program? (e.g. on mainnet it is near, and on testnet it is testnet)").prompt()?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    fn input_near_social_db_contract_account_id(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, and I want to enter the NEAR Social DB contract account ID")]
            Yes,
            #[strum(
                to_string = "No, I don't want to enter the NEAR Social DB contract account ID"
            )]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter the NEAR Social DB contract account ID on this network?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let account_id: crate::types::account_id::AccountId =
            CustomType::new("What is the name of the NEAR Social DB contract account ID (e.g. on mainnet it is social.near)").prompt()?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    fn input_faucet_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter the URL of the faucet")]
            Yes,
            #[strum(to_string = "No, I don't want to enter the faucet URL")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter the faucet URL?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let faucet_url: crate::types::url::Url =
                CustomType::new("What is the faucet url?").prompt()?;
            Ok(Some(faucet_url))
        } else {
            Ok(None)
        }
    }

    fn input_meta_transaction_relayer_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter the URL of the relayer")]
            Yes,
            #[strum(to_string = "No, I don't want to enter the relayer URL")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter the meta transaction relayer URL?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let meta_transaction_relayer_url: crate::types::url::Url =
                CustomType::new("What is the relayer url?").prompt()?;
            Ok(Some(meta_transaction_relayer_url))
        } else {
            Ok(None)
        }
    }

    fn input_stake_delegators_api(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        eprintln!();
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(to_string = "Yes, I want to enter the stake delegators API")]
            Yes,
            #[strum(to_string = "No, I don't want to enter the stake delegators API")]
            No,
        }
        let select_choose_input = Select::new(
            "Do you want to enter the stake delegators API?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let stake_delegators_api: String =
                CustomType::new("What is the stake delegators API?").prompt()?;
            Ok(Some(stake_delegators_api))
        } else {
            Ok(None)
        }
    }
}
