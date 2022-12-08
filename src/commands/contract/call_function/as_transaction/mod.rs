use inquire::Text;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct CallFunctionProperties {
    ///What is the contract account ID?
    contract_account_id: crate::types::account_id::AccountId,
    ///What is the name of the function?
    function_name: String,
    #[interactive_clap(value_enum)]
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
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: self
                .prepaid_gas
                .attached_deposit
                .sign_as
                .signer_account_id
                .clone()
                .into(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self.contract_account_id.clone().into(),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name: self.function_name.clone(),
                    args: function_args,
                    gas: self.prepaid_gas.gas.clone().inner,
                    deposit: self
                        .prepaid_gas
                        .attached_deposit
                        .deposit
                        .clone()
                        .to_yoctonear(),
                },
            )],
        };
        match crate::transaction_signature_options::sign_with(
            self.prepaid_gas
                .attached_deposit
                .sign_as
                .network_config
                .clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => crate::common::print_transaction_status(
                transaction_info,
                self.prepaid_gas
                    .attached_deposit
                    .sign_as
                    .network_config
                    .get_network_config(config),
            ),
            None => Ok(()),
        }
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
            match crate::common::NearGas::from_str(
                &Text::new("Enter gas for function call")
                    .with_initial_value("100 TeraGas")
                    .prompt()?,
            ) {
                Ok(input_gas) => {
                    let crate::common::NearGas { inner: num } = input_gas;
                    let gas = num;
                    if gas <= 300000000000000 {
                        break gas;
                    } else {
                        println!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(gas.into())
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
        match crate::common::NearBalance::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_value("0 NEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(deposit),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
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
