use dialoguer::Input;

#[derive(Debug, Clone, Default)]
pub struct FunctionCallAction {
    pub contract_account_id: Option<crate::types::account_id::AccountId>,
    pub function_name: String,
    pub function_args: Vec<u8>,
    pub gas: crate::common::NearGas,
    pub deposit: crate::common::NearBalance,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionProperties {
    ///What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    ///What is the name of the function?
    function_name: String,
    #[interactive_clap(arg_enum)]
    #[interactive_clap(skip_default_input_arg)]
    ///How do you want to pass the function call arguments?
    function_args_type: super::call_function_args_type::FunctionArgsType,
    ///Enter the arguments to this function or the path to the arguments file
    function_args: String,
    #[interactive_clap(named_arg)]
    ///Enter gas for function call
    prepaid_gas: super::as_transaction::PrepaidGas,
}

impl CallFunctionProperties {
    fn input_function_args_type(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<super::call_function_args_type::FunctionArgsType> {
        super::call_function_args_type::input_function_args_type()
    }

    pub async fn process(&self, config: crate::config::Config) -> crate::CliResult {
        let function_args = super::call_function_args_type::function_args(
            self.function_args.clone(),
            self.function_args_type.clone(),
        )?;
        let function_call_action = FunctionCallAction {
            contract_account_id: Some(self.contract_account_id.clone()),
            function_name: self.function_name.clone(),
            function_args,
            ..Default::default()
        };
        self.prepaid_gas.process(config, function_call_action).await
    }
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
            let crate::common::NearGas { inner: num } = input_gas;
            let gas = num;
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
                    args: function_call_action.function_args.clone(),
                    gas: function_call_action.gas.clone().inner,
                    deposit: function_call_action.deposit.clone().to_yoctonear(),
                },
            )],
        };
        crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config,
        )
        .await
    }
}
