use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod view_validators;
mod view_proposals;

/// инструмент выбора to view
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliViewQueryRequest {
    #[clap(subcommand)]
    query: Option<CliQueryRequest>,
}

#[derive(Debug, Clone)]
pub struct ViewQueryRequest {
    pub query: QueryRequest,
}

impl CliViewQueryRequest {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        let args = self
            .query
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default();
        args
    }
}

impl From<ViewQueryRequest> for CliViewQueryRequest {
    fn from(view_query_request: ViewQueryRequest) -> Self {
        Self {
            query: Some(view_query_request.query.into()),
        }
    }
}

impl From<CliViewQueryRequest> for ViewQueryRequest {
    fn from(item: CliViewQueryRequest) -> Self {
        let query = match item.query {
            Some(cli_query_request) => QueryRequest::from(cli_query_request),
            None => QueryRequest::choose_query_request(),
        };
        ViewQueryRequest { query }
    }
}

impl ViewQueryRequest {
    pub async fn process(self) -> crate::CliResult {
        self.query.process().await
    }
}

#[derive(Debug, Clone, clap::Clap)]
pub enum CliQueryRequest {
    /// View properties for an account
    AccountSummary(self::view_validators::operation_mode::CliOperationMode),
    /// View a contract code
    ContractCode(self::view_proposals::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum QueryRequest {
    #[strum_discriminants(strum(message = "+View properties for an account"))]
    AccountSummary(self::view_validators::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a contract code"))]
    ContractCode(self::view_proposals::operation_mode::OperationMode),
}

impl CliQueryRequest {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::AccountSummary(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("account-summary".to_owned());
                args
            }
            Self::ContractCode(subcommand) => {
                let mut args = subcommand.to_cli_args();
                args.push_front("contract-code".to_owned());
                args
            }
        }
    }
}

impl From<QueryRequest> for CliQueryRequest {
    fn from(query_request: QueryRequest) -> Self {
        match query_request {
            QueryRequest::AccountSummary(operation_mode) => {
                Self::AccountSummary(operation_mode.into())
            }
            QueryRequest::ContractCode(operation_mode) => Self::ContractCode(operation_mode.into()),
        }
    }
}

impl From<CliQueryRequest> for QueryRequest {
    fn from(item: CliQueryRequest) -> Self {
        match item {
            CliQueryRequest::AccountSummary(cli_operation_mode) => {
                QueryRequest::AccountSummary(cli_operation_mode.into())
            }
            CliQueryRequest::ContractCode(cli_operation_mode) => {
                QueryRequest::ContractCode(cli_operation_mode.into())
            }
        }
    }
}

impl QueryRequest {
    fn choose_query_request() -> Self {
        println!();
        let variants = QueryRequestDiscriminants::iter().collect::<Vec<_>>();
        let requests = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_request = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Сhoose what you want to view")
            .items(&requests)
            .default(0)
            .interact()
            .unwrap();
        let cli_request = match variants[selected_request] {
            QueryRequestDiscriminants::AccountSummary => {
                CliQueryRequest::AccountSummary(Default::default())
            }
            QueryRequestDiscriminants::ContractCode => {
                CliQueryRequest::ContractCode(Default::default())
            }
        };
        Self::from(cli_request)
    }

    pub async fn process(self) -> crate::CliResult {
        match self {
            QueryRequest::AccountSummary(operation_mode) => operation_mode.process().await,
            QueryRequest::ContractCode(operation_mode) => operation_mode.process().await,
        }
    }
}
