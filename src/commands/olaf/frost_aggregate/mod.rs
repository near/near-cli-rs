use near_primitives::transaction::Transaction;
use olaf::frost::{aggregate, SigningPackage};
use serde_json::from_str;
use std::{
    fs::{self, File},
    io::Write,
    str::FromStr,
};

use crate::{
    common::JsonRpcClientExt,
    config::NetworkConfig,
    types::{path_buf::PathBuf, transaction::TransactionAsBase64},
};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = FrostAggregateContext)]
pub struct FrostAggregate {
    #[interactive_clap(long)]
    /// The folder that contains the files for the aggregation round of the FROST protocol
    files: PathBuf,
}

#[derive(Debug, Clone)]
pub struct FrostAggregateContext;

impl FrostAggregateContext {
    pub fn from_previous_context(
        _previous_context: crate::GlobalContext,
        scope: &<FrostAggregate as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> crate::CliResult {
        let file_path: std::path::PathBuf = scope.files.clone().into();

        let signing_packages_string =
            fs::read_to_string(file_path.join("signing_packages.json")).unwrap();
        let signing_packages_bytes: Vec<Vec<u8>> = from_str(&signing_packages_string).unwrap();

        let signing_packages: Vec<SigningPackage> = signing_packages_bytes
            .iter()
            .map(|signing_commitments| SigningPackage::from_bytes(signing_commitments).unwrap())
            .collect();

        let signature = aggregate(&signing_packages).unwrap();

        let signature_json = serde_json::to_string_pretty(&signature.to_bytes().to_vec()).unwrap();

        let mut signature_file = File::create(file_path.join("signature.json")).unwrap();

        signature_file.write_all(signature_json.as_bytes()).unwrap();

        let unsigned_tx_string = fs::read_to_string(file_path.join("unsigned_tx.json")).unwrap();

        let unsigned_tx_str: String = from_str(&unsigned_tx_string).unwrap();

        let unsigned_transaction: Transaction = TransactionAsBase64::from_str(&unsigned_tx_str)
            .unwrap()
            .into();

        let signed_transaction = near_primitives::transaction::SignedTransaction::new(
            near_crypto::Signature::ED25519(signature),
            unsigned_transaction,
        );

        let network_config = NetworkConfig {
            network_name: "testnet".to_string(),
            rpc_url: "https://archival-rpc.testnet.near.org".parse().unwrap(),
            wallet_url: "https://testnet.mynearwallet.com/".parse().unwrap(),
            explorer_transaction_url: "https://explorer.testnet.near.org/transactions/"
                .parse()
                .unwrap(),
            rpc_api_key: None,
            linkdrop_account_id: Some("testnet".parse().unwrap()),
            near_social_db_contract_account_id: Some("v1.social08.testnet".parse().unwrap()),
            faucet_url: Some("https://helper.nearprotocol.com/account".parse().unwrap()),
            meta_transaction_relayer_url: None,
            fastnear_url: None,
            staking_pools_factory_account_id: Some("pool.f863973.m0".parse().unwrap()),
            coingecko_url: None,
        };

        eprintln!("Transaction sent ...");
        let transaction_info = loop {
            let transaction_info_result = network_config.json_rpc_client().blocking_call(
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcBroadcastTxCommitRequest {
                    signed_transaction: signed_transaction.clone(),
                },
            );
            match transaction_info_result {
                Ok(response) => {
                    break response;
                }
                Err(err) => match crate::common::rpc_transaction_error(&err) {
                    Ok(_) => std::thread::sleep(std::time::Duration::from_millis(100)),
                    Err(report) => return Err(color_eyre::Report::msg(report)),
                },
            };
        };
        crate::common::print_transaction_status(&transaction_info, &network_config)
    }
}
