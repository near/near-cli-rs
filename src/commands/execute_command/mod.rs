use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod change_method;
mod view_method;

/// выбор метода для выполнения
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliOptionMethod {
    #[clap(subcommand)]
    method: Option<CliMethod>,
}

#[derive(Debug, Clone)]
pub struct OptionMethod {
    method: Method,
}

impl CliOptionMethod {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.method
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<OptionMethod> for CliOptionMethod {
    fn from(option_method: OptionMethod) -> Self {
        Self {
            method: Some(option_method.method.into()),
        }
    }
}

impl OptionMethod {
    pub fn from(item: CliOptionMethod) -> color_eyre::eyre::Result<Self> {
        let method = match item.method {
            Some(cli_method) => Method::from(cli_method)?,
            None => Method::choose_method()?,
        };
        Ok(Self { method })
    }
}

impl OptionMethod {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.method.process(prepopulated_unsigned_transaction).await
    }
}

#[derive(Debug, Clone, clap::Clap)]
enum CliMethod {
    /// Specify a change method
    ChangeMethod(self::change_method::operation_mode::CliOperationMode),
    /// Specify a view method
    ViewMethod(self::view_method::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
enum Method {
    #[strum_discriminants(strum(message = "Execute a changing method (construct a transaction with a function call)"))]
    ChangeMethod(self::change_method::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "Execute a viewing method (read-only call, which does not require a transaction)"))]
    ViewMethod(self::view_method::operation_mode::OperationMode),
}

impl CliMethod {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::ChangeMethod(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("change-method".to_owned());
                command
            }
            Self::ViewMethod(subcommand) => {
                let mut command = subcommand.to_cli_args();
                command.push_front("view-method".to_owned());
                command
            }
        }
    }
}

impl From<Method> for CliMethod {
    fn from(method: Method) -> Self {
        match method {
            Method::ChangeMethod(operation_mode) => Self::ChangeMethod(operation_mode.into()),
            Method::ViewMethod(operation_mode) => Self::ViewMethod(operation_mode.into()),
        }
    }
}

impl Method {
    fn from(item: CliMethod) -> color_eyre::eyre::Result<Self> {
        match item {
            CliMethod::ChangeMethod(cli_operation_mode) => Ok(Method::ChangeMethod(
                self::change_method::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
            CliMethod::ViewMethod(cli_operation_mode) => Ok(Method::ViewMethod(
                self::view_method::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
        }
    }
}

impl Method {
    fn choose_method() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = MethodDiscriminants::iter().collect::<Vec<_>>();
        let methods = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_method = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your method")
            .items(&methods)
            .default(0)
            .interact()
            .unwrap();
        let cli_method = match variants[selected_method] {
            MethodDiscriminants::ChangeMethod => CliMethod::ChangeMethod(Default::default()),
            MethodDiscriminants::ViewMethod => CliMethod::ViewMethod(Default::default()),
        };
        Self::from(cli_method)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::ChangeMethod(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
            Self::ViewMethod(operation_mode) => operation_mode.process().await,
        }
    }
}
