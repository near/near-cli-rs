use dialoguer::Input;

#[derive(Debug, Clone, interactive_clap_derive::InteractiveClap)]
#[interactive_clap(context = crate::common::SignerContext)]
pub struct StakeNEARTokensAction {
    pub stake_amount: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    ///Enter an public key
    pub transactions_signing_public_key: super::transactions_signing::TransactionsSigningAction,
}

impl interactive_clap::ToCli for crate::common::NearBalance {
    type CliVariant = crate::common::NearBalance;
}

impl StakeNEARTokensAction {
    fn input_stake_amount(
        _context: &crate::common::SignerContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        Ok(Input::new()
            .with_prompt("How many NEAR Tokens do you want to stake? (example: 10NEAR or 0.5near or 10000yoctonear)")
            .interact_text()
            ?)
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
    ) -> crate::CliResult {
        self.transactions_signing_public_key
            .process(
                prepopulated_unsigned_transaction,
                network_connection_config,
                self.stake_amount.to_yoctonear(),
            )
            .await
    }
}
