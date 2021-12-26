mod public_key_mode;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct SubAccountFullAccess {
    #[interactive_clap(subcommand)]
    pub public_key_mode: self::public_key_mode::PublicKeyMode,
}

impl SubAccountFullAccess {
    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.public_key_mode
            .process(prepopulated_unsigned_transaction, network_connection_config)
            .await
    }
}
