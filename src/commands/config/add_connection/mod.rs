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
    fastnear_url: Option<crate::types::url::Url>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    staking_pools_factory_account_id: Option<crate::types::account_id::AccountId>,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    coingecko_url: Option<crate::types::url::Url>,
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
                fastnear_url: scope
                    .fastnear_url
                    .clone()
                    .map(|fastnear_url| fastnear_url.into()),
                staking_pools_factory_account_id: scope
                    .staking_pools_factory_account_id
                    .clone()
                    .map(|staking_pools_factory_account_id| {
                        staking_pools_factory_account_id.into()
                    }),
                coingecko_url: scope
                    .coingecko_url
                    .clone()
                    .map(|coingecko_url| coingecko_url.into()),
            },
        );
        eprintln!();
        config.write_config_toml()?;
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
        let confirm_yes = "Yes, the RPC endpoint requires API key";
        let confirm_no = "No, the RPC endpoint does not require API key";
        if cliclack::select("Do you want to input an API key?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let api_key: crate::types::api_key::ApiKey =
                cliclack::input("Enter an API key:").interact()?;
            Ok(Some(api_key))
        } else {
            Ok(None)
        }
    }

    fn input_linkdrop_account_id(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let confirm_yes =
            "Yes, and I want to enter the name of the account hosting the program \"linkdrop\"";
        let confirm_no = "I dont know";
        if cliclack::select("Is there a \"linkdrop\" program on this network?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let account_id: crate::types::account_id::AccountId =
                cliclack::input(
                    "What is the name of the account that hosts the \"linkdrop\" program? (e.g. on mainnet it is near, and on testnet it is testnet)"
                )
                .interact()?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    fn input_near_social_db_contract_account_id(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let confirm_yes = "Yes, and I want to enter the NEAR Social DB contract account ID";
        let confirm_no = "No, I don't want to enter the NEAR Social DB contract account ID";
        if cliclack::select(
            "Do you want to enter the NEAR Social DB contract account ID on this network?",
        )
        .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
        .interact()?
        {
            let account_id: crate::types::account_id::AccountId =
                cliclack::input(
                    "What is the name of the NEAR Social DB contract account ID (e.g. on mainnet it is social.near)"
                )
                .interact()?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    fn input_faucet_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the URL of the faucet";
        let confirm_no = "No, I don't want to enter the faucet URL";
        if cliclack::select("Do you want to enter the faucet URL?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let faucet_url: crate::types::url::Url =
                cliclack::input("What is the faucet url?").interact()?;
            Ok(Some(faucet_url))
        } else {
            Ok(None)
        }
    }

    fn input_meta_transaction_relayer_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the URL of the relayer";
        let confirm_no = "No, I don't want to enter the relayer URL";
        if cliclack::select("Do you want to enter the meta transaction relayer URL?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let meta_transaction_relayer_url: crate::types::url::Url =
                cliclack::input("What is the relayer url?").interact()?;
            Ok(Some(meta_transaction_relayer_url))
        } else {
            Ok(None)
        }
    }

    fn input_fastnear_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the fastnear API url";
        let confirm_no = "No, I don't want to enter the fastnear API url";
        if cliclack::select("Do you want to enter the fastnear API url?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let stake_delegators_api: crate::types::url::Url =
                cliclack::input("What is the fastnear API url?").interact()?;
            Ok(Some(stake_delegators_api))
        } else {
            Ok(None)
        }
    }

    fn input_staking_pools_factory_account_id(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        let confirm_yes = "Yes, I want to enter the staking pools factory account ID";
        let confirm_no = "No, I don't want to enter the staking pools factory account ID";
        if cliclack::select("Do you want to enter the staking pools factory account ID?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let account_id: crate::types::account_id::AccountId =
                cliclack::input("What is the staking pools factory account ID?").interact()?;
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    fn input_coingecko_url(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::url::Url>> {
        let confirm_yes = "Yes, I want to enter the coingecko API url";
        let confirm_no = "No, I don't want to enter the coingecko API url";
        if cliclack::select("Do you want to enter the coingecko API url?")
            .items(&[(true, confirm_yes, ""), (false, confirm_no, "")])
            .interact()?
        {
            let coingecko_api: crate::types::url::Url =
                cliclack::input("What is the coingecko API url?")
                    .default_input("https://api.coingecko.com/")
                    .interact()?;
            Ok(Some(coingecko_api))
        } else {
            Ok(None)
        }
    }
}
