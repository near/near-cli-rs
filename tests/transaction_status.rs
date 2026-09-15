mod common;
use std::process::Command;

#[tokio::test]
async fn test_failed_transaction_reports_receipt_totals_and_logs_once()
-> Result<(), Box<dyn std::error::Error>> {
    let caller = near_sandbox::GenesisAccount::default_with_name("caller.near".parse()?);
    let ctx = common::prepare_tests_with_accounts(vec![caller.clone()]).await?;

    // Two function calls to an account without a contract: the transaction fails on-chain
    // and produces more than one receipt outcome.
    let output = Command::new("target/debug/near")
        .env("XDG_CONFIG_HOME", &ctx.config_home)
        .env("HOME", &ctx.config_home)
        .env("APPDATA", &ctx.config_home)
        .args([
            "transaction",
            "construct-transaction",
            caller.account_id.as_str(),
            caller.account_id.as_str(),
            "add-action",
            "function-call",
            "first",
            "json-args",
            "{}",
            "prepaid-gas",
            "10 Tgas",
            "attached-deposit",
            "0 NEAR",
            "add-action",
            "function-call",
            "second",
            "json-args",
            "{}",
            "prepaid-gas",
            "10 Tgas",
            "attached-deposit",
            "0 NEAR",
            "skip",
            "network-config",
            "sandbox",
            "sign-with-plaintext-private-key",
            &caller.private_key,
            "send",
        ])
        .output()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "expected failure: {stderr}");

    let tx_hash = regex::Regex::new(r"Transaction ID: (\w+)")?
        .captures(&stderr)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| panic!("no transaction ID in output: {stderr}"));

    let outcome = near_jsonrpc_client::JsonRpcClient::connect(&ctx.sandbox.rpc_addr)
        .call(
            near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
                transaction_info:
                    near_jsonrpc_client::methods::tx::TransactionInfo::TransactionId {
                        tx_hash: tx_hash
                            .parse()
                            .map_err(|e: Box<dyn std::error::Error + Send + Sync>| e.to_string())?,
                        sender_account_id: caller.account_id.as_str().parse()?,
                    },
                wait_until: near_primitives::views::TxExecutionStatus::Final,
            },
        )
        .await?
        .final_execution_outcome
        .expect("final execution outcome")
        .into_outcome();
    let mut gas = outcome.transaction_outcome.outcome.gas_burnt;
    let mut fee = outcome.transaction_outcome.outcome.tokens_burnt;
    for receipt in &outcome.receipts_outcome {
        gas = gas.checked_add(receipt.outcome.gas_burnt).unwrap();
        fee = fee.checked_add(receipt.outcome.tokens_burnt).unwrap();
    }
    assert!(outcome.receipts_outcome.len() > 1, "{stderr}");

    assert!(stderr.contains(&format!("Gas burned: {gas}")), "{stderr}");
    assert!(
        stderr.contains(&format!("Transaction fee: {}", fee.exact_amount_display())),
        "{stderr}"
    );
    assert_eq!(
        stderr.matches("Function execution logs:").count(),
        1,
        "{stderr}"
    );
    assert_eq!(
        stderr.matches("Logs [").count(),
        outcome.receipts_outcome.len(),
        "{stderr}"
    );

    Ok(())
}
