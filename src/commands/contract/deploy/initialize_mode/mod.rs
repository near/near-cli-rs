use strum::{EnumDiscriminants, EnumIter, EnumMessage};

mod call_function_type;

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
///Select the need for initialization
pub enum InitializeMode {
    /// Add an initialize
    #[strum_discriminants(strum(message = "with-init-call     - Add an initialize"))]
    WithInitCall(self::call_function_type::CallFunctionAction),
    /// Don't add an initialize
    #[strum_discriminants(strum(message = "without-init-call  - Don't add an initialize"))]
    WithoutInitCall(NoInitialize),
}

impl InitializeMode {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self {
            InitializeMode::WithInitCall(call_function_action) => {
                call_function_action
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
            InitializeMode::WithoutInitCall(no_initialize) => {
                no_initialize
                    .process(config, prepopulated_unsigned_transaction)
                    .await
            }
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct NoInitialize {
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl NoInitialize {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => crate::common::print_transaction_status(
                transaction_info,
                self.network_config.get_network_config(config),
            ),
            None => Ok(()),
        }
    }
}
