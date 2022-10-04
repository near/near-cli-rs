use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionProperties {
    ///What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    ///What is the name of the function?
    function_name: String,
    #[interactive_clap(subcommand)]
    function_args: super::call_function_args::CallFunctionArgs,
}

impl CallFunctionProperties {
    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let function_call_action = FunctionCallAction {
            contract_account_id: Some(self.contract_account_id.clone()),
            function_name: self.function_name.clone(),
            ..Default::default()
        };
        self.function_args
            .process(config, Some(function_call_action))
            .await
    }
}

#[derive(Debug, Clone, Default)]
pub struct FunctionCallAction {
    pub contract_account_id: Option<crate::types::account_id::AccountId>,
    pub function_name: String,
    pub function_args: String,
    pub gas: crate::common::NearGas,
    pub deposit: crate::common::NearBalance,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    ///Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    ///Enter deposit for a function call
    attached_deposit: Deposit,
}

impl PrepaidGas {
    fn input_gas(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearGas> {
        println!();
        let gas: u64 = loop {
            let input_gas: crate::common::NearGas = Input::new()
                .with_prompt("Enter gas for function call")
                .with_initial_text("100 TeraGas")
                .interact_text()?;
            let gas: u64 = match input_gas {
                crate::common::NearGas { inner: num } => num,
            };
            if gas <= 300000000000000 {
                break gas;
            } else {
                println!("You need to enter a value of no more than 300 TERAGAS")
            }
        };
        Ok(gas.into())
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        function_call_action: FunctionCallAction,
    ) -> crate::CliResult {
        let function_call_action = FunctionCallAction {
            gas: self.gas.clone(),
            ..function_call_action
        };
        self.attached_deposit
            .process(config, function_call_action)
            .await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    ///Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    ///What is the signer account ID?
    sign_as: SignerAccountId,
}

impl Deposit {
    fn input_deposit(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0 NEAR")
            .interact_text()?;
        Ok(deposit)
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        function_call_action: FunctionCallAction,
    ) -> crate::CliResult {
        let function_call_action = FunctionCallAction {
            deposit: self.deposit.clone(),
            ..function_call_action
        };
        self.sign_as.process(config, function_call_action).await
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SignerAccountId {
    ///What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub async fn process(
        &self,
        config: crate::config::Config,
        function_call_action: FunctionCallAction,
    ) -> crate::CliResult {
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self.signer_account_id.clone().into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: function_call_action
                .contract_account_id
                .clone()
                .expect("Impossible to get contract_account_id!")
                .into(),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: function_call_action.function_name.clone(),
                    args: function_call_action.function_args.clone().into_bytes(),
                    gas: function_call_action.gas.clone().inner,
                    deposit: function_call_action.deposit.clone().to_yoctonear(),
                },
            )],
        };

        match self.network_config.get_sign_option() {
            crate::transaction_signature_options::SignWith::SignWithPlaintextPrivateKey(
                sign_private_key,
            ) => {
                sign_private_key
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network_config.get_network_config(config),
                    )
                    .await
            }
            crate::transaction_signature_options::SignWith::SignWithKeychain(sign_keychain) => {
                sign_keychain
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network_config.get_network_config(config.clone()),
                        config.credentials_home_dir,
                    )
                    .await
            }
            #[cfg(feature = "ledger")]
            crate::transaction_signature_options::SignWith::SignWithLedger(sign_ledger) => {
                sign_ledger
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network_config.get_network_config(config),
                    )
                    .await
            }
        }
    }
}
