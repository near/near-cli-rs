use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

mod view_contract_code;
mod view_transaction_status;
mod view_account;


/// инструмент выбора to view
#[derive(Debug, Default, clap::Clap)]
pub struct CliViewQueryRequest {
    #[clap(subcommand)]
    query: Option<CliQueryRequest>,
}

#[derive(Debug)]
pub struct ViewQueryRequest {
    pub query: QueryRequest,
}

impl From<CliViewQueryRequest> for ViewQueryRequest {
    fn from(item: CliViewQueryRequest) -> Self {
        let query = match item.query {
            Some(cli_query_request) => QueryRequest::from(cli_query_request),
            None => QueryRequest::choose_query_request()
        };
        ViewQueryRequest{ query }
    }
}

impl ViewQueryRequest {
    pub async fn process(
        self,
    ) -> crate::CliResult {
        self.query
            .process()
            .await
    }
}

#[derive(Debug, clap::Clap)]
pub enum CliQueryRequest {
    /// View properties for an account
    AccountSummary(self::view_account::operation_mode::CliOperationMode),
    /// View a contract code
    ContractCode(self::view_contract_code::operation_mode::CliOperationMode),
    /// View a transaction status
    Transaction(self::view_transaction_status::operation_mode::CliOperationMode),
}

#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
pub enum QueryRequest {
    #[strum_discriminants(strum(message = "View properties for an account"))]
    AccountSummary(self::view_account::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a contract code"))]
    ContractCode(self::view_contract_code::operation_mode::OperationMode),
    #[strum_discriminants(strum(message = "View a transaction status"))]
    Transaction(self::view_transaction_status::operation_mode::OperationMode),
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
            CliQueryRequest::Transaction(cli_operation_mode) => {
                QueryRequest::Transaction(cli_operation_mode.into())
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
            .with_prompt(
                "Сhoose what you want to view"
            )
            .items(&requests)
            .default(0)
            .interact()
            .unwrap();
        let cli_request = match variants[selected_request] {
            QueryRequestDiscriminants::AccountSummary => CliQueryRequest::AccountSummary(Default::default()),
            QueryRequestDiscriminants::ContractCode => CliQueryRequest::ContractCode(Default::default()),
            QueryRequestDiscriminants::Transaction => CliQueryRequest::Transaction(Default::default()),
        };
        Self::from(cli_request)
    }

    pub async fn process(
        self,
    ) -> crate::CliResult {
        match self {
            QueryRequest::AccountSummary(operation_mode) => {
                operation_mode
                    .process()
                    .await
            }
            QueryRequest::ContractCode(operation_mode) => {
                operation_mode
                    .process()
                    .await
            }
            QueryRequest::Transaction(operation_mode) => {
                operation_mode
                    .process()
                    .await
            }
        }
    }
}
