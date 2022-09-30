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
    network: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl NoInitialize {
    pub async fn process(
        &self,
        config: crate::config::Config,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
    ) -> crate::CliResult {
        match self.network.get_sign_option() {
            crate::transaction_signature_options::SignWith::SignWithPlaintextPrivateKey(
                sign_private_key,
            ) => {
                sign_private_key
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network.get_network_config(config),
                    )
                    .await
            }
            crate::transaction_signature_options::SignWith::SignWithKeychain(sign_keychain) => {
                sign_keychain
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network.get_network_config(config.clone()),
                        config.credentials_home_dir,
                    )
                    .await
            }
            #[cfg(feature = "ledger")]
            crate::transaction_signature_options::SignWith::SignWithLedger(sign_ledger) => {
                sign_ledger
                    .process(
                        prepopulated_unsigned_transaction,
                        self.network.get_network_config(config),
                    )
                    .await
            }
        }
    }
}
