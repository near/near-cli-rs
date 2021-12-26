use async_recursion::async_recursion;
use dialoguer::Input;

/// вызов CallFunction
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCallFunctionAction {
    method_name: Option<String>,
    args: Option<String>,
    #[clap(long = "prepaid-gas")]
    gas: Option<crate::common::NearGas>,
    #[clap(long = "attached-deposit")]
    deposit: Option<crate::common::NearBalance>,
    #[clap(subcommand)]
    next_action: Option<super::CliSkipNextAction>,
}

#[derive(Debug, Clone)]
pub struct CallFunctionAction {
    method_name: String,
    args: Vec<u8>,
    gas: near_primitives::types::Gas,
    deposit: near_primitives::types::Balance,
    next_action: Box<super::NextAction>,
}

impl interactive_clap::ToCli for CallFunctionAction {
    type CliVariant = CliCallFunctionAction;
}

impl CliCallFunctionAction {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let mut args = self
            .next_action
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        if let Some(gas) = &self.gas {
            args.push_front(gas.to_string());
            args.push_front("--prepaid-gas".to_owned())
        };
        if let Some(deposit) = &self.deposit {
            args.push_front(deposit.to_string());
            args.push_front("--attached-deposit".to_owned())
        };
        if let Some(function_args) = &self.args {
            args.push_front(function_args.to_owned());
        };
        if let Some(method_name) = &self.method_name {
            args.push_front(method_name.to_string());
        };
        args
    }
}

impl From<CallFunctionAction> for CliCallFunctionAction {
    fn from(call_function_action: CallFunctionAction) -> Self {
        Self {
            method_name: Some(call_function_action.method_name),
            args: Some(String::from_utf8(call_function_action.args).unwrap_or_default()),
            gas: Some(call_function_action.gas.into()),
            deposit: Some(crate::common::NearBalance::from_yoctonear(
                call_function_action.deposit,
            )),
            next_action: Some(super::CliSkipNextAction::Skip(super::CliSkipAction {
                sign_option: None,
            })),
        }
    }
}

impl CallFunctionAction {
    pub fn from_cli(
        optional_clap_variant: Option<<CallFunctionAction as interactive_clap::ToCli>::CliVariant>,
        context: crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<Self> {
        let method_name: String = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.method_name)
        {
            Some(cli_method_name) => cli_method_name,
            None => CallFunctionAction::input_method_name(&context)?,
        };
        let args: Vec<u8> = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.args)
        {
            Some(cli_args) => cli_args.into_bytes(),
            None => CallFunctionAction::input_args(&context)?,
        };
        let gas: near_primitives::types::Gas = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.gas)
        {
            Some(cli_gas) => match cli_gas {
                crate::common::NearGas { inner: num } => num,
            },
            None => CallFunctionAction::input_gas(&context)?,
        };
        let deposit: near_primitives::types::Balance = match optional_clap_variant
            .clone()
            .and_then(|clap_variant| clap_variant.deposit)
        {
            Some(cli_deposit) => cli_deposit.to_yoctonear(),
            None => CallFunctionAction::input_deposit(&context)?,
        };
        let skip_next_action: super::NextAction =
            match optional_clap_variant.and_then(|clap_variant| clap_variant.next_action) {
                Some(cli_skip_action) => {
                    super::NextAction::from_cli_skip_next_action(cli_skip_action, context)?
                }
                None => super::NextAction::choose_variant(context)?,
            };
        Ok(Self {
            method_name,
            args,
            gas,
            deposit,
            next_action: Box::new(skip_next_action),
        })
    }
}

impl CallFunctionAction {
    fn input_method_name(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<String> {
        println!();
        Ok(Input::new()
            .with_prompt("Enter a method name")
            .interact_text()?)
    }

    fn input_gas(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::Gas> {
        println!();
        let gas: u64 = loop {
            let input_gas: crate::common::NearGas = Input::new()
                .with_prompt("Enter a gas for function")
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
        Ok(gas)
    }

    fn input_args(_context: &crate::common::SignerContext) -> color_eyre::eyre::Result<Vec<u8>> {
        println!();
        let input: String = Input::new()
            .with_prompt("Enter args for function")
            .interact_text()?;
        Ok(input.into_bytes())
    }

    fn input_deposit(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<near_primitives::types::Balance> {
        println!();
        let deposit: crate::common::NearBalance = Input::new()
            .with_prompt(
                "Enter a deposit for function (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_text("0 NEAR")
            .interact_text()?;
        Ok(deposit.to_yoctonear())
    }

    #[async_recursion(?Send)]
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::FunctionCall(
            near_primitives::transaction::FunctionCallAction {
                method_name: self.method_name.clone(),
                args: self.args.clone(),
                gas: self.gas.clone(),
                deposit: self.deposit.clone(),
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match *self.next_action {
            super::NextAction::AddAction(select_action) => {
                select_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
            super::NextAction::Skip(skip_action) => {
                skip_action
                    .process(unsigned_transaction, network_connection_config)
                    .await
            }
        }
    }
}
