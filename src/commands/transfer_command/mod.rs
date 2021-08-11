use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumDiscriminants, EnumIter, EnumMessage, IntoEnumIterator};

pub mod operation_mode;
mod receiver;
mod sender;
pub mod transfer_near_tokens_type;

/// инструмент выбора переводимой валюты
#[derive(Debug, Default, Clone, clap::Clap)]
#[clap(
    setting(clap::AppSettings::ColoredHelp),
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::VersionlessSubcommands)
)]
pub struct CliCurrency {
    #[clap(subcommand)]
    currency_selection: Option<CliCurrencySelection>,
}

#[derive(Debug, Clone)]
pub struct Currency {
    currency_selection: CurrencySelection,
}

impl CliCurrency {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        self.currency_selection
            .as_ref()
            .map(|subcommand| subcommand.to_cli_args())
            .unwrap_or_default()
    }
}

impl From<Currency> for CliCurrency {
    fn from(currency: Currency) -> Self {
        Self {
            currency_selection: Some(CliCurrencySelection::from(currency.currency_selection)),
        }
    }
}

impl Currency {
    pub fn from(item: CliCurrency) -> color_eyre::eyre::Result<Self> {
        let currency_selection = match item.currency_selection {
            Some(cli_currency_selection) => CurrencySelection::from(cli_currency_selection)?,
            None => CurrencySelection::choose_currency()?,
        };
        Ok(Self { currency_selection })
    }
}

impl Currency {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        self.currency_selection
            .process(prepopulated_unsigned_transaction)
            .await
    }
}

#[derive(Debug, Clone, clap::Clap)]
enum CliCurrencySelection {
    /// отправка трансфера в NEAR tokens
    NEAR(self::operation_mode::CliOperationMode),
}

#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
enum CurrencySelection {
    #[strum_discriminants(strum(message = "NEAR tokens"))]
    NEAR(self::operation_mode::OperationMode),
}

impl CliCurrencySelection {
    pub fn to_cli_args(&self) -> std::collections::VecDeque<String> {
        match self {
            Self::NEAR(operation_mode) => {
                let mut args = operation_mode.to_cli_args();
                args.push_front("near".to_owned());
                args
            }
        }
    }
}

impl From<CurrencySelection> for CliCurrencySelection {
    fn from(currency_selection: CurrencySelection) -> Self {
        match currency_selection {
            CurrencySelection::NEAR(operation_mode) => {
                Self::NEAR(self::operation_mode::CliOperationMode::from(operation_mode))
            }
        }
    }
}

impl CurrencySelection {
    fn from(item: CliCurrencySelection) -> color_eyre::eyre::Result<Self> {
        match item {
            CliCurrencySelection::NEAR(cli_operation_mode) => Ok(Self::NEAR(
                self::operation_mode::OperationMode::from(cli_operation_mode)?,
            )),
        }
    }
}

impl CurrencySelection {
    fn choose_currency() -> color_eyre::eyre::Result<Self> {
        println!();
        let variants = CurrencySelectionDiscriminants::iter().collect::<Vec<_>>();
        let currencies = variants
            .iter()
            .map(|p| p.get_message().unwrap().to_owned())
            .collect::<Vec<_>>();
        let selected_currency = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What do you want to transfer?")
            .items(&currencies)
            .default(0)
            .interact()
            .unwrap();
        let cli_currency = match variants[selected_currency] {
            CurrencySelectionDiscriminants::NEAR => CliCurrencySelection::NEAR(Default::default()),
        };
        Ok(Self::from(cli_currency)?)
    }

    async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            Self::NEAR(operation_mode) => {
                operation_mode
                    .process(prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}
