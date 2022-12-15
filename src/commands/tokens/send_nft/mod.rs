use inquire::Text;
use serde_json::json;
use std::str::FromStr;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct SendNftCommand {
    ///What is the nft-contract account ID?
    nft_contract_account_id: crate::types::account_id::AccountId,
    ///What is the receiver account ID?
    receiver_account_id: crate::types::account_id::AccountId,
    ///Enter an token_id for NFT
    token_id: String,
    #[interactive_clap(long = "prepaid-gas")]
    #[interactive_clap(skip_default_input_arg)]
    ///Enter gas for function call
    gas: crate::common::NearGas,
    #[interactive_clap(long = "attached-deposit")]
    #[interactive_clap(skip_default_input_arg)]
    ///Enter deposit for a function call
    deposit: crate::common::NearBalance,
    #[interactive_clap(named_arg)]
    ///Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SendNftCommand {
    fn input_gas(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearGas> {
        println!();
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
                        println!("You need to enter a value of no more than 300 TERAGAS")
                    }
                }
                Err(err) => return Err(color_eyre::Report::msg(err)),
            }
        };
        Ok(gas.into())
    }

    fn input_deposit(
        _context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<crate::common::NearBalance> {
        println!();
        match crate::common::NearBalance::from_str(
            &Text::new(
                "Enter deposit for a function call (example: 10NEAR or 0.5near or 10000yoctonear).",
            )
            .with_initial_value("1 yoctoNEAR")
            .prompt()?,
        ) {
            Ok(deposit) => Ok(deposit),
            Err(err) => Err(color_eyre::Report::msg(err)),
        }
    }

    pub async fn process(
        &self,
        config: crate::config::Config,
        owner_account_id: near_primitives::types::AccountId,
    ) -> crate::CliResult {
        let method_name = "nft_transfer".to_string();
        let args = json!({
            "receiver_id": self.receiver_account_id.to_string(),
            "token_id": self.token_id
        })
        .to_string()
        .into_bytes();
        let prepopulated_unsigned_transaction = near_primitives::transaction::Transaction {
            signer_id: owner_account_id.clone(),
            public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
            nonce: 0,
            receiver_id: self.nft_contract_account_id.clone().into(),
            block_hash: Default::default(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                near_primitives::transaction::FunctionCallAction {
                    method_name,
                    args,
                    gas: self.gas.clone().inner,
                    deposit: self.deposit.clone().to_yoctonear(),
                },
            )],
        };
        match crate::transaction_signature_options::sign_with(
            self.network_config.clone(),
            prepopulated_unsigned_transaction,
            config.clone(),
        )
        .await?
        {
            Some(transaction_info) => {
                if !matches!(
                    transaction_info.status,
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_)
                ) {
                    return crate::common::print_transaction_status(
                        transaction_info,
                        self.network_config.get_network_config(config),
                    );
                }
                println!(
                    "<{sender}> has successfully transferred NFT token_id=\"{id}\" to <{receiver}> on contract <{contract}>.",
                    sender=owner_account_id,
                    id=self.token_id,
                    receiver=self.receiver_account_id,
                    contract=self.nft_contract_account_id
                );
                println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
                    id=transaction_info.transaction_outcome.id,
                    path=self.network_config.get_network_config(config).explorer_transaction_url
                );
                Ok(())
            }
            None => Ok(()),
        }
    }
}
