use std::str::FromStr;

use inquire::Text;
use serde_json::json;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::TokensCommandsContext)]
#[interactive_clap(output_context = SendNftCommandContext)]
pub struct SendNftCommand {
    /// What is the nft-contract account ID?
    nft_contract_account_id: crate::types::account_id::AccountId,
    /// What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    /// Enter an token_id for NFT
    token_id: String,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

#[derive(Debug, Clone)]
pub struct SendNftCommandContext {
    config: crate::config::Config,
    signer_account_id: near_primitives::types::AccountId,
    nft_contract_account_id: near_primitives::types::AccountId,
    receiver_account_id: near_primitives::types::AccountId,
    token_id: String,
    gas: crate::common::NearGas,
    deposit: crate::common::NearBalance,
}

impl SendNftCommandContext {
    pub fn from_previous_context(
        previous_context: super::TokensCommandsContext,
        scope: &<SendNftCommand as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            config: previous_context.config,
            signer_account_id: previous_context.owner_account_id,
            nft_contract_account_id: scope.nft_contract_account_id.clone().into(),
            receiver_account_id: scope.receiver_account_id.clone().into(),
            token_id: scope.token_id.clone(),
            gas: scope.gas.clone(),
            deposit: scope.deposit.clone(),
        })
    }
}

impl From<SendNftCommandContext> for crate::commands::ActionContext {
    fn from(item: SendNftCommandContext) -> Self {
        let signer_account_id = item.signer_account_id.clone();
        let nft_contract_account_id = item.nft_contract_account_id.clone();
        let receiver_account_id = item.receiver_account_id.clone();
        let token_id = item.token_id.clone();

        let on_after_getting_network_callback: crate::commands::OnAfterGettingNetworkCallback =
            std::sync::Arc::new(move |prepopulated_unsigned_transaction, _network_config| {
                prepopulated_unsigned_transaction.signer_id = signer_account_id.clone();
                prepopulated_unsigned_transaction.receiver_id = nft_contract_account_id.clone();
                prepopulated_unsigned_transaction.actions =
                    vec![near_primitives::transaction::Action::FunctionCall(
                        near_primitives::transaction::FunctionCallAction {
                            method_name: "nft_transfer".to_string(),
                            args: json!({
                                "receiver_id": receiver_account_id.to_string(),
                                "token_id": token_id
                            })
                            .to_string()
                            .into_bytes(),
                            gas: item.gas.inner,
                            deposit: item.deposit.to_yoctonear(),
                        },
                    )];
                Ok(())
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new(
            move |outcome_view, _network_config| {
                if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
                    eprintln!(
                        "<{}> has successfully transferred NFT token_id=\"{}\" to <{}> on contract <{}>.",
                        item.signer_account_id,
                        item.token_id,
                        item.receiver_account_id,
                        item.nft_contract_account_id,
                    );
                }
                Ok(())
            },
        );

        Self {
            config: item.config,
            on_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config, _message| Ok(()),
            ),
            on_after_sending_transaction_callback,
        }
    }
}

impl SendNftCommand {
    fn input_gas(
        _context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        eprintln!();
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
                        eprintln!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(Some(gas.into()))
    }

    fn input_deposit(
        _context: &super::TokensCommandsContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearBalance>> {
        eprintln!();
        match crate::common::NearBalance::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_value("1 yoctoNEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(Some(deposit)),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }
}
