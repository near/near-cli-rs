use std::collections::{HashSet, VecDeque};
use std::fs::OpenOptions;
use std::io::Write;
use std::str::FromStr;

use base64::Engine as _;

use color_eyre::eyre::{ContextCompat, WrapErr};
use color_eyre::owo_colors::OwoColorize;
use futures::{StreamExt, TryStreamExt};
use prettytable::Table;
use rust_decimal::prelude::FromPrimitive;
use tracing_indicatif::span_ext::IndicatifSpanExt;
use tracing_indicatif::suspend_tracing_indicatif;

use near_kit::BlockReference;

pub type CliResult = color_eyre::eyre::Result<()>;

use inquire::{Select, Text};
use strum::IntoEnumIterator;

use crate::types::partial_protocol_config::get_partial_protocol_config;

const FINAL_COMMAND_FILE_NAME: &str = "near-cli-rs-final-command.log";

pub fn get_near_exec_path() -> String {
    std::env::args()
        .next()
        .unwrap_or_else(|| "./near".to_owned())
}

#[derive(
    Debug,
    Clone,
    strum_macros::IntoStaticStr,
    strum_macros::EnumString,
    strum_macros::EnumVariantNames,
    Default,
)]
#[strum(serialize_all = "snake_case")]
pub enum OutputFormat {
    #[default]
    Plaintext,
    Json,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Plaintext => write!(f, "plaintext"),
            OutputFormat::Json => write!(f, "json"),
        }
    }
}

pub use near_gas::NearGas;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd)]
pub struct TransferAmount {
    amount: near_token::NearToken,
}

impl interactive_clap::ToCli for TransferAmount {
    type CliVariant = near_token::NearToken;
}

impl std::fmt::Display for TransferAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

impl TransferAmount {
    pub fn from(
        amount: near_token::NearToken,
        account_transfer_allowance: &AccountTransferAllowance,
    ) -> color_eyre::eyre::Result<Self> {
        if amount <= account_transfer_allowance.transfer_allowance() {
            Ok(Self { amount })
        } else {
            Err(color_eyre::Report::msg(
                "the amount exceeds the transfer allowance",
            ))
        }
    }

    pub fn from_unchecked(amount: near_token::NearToken) -> Self {
        Self { amount }
    }

    pub fn as_yoctonear(&self) -> u128 {
        self.amount.as_yoctonear()
    }
}

impl From<TransferAmount> for near_token::NearToken {
    fn from(item: TransferAmount) -> Self {
        item.amount
    }
}

#[derive(Debug)]
pub struct AccountTransferAllowance {
    account_id: near_kit::AccountId,
    account_liquid_balance: near_token::NearToken,
    account_locked_balance: near_token::NearToken,
    storage_stake: near_token::NearToken,
    pessimistic_transaction_fee: near_token::NearToken,
}

impl std::fmt::Display for AccountTransferAllowance {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            fmt,
            "{} account has {} available for transfer (the total balance is {}, but {} is locked for storage)",
            self.account_id,
            self.transfer_allowance(),
            self.account_liquid_balance,
            self.liquid_storage_stake(),
        )
    }
}

impl AccountTransferAllowance {
    pub fn liquid_storage_stake(&self) -> near_token::NearToken {
        self.storage_stake
            .saturating_sub(self.account_locked_balance)
    }

    pub fn transfer_allowance(&self) -> near_token::NearToken {
        self.account_liquid_balance
            .saturating_sub(self.liquid_storage_stake())
            .saturating_sub(self.pessimistic_transaction_fee)
    }
}

#[derive(Debug)]
pub enum AccountStateError {
    RpcError(String),
    Cancel,
    Skip,
}

impl std::fmt::Display for AccountStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::RpcError(err) => write!(f, "{err}"),
            Self::Cancel => write!(f, "Operation was canceled by the user"),
            Self::Skip => write!(f, "Operation was skipped by the user"),
        }
    }
}

impl std::error::Error for AccountStateError {}

#[tracing::instrument(name = "Waiting 3 seconds before sending a request via RPC", skip_all)]
pub async fn sleep_after_error(additional_message_for_name: String) {
    tracing::Span::current().pb_set_message(&additional_message_for_name);
    tracing::info!(target: "near_teach_me", "Waiting 3 seconds before sending a request via RPC {additional_message_for_name}");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
}

#[tracing::instrument(name = "Getting the transfer allowance for the account ...", skip_all)]
pub async fn get_account_transfer_allowance(
    network_config: &crate::config::NetworkConfig,
    account_id: near_kit::AccountId,
    block_reference: BlockReference,
) -> color_eyre::eyre::Result<AccountTransferAllowance> {
    tracing::info!(target: "near_teach_me", "Getting the transfer allowance for the account ...");
    let account_state =
        get_account_state(network_config, &account_id, block_reference.clone()).await;
    let account_view = match account_state {
        Ok(account_view) => account_view,
        Err(ViewAccountError::UnknownAccount { .. })
            if account_id.get_account_type().is_implicit() =>
        {
            return Ok(AccountTransferAllowance {
                account_id,
                account_liquid_balance: near_token::NearToken::ZERO,
                account_locked_balance: near_token::NearToken::ZERO,
                storage_stake: near_token::NearToken::ZERO,
                pessimistic_transaction_fee: near_token::NearToken::ZERO,
            });
        }
        Err(ViewAccountError::TransportError(err)) => {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount information ({account_id}) cannot be fetched on <{}> network due to connectivity issue.\n{err}",
                network_config.network_name
            ));
        }
        Err(err @ (ViewAccountError::UnknownAccount { .. } | ViewAccountError::ServerError(_))) => {
            return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!(
                "\nAccount information ({account_id}) cannot be fetched on <{}> network due to server error.\n{err}",
                network_config.network_name
            ));
        }
    };
    let storage_amount_per_byte =
        get_partial_protocol_config(network_config, &block_reference)
            .await?
            .runtime_config
            .storage_amount_per_byte;

    Ok(AccountTransferAllowance {
        account_id,
        account_liquid_balance: account_view.amount,
        account_locked_balance: account_view.locked,
        storage_stake: storage_amount_per_byte.saturating_mul(account_view.storage_usage.into()),
        // pessimistic_transaction_fee = 10^21 - this value is set temporarily
        // In the future, its value will be calculated by the function: fn tx_cost(...)
        // https://github.com/near/nearcore/blob/8a377fda0b4ce319385c463f1ae46e4b0b29dcd9/runtime/runtime/src/config.rs#L178-L232
        pessimistic_transaction_fee: near_token::NearToken::from_millinear(1),
    })
}

#[allow(clippy::result_large_err)]
#[tracing::instrument(name = "Account access key verification ...", skip_all)]
pub fn verify_account_access_key(
    account_id: near_kit::AccountId,
    public_key: near_kit::PublicKey,
    network_config: crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<
    near_kit::AccessKeyView,
    AccountStateError,
> {
    tracing::info!(target: "near_teach_me", "Account access key verification ...");
    loop {
        match blocking_view_access_key(
            &network_config,
            &account_id,
            &public_key,
            near_kit::BlockReference::optimistic(),
        ) {
            Ok(access_key_view) => {
                return Ok(access_key_view);
            }
            Err(err) => {
                let err_str = format!("{err}");
                // Check for "access key not found" / "unknown access key" style errors
                if err_str.contains("UnknownAccessKey")
                    || err_str.contains("access key")
                        && err_str.contains("does not exist")
                {
                    return Err(AccountStateError::RpcError(err_str));
                }
                // Distinguish transport vs server errors by checking for common transport error patterns
                let is_transport = err_str.contains("transport")
                    || err_str.contains("connection")
                    || err_str.contains("timeout")
                    || err_str.contains("DNS");
                let category = if is_transport {
                    "connectivity issue"
                } else {
                    "server error"
                };
                let need_check_account = suspend_tracing_indicatif::<
                    _,
                    color_eyre::eyre::Result<bool>,
                >(|| {
                    need_check_account(format!(
                        "Account information ({account_id}) cannot be fetched on <{}> network due to {category}.",
                        network_config.network_name
                    ))
                });
                if need_check_account.is_err() {
                    return Err(AccountStateError::Cancel);
                }
                if let Ok(false) = need_check_account {
                    return Err(AccountStateError::RpcError(err_str));
                }
            }
        }
    }
}

#[tracing::instrument(name = "Checking the existence of the account ...", skip_all)]
pub fn is_account_exist(
    context: &crate::GlobalContext,
    account_id: near_kit::AccountId,
) -> color_eyre::eyre::Result<bool> {
    tracing::info!(target: "near_teach_me", "Checking the existence of the account ...");
    loop {
        match find_network_where_account_exist(context, account_id.clone()) {
            Ok(network) => {
                if network.is_some() {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
            Err(err) => {
                tracing::warn!("{}", format!("{err}").red());
                let need_check_account =
                    suspend_tracing_indicatif::<_, color_eyre::eyre::Result<bool>>(|| {
                        need_check_account(format!(
                            "Failed to check account existence for {account_id}."
                        ))
                    });
                if !need_check_account.wrap_err_with(|| format!("{err}"))? {
                    return Ok(true);
                }
            }
        }
    }
}

#[tracing::instrument(name = "Searching for a network where an account exists for", skip_all)]
pub fn find_network_where_account_exist(
    context: &crate::GlobalContext,
    new_account_id: near_kit::AccountId,
) -> color_eyre::eyre::Result<Option<crate::config::NetworkConfig>> {
    tracing::Span::current().pb_set_message(new_account_id.as_str());
    tracing::info!(target: "near_teach_me", "Searching for a network where an account exists for {new_account_id} ...");
    let networks: HashSet<String> = context
        .config
        .network_connection
        .iter()
        .map(|(_, network_config)| network_config.network_name.clone())
        .collect();
    if networks.is_empty() {
        return Err(color_eyre::eyre::eyre!(
            "No network connections are configured, so it's impossible to check the existence of the account."
        ));
    }
    let mut checked_networks: HashSet<String> = HashSet::new();
    let mut unknown_account_result: HashSet<String> = HashSet::new();
    for (_, network_config) in context.config.network_connection.iter() {
        if checked_networks.contains(&network_config.network_name) {
            continue;
        } else {
            checked_networks.insert(network_config.network_name.clone());
        }

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(get_account_state(
                network_config,
                &new_account_id,
                near_kit::BlockReference::optimistic(),
            ));

        match result {
            Ok(_) => return Ok(Some(network_config.clone())),
            Err(ViewAccountError::UnknownAccount { .. }) => {
                unknown_account_result.insert(network_config.network_name.clone());
            }
            Err(_err) => {
                checked_networks.remove(&network_config.network_name);
            }
        }
    }
    if networks == unknown_account_result {
        Ok(None)
    } else if unknown_account_result.is_empty() {
        let mut error_networks: Vec<String> = networks
            .difference(&unknown_account_result)
            .cloned()
            .collect();
        error_networks.sort();

        Err(color_eyre::eyre::eyre!(
            "Account information ({new_account_id}) cannot be fetched on the following networks due to errors: {}.",
            error_networks.join(", "),
        ))
    } else {
        let mut error_networks: Vec<String> = networks
            .difference(&unknown_account_result)
            .cloned()
            .collect();
        error_networks.sort();
        let mut unknown_vec: Vec<String> = unknown_account_result.iter().cloned().collect();
        unknown_vec.sort();

        Err(color_eyre::eyre::eyre!(
            "Account information ({new_account_id}) cannot be fetched on the following networks due to errors: {}.\nIt was checked that the account does not exist on the following networks: {}.",
            error_networks.join(", "),
            unknown_vec.join(", ")
        ))
    }
}

pub fn ask_if_different_account_id_wanted() -> color_eyre::eyre::Result<bool> {
    #[derive(strum_macros::Display, PartialEq)]
    enum ConfirmOptions {
        #[strum(to_string = "Yes, I want to enter a new name for account ID.")]
        Yes,
        #[strum(to_string = "No, I want to keep using this name for account ID.")]
        No,
    }
    let select_choose_input = Select::new(
        "Do you want to enter a different name for the new account ID?",
        vec![ConfirmOptions::Yes, ConfirmOptions::No],
    )
    .prompt()?;
    Ok(select_choose_input == ConfirmOptions::Yes)
}

/// Error type for `get_account_state` that categorizes errors from the RPC call.
///
/// Callers can match on variants to distinguish between "account not found",
/// "transport/connectivity" issues, and other server errors.
#[derive(Debug)]
pub enum ViewAccountError {
    /// The account does not exist on-chain.
    UnknownAccount {
        account_id: near_kit::AccountId,
    },
    /// A transport / connectivity error occurred.
    TransportError(String),
    /// Any other server-side error.
    ServerError(String),
}

impl std::fmt::Display for ViewAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAccount { account_id } => {
                write!(f, "Account not found: {account_id}")
            }
            Self::TransportError(msg) => write!(f, "Transport error: {msg}"),
            Self::ServerError(msg) => write!(f, "Server error: {msg}"),
        }
    }
}

impl std::error::Error for ViewAccountError {}

/// Classify a near-kit `RpcError` into a `ViewAccountError`.
fn classify_view_account_error(
    err: near_kit::RpcError,
    account_id: &near_kit::AccountId,
) -> ViewAccountError {
    match &err {
        near_kit::RpcError::AccountNotFound(_) => ViewAccountError::UnknownAccount {
            account_id: account_id.clone(),
        },
        near_kit::RpcError::Http(_)
        | near_kit::RpcError::Network { .. }
        | near_kit::RpcError::Timeout(_) => ViewAccountError::TransportError(err.to_string()),
        _ => ViewAccountError::ServerError(err.to_string()),
    }
}

#[tracing::instrument(name = "Getting account status information for", skip_all)]
pub async fn get_account_state(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    block_reference: BlockReference,
) -> Result<near_kit::AccountView, ViewAccountError> {
    tracing::Span::current().pb_set_message(&format!(
        "<{account_id}> on network <{}> ...",
        network_config.network_name
    ));
    tracing::info!(target: "near_teach_me", "Getting account status information for <{account_id}> on network <{}> ...", network_config.network_name);
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to query information about `{}` account, learn more https://docs.near.org/api/rpc/contracts#view-account",
        account_id
    );

    let nk_block_ref = block_reference.clone();

    let mut retries_left = (0..5).rev();
    loop {
        let result = network_config
            .client()
            .rpc()
            .view_account(account_id, nk_block_ref.clone())
            .await;

        match result {
            Ok(account_view) => {
                return Ok(account_view);
            }
            Err(err) => {
                let classified = classify_view_account_error(err, account_id);
                match &classified {
                    ViewAccountError::UnknownAccount { .. } => {
                        return Err(classified);
                    }
                    ViewAccountError::TransportError(_) | ViewAccountError::ServerError(_) => {
                        if let Some(retries_left) = retries_left.next() {
                            sleep_after_error(format!(
                                "(Previous attempt failed with error: `{}`. Will retry {} more times)",
                                classified.to_string().red(),
                                retries_left
                            ))
                            .await;
                        } else {
                            return Err(classified);
                        }
                    }
                }
            }
        }
    }
}

fn need_check_account(message: String) -> color_eyre::eyre::Result<bool> {
    #[derive(strum_macros::Display, PartialEq)]
    enum ConfirmOptions {
        #[strum(to_string = "Yes, I want to check the account again.")]
        Yes,
        #[strum(to_string = "No, I want to skip the check and use the specified account ID.")]
        No,
    }
    let select_choose_input = Select::new(
        &format!("{message} Do you want to try again?"),
        vec![ConfirmOptions::Yes, ConfirmOptions::No],
    )
    .prompt()?;

    Ok(select_choose_input == ConfirmOptions::Yes)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyPairProperties {
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    pub master_seed_phrase: String,
    pub implicit_account_id: near_kit::AccountId,
    #[serde(rename = "public_key")]
    pub public_key_str: String,
    #[serde(rename = "private_key")]
    pub secret_keypair_str: String,
}

pub fn get_key_pair_properties_from_seed_phrase(
    seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    master_seed_phrase: String,
) -> color_eyre::eyre::Result<KeyPairProperties> {
    let master_seed = bip39::Mnemonic::parse(&master_seed_phrase)?.to_seed("");
    let derived_private_key = slipped10::derive_key_from_path(
        &master_seed,
        slipped10::Curve::Ed25519,
        &seed_phrase_hd_path.clone().into(),
    )
    .map_err(|err| {
        color_eyre::Report::msg(format!("Failed to derive a key from the master key: {err}"))
    })?;

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&derived_private_key.key);

    let public_key = signing_key.verifying_key();
    let implicit_account_id = near_kit::AccountId::try_from(hex::encode(public_key))?;
    let public_key_str = format!("ed25519:{}", bs58::encode(&public_key).into_string());
    let secret_keypair_str = format!(
        "ed25519:{}",
        bs58::encode(signing_key.to_keypair_bytes()).into_string()
    );
    let key_pair_properties: KeyPairProperties = KeyPairProperties {
        seed_phrase_hd_path,
        master_seed_phrase,
        implicit_account_id,
        public_key_str,
        secret_keypair_str,
    };
    Ok(key_pair_properties)
}

pub fn get_public_key_from_seed_phrase(
    seed_phrase_hd_path: slipped10::BIP32Path,
    master_seed_phrase: &str,
) -> color_eyre::eyre::Result<near_kit::PublicKey> {
    let master_seed = bip39::Mnemonic::parse(master_seed_phrase)?.to_seed("");
    let derived_private_key = slipped10::derive_key_from_path(
        &master_seed,
        slipped10::Curve::Ed25519,
        &seed_phrase_hd_path,
    )
    .map_err(|err| {
        color_eyre::Report::msg(format!("Failed to derive a key from the master key: {err}"))
    })?;
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&derived_private_key.key);
    let public_key_str = format!(
        "ed25519:{}",
        bs58::encode(&signing_key.verifying_key()).into_string()
    );
    Ok(near_kit::PublicKey::from_str(&public_key_str)?)
}

pub fn generate_keypair() -> color_eyre::eyre::Result<KeyPairProperties> {
    let generate_keypair: crate::utils_command::generate_keypair_subcommand::CliGenerateKeypair =
        crate::utils_command::generate_keypair_subcommand::CliGenerateKeypair::default();
    let (master_seed_phrase, master_seed) =
        if let Some(master_seed_phrase) = generate_keypair.master_seed_phrase.as_deref() {
            (
                master_seed_phrase.to_owned(),
                bip39::Mnemonic::parse(master_seed_phrase)?.to_seed(""),
            )
        } else {
            let mnemonic =
                bip39::Mnemonic::generate(generate_keypair.new_master_seed_phrase_words_count)?;
            let master_seed_phrase = mnemonic.words().collect::<Vec<&str>>().join(" ");
            (master_seed_phrase, mnemonic.to_seed(""))
        };

    let derived_private_key = slipped10::derive_key_from_path(
        &master_seed,
        slipped10::Curve::Ed25519,
        &generate_keypair.seed_phrase_hd_path.clone().into(),
    )
    .map_err(|err| {
        color_eyre::Report::msg(format!("Failed to derive a key from the master key: {err}"))
    })?;

    let signing_key = ed25519_dalek::SigningKey::from_bytes(&derived_private_key.key);

    let public = signing_key.verifying_key();
    let implicit_account_id = near_kit::AccountId::try_from(hex::encode(public))?;
    let public_key_str = format!("ed25519:{}", bs58::encode(&public).into_string());
    let secret_keypair_str = format!(
        "ed25519:{}",
        bs58::encode(signing_key.to_keypair_bytes()).into_string()
    );
    let key_pair_properties: KeyPairProperties = KeyPairProperties {
        seed_phrase_hd_path: generate_keypair.seed_phrase_hd_path,
        master_seed_phrase,
        implicit_account_id,
        public_key_str,
        secret_keypair_str,
    };
    Ok(key_pair_properties)
}

pub fn print_full_signed_transaction(
    transaction: &near_kit::SignedTransaction,
) -> String {
    let mut info_str = format!("\n{:<13} {}", "signature:", transaction.signature);
    info_str.push_str(&format!(
        "\nunsigned transaction hash (Base58-encoded SHA-256 hash): {}",
        transaction.transaction.get_hash()
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "public_key:",
        &transaction.transaction.public_key
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "nonce:",
        transaction.transaction.nonce
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "block_hash:",
        &transaction.transaction.block_hash
    ));
    let prepopulated = crate::commands::PrepopulatedTransaction {
        signer_id: transaction.transaction.signer_id.clone(),
        receiver_id: transaction.transaction.receiver_id.clone(),
        actions: transaction.transaction.actions.clone(),
    };
    info_str.push_str(&print_unsigned_transaction(&prepopulated));
    info_str
}

pub fn print_full_unsigned_transaction(
    transaction: &near_kit::Transaction,
) -> String {
    let mut info_str = format!(
        "\nunsigned transaction hash (Base58-encoded SHA-256 hash): {}",
        transaction.get_hash_and_size().0
    );

    info_str.push_str(&format!(
        "\n{:<13} {}",
        "public_key:",
        &transaction.public_key
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "nonce:",
        transaction.nonce
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "block_hash:",
        &transaction.block_hash
    ));

    let prepopulated = crate::commands::PrepopulatedTransaction {
        signer_id: transaction.signer_id.clone(),
        receiver_id: transaction.receiver_id.clone(),
        actions: transaction.actions.clone(),
    };

    info_str.push_str(&print_unsigned_transaction(&prepopulated));

    info_str
}

pub fn print_unsigned_transaction(
    transaction: &crate::commands::PrepopulatedTransaction,
) -> String {
    let mut info_str = String::new();
    info_str.push_str(&format!("\n{:<13} {}", "signer_id:", transaction.signer_id));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "receiver_id:", transaction.receiver_id
    ));

    if transaction
        .actions
        .iter()
        .any(|action| matches!(action, near_kit::Action::Delegate(_)))
    {
        info_str.push_str("\nsigned delegate action:");
    } else {
        info_str.push_str("\nactions:");
    };

    for action in &transaction.actions {
        match action {
            near_kit::Action::CreateAccount(_) => {
                info_str.push_str(&format!(
                    "\n{:>5} {:<20} {}",
                    "--", "create account:", &transaction.receiver_id
                ));
            }
            near_kit::Action::DeployContract(deploy) => {
                let code_hash = near_kit::CryptoHash::hash(&deploy.code);
                info_str.push_str(&format!(
                    "\n{:>5} {:<70}",
                    "--",
                    format!(
                        "deploy code <{}> to a account <{}>",
                        code_hash, transaction.receiver_id
                    )
                ))
            }
            near_kit::Action::FunctionCall(function_call_action) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "function call:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "method name:", &function_call_action.method_name
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "",
                    "args:",
                    match serde_json::from_slice::<serde_json::Value>(&function_call_action.args) {
                        Ok(parsed_args) => {
                            serde_json::to_string_pretty(&parsed_args)
                                .unwrap_or_else(|_| "".to_string())
                                .replace('\n', "\n                                 ")
                        }
                        Err(_) => {
                            if let Ok(args) = String::from_utf8(function_call_action.args.clone()) {
                                args
                            } else {
                                format!(
                                    "<non-printable data ({})>",
                                    bytesize::ByteSize(function_call_action.args.len() as u64)
                                )
                            }
                        }
                    }
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "gas:", function_call_action.gas
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "",
                    "deposit:",
                    function_call_action.deposit.exact_amount_display()
                ));
            }
            near_kit::Action::Transfer(transfer_action) => {
                info_str.push_str(&format!(
                    "\n{:>5} {:<20} {}",
                    "--",
                    "transfer deposit:",
                    transfer_action.deposit.exact_amount_display()
                ));
            }
            near_kit::Action::Stake(stake_action) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "stake:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", &stake_action.public_key
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "",
                    "stake:",
                    stake_action.stake.exact_amount_display()
                ));
            }
            near_kit::Action::AddKey(add_key_action) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "add access key:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", &add_key_action.public_key
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "nonce:", &add_key_action.access_key.nonce
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {:?}",
                    "", "permission:", &add_key_action.access_key.permission
                ));
            }
            near_kit::Action::DeleteKey(delete_key_action) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "delete access key:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", &delete_key_action.public_key
                ));
            }
            near_kit::Action::DeleteAccount(delete_account_action) => {
                info_str.push_str(&format!(
                    "\n{:>5} {:<20} {}",
                    "--", "delete account:", &transaction.receiver_id
                ));
                info_str.push_str(&format!(
                    "\n{:>8} {:<17} {}",
                    "", "beneficiary id:", &delete_account_action.beneficiary_id
                ));
            }
            near_kit::Action::Delegate(signed_delegate_action) => {
                let prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                    signer_id: signed_delegate_action.delegate_action.sender_id.clone(),
                    receiver_id: signed_delegate_action.delegate_action.receiver_id.clone(),
                    actions: signed_delegate_action
                        .delegate_action
                        .actions
                        .iter()
                        .map(|nda| {
                            // NonDelegateAction wraps Action with identical borsh encoding
                            let bytes = borsh::to_vec(nda).expect("NonDelegateAction borsh serialization should not fail");
                            borsh::from_slice::<near_kit::Action>(&bytes).expect("Action borsh deserialization should not fail")
                        })
                        .collect(),
                };
                info_str.push_str(&print_unsigned_transaction(&prepopulated_transaction));
            }
            near_kit::Action::DeployGlobalContract(deploy) => {
                let code_hash = near_kit::CryptoHash::hash(&deploy.code);
                let identifier = match deploy.deploy_mode {
                    near_kit::GlobalContractDeployMode::CodeHash => {
                        format!("deploy code <{code_hash}> as a global hash")
                    }
                    near_kit::GlobalContractDeployMode::AccountId => {
                        format!(
                            "deploy code <{}> to a global account <{}>",
                            code_hash, transaction.receiver_id
                        )
                    }
                };
                info_str.push_str(&format!("{:>5} {:<70}", "--", identifier));
            }
            near_kit::Action::UseGlobalContract(use_global) => {
                let identifier = match &use_global.contract_identifier {
                    near_kit::GlobalContractIdentifier::CodeHash(hash) => {
                        format!("use global <{hash}> code to deploy from")
                    }
                    near_kit::GlobalContractIdentifier::AccountId(account_id) => {
                        format!("use global <{account_id}> code to deploy from")
                    }
                };
                info_str.push_str(&format!("{:>5} {:<70}", "--", identifier));
            }
            near_kit::Action::DeterministicStateInit(deterministic_init_action) => {
                info_str.push_str(&format!(
                    "\n{:>5} {:<20}",
                    "--", "initizalize deterministic account id:"
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "deposit:", deterministic_init_action.deposit
                ));
                let state_init = match &deterministic_init_action.state_init {
                    near_kit::DeterministicAccountStateInit::V1(v1) => {
                        let mut ret = "V1".to_string();
                        ret.push_str(&format!("\n{:>31} {:<13} {:?}", "", "data", v1.data));
                        ret.push_str(&format!("\n{:>31} {:<13} {}", "", "code", match &v1.code {
                            near_kit::GlobalContractIdentifier::CodeHash(hash) => {
                                format!("use global <{hash}> code to deploy from")
                            }
                            near_kit::GlobalContractIdentifier::AccountId(account_id) => {
                                format!("use global <{account_id}> code to deploy from")
                            }
                        }));
                        ret
                    },
                };

                info_str.push_str(&format!("\n{:>18} {:<13} {}", "", "state:", state_init));
            }
            near_kit::Action::TransferToGasKey(transfer_to_gas_key) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "transfer to gas key:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", &transfer_to_gas_key.public_key
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "",
                    "deposit:",
                    transfer_to_gas_key.deposit.exact_amount_display()
                ));
            }
            near_kit::Action::WithdrawFromGasKey(withdraw_from_gas_key) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "withdraw from gas key:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", &withdraw_from_gas_key.public_key
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "",
                    "amount:",
                    withdraw_from_gas_key.amount.exact_amount_display()
                ));
            }
        }
    }
    info_str.push_str("\n ");
    info_str
}

fn print_value_successful_transaction(
    transaction_info: near_kit::FinalExecutionOutcome,
) -> String {
    let mut info_str: String = String::from('\n');
    for action in transaction_info.transaction.actions {
        match action {
            near_kit::ActionView::CreateAccount => {
                info_str.push_str(&format!(
                    "\nNew account <{}> has been successfully created.",
                    transaction_info.transaction.receiver_id,
                ));
            }
            near_kit::ActionView::DeployContract { code: _ } => {
                info_str.push_str("Contract code has been successfully deployed.");
            }
            near_kit::ActionView::FunctionCall {
                method_name,
                args: _,
                gas: _,
                deposit: _,
            } => {
                info_str.push_str(&format!(
                    "\nThe \"{}\" call to <{}> on behalf of <{}> succeeded.",
                    method_name,
                    transaction_info.transaction.receiver_id,
                    transaction_info.transaction.signer_id,
                ));
            }
            near_kit::ActionView::Transfer { deposit } => {
                info_str.push_str(&format!(
                    "\n<{}> has transferred {} to <{}> successfully.",
                    transaction_info.transaction.signer_id,
                    deposit.exact_amount_display(),
                    transaction_info.transaction.receiver_id,
                ));
            }
            near_kit::ActionView::Stake {
                stake,
                public_key: _,
            } => {
                if stake == near_token::NearToken::ZERO {
                    info_str.push_str(&format!(
                        "\nValidator <{}> successfully unstaked.",
                        transaction_info.transaction.signer_id,
                    ));
                } else {
                    info_str.push_str(&format!(
                        "\nValidator <{}> has successfully staked {}.",
                        transaction_info.transaction.signer_id,
                        stake.exact_amount_display(),
                    ));
                }
            }
            near_kit::ActionView::AddKey {
                public_key,
                access_key: _,
            } => {
                info_str.push_str(&format!(
                    "\nAdded access key = {} to {}.",
                    public_key, transaction_info.transaction.receiver_id,
                ));
            }
            near_kit::ActionView::DeleteKey { public_key } => {
                info_str.push_str(&format!(
                    "\nAccess key <{}> for account <{}> has been successfully deleted.",
                    public_key, transaction_info.transaction.signer_id,
                ));
            }
            near_kit::ActionView::DeleteAccount { beneficiary_id: _ } => {
                info_str.push_str(&format!(
                    "\nAccount <{}> has been successfully deleted.",
                    transaction_info.transaction.signer_id,
                ));
            }
            near_kit::ActionView::Delegate {
                delegate_action,
                signature: _,
            } => {
                info_str.push_str(&format!(
                    "Actions delegated for <{}> completed successfully.",
                    delegate_action.sender_id,
                ));
            }
            near_kit::ActionView::DeployGlobalContract { code: _ }
            | near_kit::ActionView::DeployGlobalContractByAccountId { code: _ } => {
                info_str.push_str("Global contract has been successfully deployed.");
            }
            near_kit::ActionView::UseGlobalContractByAccountId { account_id } => {
                info_str.push_str(&format!("Contract has been successfully deployed with the code from the global account <{account_id}>."));
            }
            near_kit::ActionView::UseGlobalContract { code_hash } => {
                info_str.push_str(&format!("Contract has been successfully deployed with the code from the global hash <{code_hash}>."));
            }
            near_kit::ActionView::DeterministicStateInit {
                code: _,
                data: _,
                deposit: _,
            } => {
                info_str.push_str(&format!(
                    "\nNew deterministic account <{}> has been successfully created.",
                    transaction_info.transaction.receiver_id,
                ));
            }
            near_kit::ActionView::TransferToGasKey {
                public_key,
                deposit,
            } => {
                info_str.push_str(&format!(
                    "\n<{}> has transferred {} to gas key <{}> successfully.",
                    transaction_info.transaction.signer_id,
                    deposit.exact_amount_display(),
                    public_key,
                ));
            }
            near_kit::ActionView::WithdrawFromGasKey { public_key, amount } => {
                info_str.push_str(&format!(
                    "\n<{}> has withdrawn {} from gas key <{}> successfully.",
                    transaction_info.transaction.signer_id,
                    amount.exact_amount_display(),
                    public_key,
                ));
            }
        }
    }
    info_str.push('\n');
    info_str
}

fn get_near_usd_exchange_rate(coingecko_url: &url::Url) -> color_eyre::Result<f64> {
    #[derive(serde::Deserialize)]
    struct CoinGeckoResponse {
        near: CoinGeckoNearData,
    }

    #[derive(serde::Deserialize)]
    struct CoinGeckoNearData {
        usd: f64,
    }

    let coingecko_exchange_rate_api_url =
        coingecko_url.join("api/v3/simple/price?ids=near&vs_currencies=usd")?;
    let mut last_error_message = String::new();

    for _ in 0..10 {
        match reqwest::blocking::get(coingecko_exchange_rate_api_url.clone()) {
            Ok(response) => match response.json::<CoinGeckoResponse>() {
                Ok(parsed_body) => return Ok(parsed_body.near.usd),
                Err(err) => {
                    last_error_message =
                        format!("Failed to parse the response from Coingecko API as JSON: {err}");
                }
            },
            Err(err) => {
                last_error_message =
                    format!("Failed to get the response from Coingecko API: {err}");
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Err(color_eyre::eyre::eyre!(last_error_message))
}

fn calculate_usd_amount(tokens: u128, price: f64) -> Option<rust_decimal::Decimal> {
    let tokens_decimal = rust_decimal::Decimal::from_u128(tokens)?;
    let price_decimal = rust_decimal::Decimal::from_f64(price)?;

    let divisor = rust_decimal::Decimal::from_u128(10u128.pow(24))?;
    let tokens_decimal = tokens_decimal / divisor;

    Some(tokens_decimal * price_decimal)
}

pub fn print_transaction_status(
    transaction_info: &near_kit::FinalExecutionOutcome,
    network_config: &crate::config::NetworkConfig,
    verbosity: crate::Verbosity,
) -> crate::CliResult {
    let near_usd_exchange_rate: Option<Result<f64, color_eyre::eyre::Error>> = network_config
        .coingecko_url
        .as_ref()
        .map(get_near_usd_exchange_rate);

    let mut result_info = String::new();
    let mut return_value = String::new();
    let mut returned_value_bytes: Vec<u8> = Vec::new();

    let result = match &transaction_info.status {
        near_kit::FinalExecutionStatus::NotStarted => {
            if let crate::Verbosity::Quiet = verbosity {
                return Ok(());
            }
            tracing::warn!(
                parent: &tracing::Span::none(),
                "The execution has not yet started."
            );
            Ok(())
        }
        near_kit::FinalExecutionStatus::Started => {
            if let crate::Verbosity::Quiet = verbosity {
                return Ok(());
            }
            tracing::warn!(
                parent: &tracing::Span::none(),
                "The execution has started and still going."
            );
            Ok(())
        }
        near_kit::FinalExecutionStatus::Failure(tx_execution_error) => {
            Err(color_eyre::eyre::eyre!("{}", tx_execution_error))
        }
        near_kit::FinalExecutionStatus::SuccessValue(base64_result) => {
            let bytes_result = base64::engine::general_purpose::STANDARD.decode(base64_result)
                .unwrap_or_default();
            if let crate::Verbosity::Quiet = verbosity {
                std::io::stdout().write_all(&bytes_result)?;
                return Ok(());
            };
            returned_value_bytes = bytes_result.clone();
            return_value = if bytes_result.is_empty() {
                "Empty return value".to_string()
            } else if let Ok(json_result) =
                serde_json::from_slice::<serde_json::Value>(&bytes_result)
            {
                serde_json::to_string_pretty(&json_result)?
            } else if let Ok(string_result) = String::from_utf8(bytes_result) {
                string_result
            } else {
                "The returned value is not printable (binary data)".to_string()
            };
            result_info.push_str(&return_value);
            Ok(())
        }
    };

    let mut transaction_execution_info = String::new();
    let mut total_gas_burnt = transaction_info.transaction_outcome.outcome.gas_burnt;
    let mut total_tokens_burnt = transaction_info.transaction_outcome.outcome.tokens_burnt;

    transaction_execution_info.push_str(&format!("\nGas burned: {total_gas_burnt}"));

    transaction_execution_info.push_str(&format!(
        "\nTransaction fee: {}{}",
        total_tokens_burnt.exact_amount_display(),
        match near_usd_exchange_rate {
            Some(Ok(exchange_rate)) => calculate_usd_amount(total_tokens_burnt.as_yoctonear(), exchange_rate).map_or_else(
                || format!(" (USD equivalent is too big to be displayed, using ${exchange_rate:.2} USD/NEAR exchange rate)"),
                |amount| format!(" (approximately ${amount:.8} USD, using ${exchange_rate:.2} USD/NEAR exchange rate)")
            ),
            Some(Err(err)) => format!(" (USD equivalent is unavailable due to an error: {err})"),
            None => String::new(),
        }
    ));

    transaction_execution_info.push_str(&format!(
        "\nTransaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n ",
        id=transaction_info.transaction_outcome.id,
        path=network_config.explorer_transaction_url
    ));

    if result_info.is_empty() {
        tracing::error!(
            parent: &tracing::Span::none(),
            "Transaction failed{}",
            crate::common::indent_payload(&transaction_execution_info)
        );
    } else {
        tracing::info!(
            parent: &tracing::Span::none(),
            "Transaction Execution Info:{}",
            crate::common::indent_payload(&transaction_execution_info)
        );
    }

    let mut logs_info = String::new();

    for receipt in &transaction_info.receipts_outcome {
        total_gas_burnt = total_gas_burnt
            .checked_add(receipt.outcome.gas_burnt)
            .context("overflow while adding transaction status total gas")?;
        total_tokens_burnt = total_tokens_burnt
            .checked_add(receipt.outcome.tokens_burnt)
            .context("overflow while adding transaction status total tokens burnt")?;

        if receipt.outcome.logs.is_empty() {
            logs_info.push_str(&format!(
                "\nLogs [{}]:   No logs",
                receipt.outcome.executor_id
            ));
        } else {
            logs_info.push_str(&format!("\nLogs [{}]:", receipt.outcome.executor_id));
            logs_info.push_str(&format!("\n  {}", receipt.outcome.logs.join("\n  ")));
        };
    }

    for action in &transaction_info.transaction.actions {
        if let near_kit::ActionView::FunctionCall { .. } = action {
            tracing::info!(
                parent: &tracing::Span::none(),
                "Function execution logs:{}",
                crate::common::indent_payload(&format!("{logs_info}\n "))
            );
            if returned_value_bytes.is_empty() {
                tracing::info!(
                    parent: &tracing::Span::none(),
                    "Function execution return value:\n{}",
                    crate::common::indent_payload("Empty return value\n ")
                );
            } else {
                suspend_tracing_indicatif(|| {
                    eprintln!("\nFunction execution return value (printed to stdout):")
                });
                suspend_tracing_indicatif(|| println!("{return_value}"));
            }
        }
    }

    if !result_info.is_empty() {
        suspend_tracing_indicatif(|| {
            eprintln!(
                "{}",
                print_value_successful_transaction(transaction_info.clone(),)
            )
        });
    }

    result
}

pub fn save_access_key_to_keychain_or_save_to_legacy_keychain(
    network_config: crate::config::NetworkConfig,
    credentials_home_dir: std::path::PathBuf,
    key_pair_properties_buf: &str,
    public_key_str: &str,
    account_id: &str,
) -> color_eyre::eyre::Result<String> {
    match save_access_key_to_keychain(
        network_config.clone(),
        key_pair_properties_buf,
        public_key_str,
        account_id,
    ) {
        Ok(message) => Ok(message),
        Err(err) => {
            let info_str = format!(
                "{}\n{}\n",
                format!("Failed to save the access key <{public_key_str}> to the keychain.\n{err}")
                    .red(),
                "The data for the access key will be stored in the legacy keychain.".red()
            );
            tracing::warn!(
                parent: &tracing::Span::none(),
                "\n{}",
                indent_payload(&info_str)
            );
            save_access_key_to_legacy_keychain(
                network_config.clone(),
                credentials_home_dir,
                key_pair_properties_buf,
                public_key_str,
                account_id,
            )
        }
    }
}

pub fn save_access_key_to_keychain(
    network_config: crate::config::NetworkConfig,
    key_pair_properties_buf: &str,
    public_key_str: &str,
    account_id: &str,
) -> color_eyre::eyre::Result<String> {
    let service_name = std::borrow::Cow::Owned(format!(
        "near-{}-{}",
        network_config.network_name, account_id
    ));

    keyring::Entry::new(&service_name, &format!("{account_id}:{public_key_str}"))
        .wrap_err("Failed to open keychain")?
        .set_password(key_pair_properties_buf)
        .wrap_err("Failed to save password to keychain. You may need to install the secure keychain package by following this instruction: https://github.com/jaraco/keyring#using-keyring-on-headless-linux-systems")?;

    Ok("The data for the access key is saved in the keychain".to_string())
}

pub fn save_access_key_to_legacy_keychain(
    network_config: crate::config::NetworkConfig,
    credentials_home_dir: std::path::PathBuf,
    key_pair_properties_buf: &str,
    public_key_str: &str,
    account_id: &str,
) -> color_eyre::eyre::Result<String> {
    let dir_name = network_config.network_name.as_str();
    let file_with_key_name: std::path::PathBuf =
        format!("{}.json", public_key_str.replace(':', "_")).into();
    let mut path_with_key_name = std::path::PathBuf::from(&credentials_home_dir);
    path_with_key_name.push(dir_name);
    path_with_key_name.push(account_id);
    std::fs::create_dir_all(&path_with_key_name)?;
    path_with_key_name.push(file_with_key_name);
    let message_1 = if path_with_key_name.exists() {
        format!(
            "The file: {} already exists! Therefore it was not overwritten.",
            &path_with_key_name.display()
        )
    } else {
        std::fs::File::create(&path_with_key_name)
            .wrap_err_with(|| format!("Failed to create file: {path_with_key_name:?}"))?
            .write(key_pair_properties_buf.as_bytes())
            .wrap_err_with(|| format!("Failed to write to file: {path_with_key_name:?}"))?;
        format!(
            "The data for the access key is saved in a file {}",
            &path_with_key_name.display()
        )
    };

    let file_with_account_name: std::path::PathBuf = format!("{account_id}.json").into();
    let mut path_with_account_name = std::path::PathBuf::from(&credentials_home_dir);
    path_with_account_name.push(dir_name);
    path_with_account_name.push(file_with_account_name);
    if path_with_account_name.exists() {
        Ok(format!(
            "{}\nThe file: {} already exists! Therefore it was not overwritten.",
            message_1,
            &path_with_account_name.display()
        ))
    } else {
        std::fs::File::create(&path_with_account_name)
            .wrap_err_with(|| format!("Failed to create file: {path_with_account_name:?}"))?
            .write(key_pair_properties_buf.as_bytes())
            .wrap_err_with(|| format!("Failed to write to file: {path_with_account_name:?}"))?;
        Ok(format!(
            "{}\nThe data for the access key is saved in a file {}",
            message_1,
            &path_with_account_name.display()
        ))
    }
}

pub fn try_external_subcommand_execution(error: clap::Error) -> CliResult {
    let (subcommand, args) = {
        let mut args = std::env::args().skip(1);
        let subcommand = args
            .next()
            .ok_or_else(|| color_eyre::eyre::eyre!("subcommand is not provided"))?;
        (subcommand, args.collect::<Vec<String>>())
    };
    let is_top_level_command_known = crate::commands::TopLevelCommandDiscriminants::iter()
        .map(|x| format!("{:?}", &x).to_lowercase())
        .any(|x| x == subcommand);
    if is_top_level_command_known {
        error.exit()
    }
    let subcommand_exe = format!("near-{}{}", subcommand, std::env::consts::EXE_SUFFIX);

    let path = path_directories()
        .iter()
        .map(|dir| dir.join(&subcommand_exe))
        .find(|file| is_executable(file));

    let command = path.ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "{} command or {} extension does not exist",
            subcommand,
            subcommand_exe
        )
    })?;

    let err = match cargo_util::ProcessBuilder::new(command)
        .args(&args)
        .exec_replace()
    {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };

    if let Some(perr) = err.downcast_ref::<cargo_util::ProcessError>()
        && let Some(code) = perr.code
    {
        return Err(color_eyre::eyre::eyre!("perror occurred, code: {}", code));
    }
    Err(color_eyre::eyre::eyre!(err))
}

fn is_executable<P: AsRef<std::path::Path>>(path: P) -> bool {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::prelude::*;
        std::fs::metadata(path)
            .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(target_family = "windows")]
    path.as_ref().is_file()
}

fn path_directories() -> Vec<std::path::PathBuf> {
    if let Some(val) = std::env::var_os("PATH") {
        std::env::split_paths(&val).collect()
    } else {
        Vec::new()
    }
}

pub fn get_delegated_validator_list_from_mainnet(
    network_connection: &linked_hash_map::LinkedHashMap<String, crate::config::NetworkConfig>,
) -> color_eyre::eyre::Result<std::collections::BTreeSet<near_kit::AccountId>> {
    let network_config = network_connection
        .get("mainnet")
        .wrap_err("There is no 'mainnet' network in your configuration.")?;

    let epoch_validator_info = blocking_validators(network_config)?;

    Ok(epoch_validator_info
        .current_proposals
        .into_iter()
        .map(|proposal| proposal.into_v1().account_id)
        .chain(
            epoch_validator_info
                .current_validators
                .into_iter()
                .map(|current_validator| current_validator.account_id),
        )
        .chain(
            epoch_validator_info
                .next_validators
                .into_iter()
                .map(|next_validator| next_validator.account_id),
        )
        .collect())
}

#[tracing::instrument(
    name = "Retrieving a list of delegated validators from \"mainnet\" ...",
    skip_all
)]
pub fn get_used_delegated_validator_list(
    config: &crate::config::Config,
) -> color_eyre::eyre::Result<VecDeque<near_kit::AccountId>> {
    tracing::info!(target: "near_teach_me", "Retrieving a list of delegated validators from \"mainnet\" ...");
    let used_account_list: VecDeque<UsedAccount> =
        get_used_account_list(&config.credentials_home_dir);
    let mut delegated_validator_list =
        get_delegated_validator_list_from_mainnet(&config.network_connection)?;
    let mut used_delegated_validator_list: VecDeque<near_kit::AccountId> =
        VecDeque::new();

    for used_account in used_account_list {
        if delegated_validator_list.remove(&used_account.account_id) {
            used_delegated_validator_list.push_back(used_account.account_id);
        }
    }

    used_delegated_validator_list.extend(delegated_validator_list);
    Ok(used_delegated_validator_list)
}

pub fn input_staking_pool_validator_account_id(
    config: &crate::config::Config,
) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
    let used_delegated_validator_list = get_used_delegated_validator_list(config)?
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
    let validator_account_id_str = match Text::new("What is delegated validator account ID?")
        .with_autocomplete(move |val: &str| {
            Ok(used_delegated_validator_list
                .iter()
                .filter(|s| s.contains(val))
                .cloned()
                .collect())
        })
        .with_validator(|account_id_str: &str| {
            match near_kit::AccountId::validate(account_id_str) {
                Ok(_) => Ok(inquire::validator::Validation::Valid),
                Err(err) => Ok(inquire::validator::Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom(format!("Invalid account ID: {err}")),
                )),
            }
        })
        .prompt()
    {
        Ok(value) => value,
        Err(
            inquire::error::InquireError::OperationCanceled
            | inquire::error::InquireError::OperationInterrupted,
        ) => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let validator_account_id =
        crate::types::account_id::AccountId::from_str(&validator_account_id_str)?;
    update_used_account_list_as_non_signer(
        &config.credentials_home_dir,
        validator_account_id.as_ref(),
    );
    Ok(Some(validator_account_id))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StakingPoolInfo {
    pub validator_id: near_kit::AccountId,
    pub fee: Option<RewardFeeFraction>,
    pub delegators: Option<u64>,
    pub stake: near_token::NearToken,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct RewardFeeFraction {
    pub numerator: u32,
    pub denominator: u32,
}

#[tracing::instrument(name = "Getting a list of validators ...", skip_all)]
pub fn get_validator_list(
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<Vec<StakingPoolInfo>> {
    tracing::info!(target: "near_teach_me", "Getting a list of validators ...");

    let validators_stake = get_validators_stake(network_config)?;

    let client = network_config.client();
    let rpc = client.rpc();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    let concurrency = 10;

    let mut validator_list = runtime.block_on(
        futures::stream::iter(validators_stake.iter())
            .map(|(validator_account_id, stake)| async {
                get_staking_pool_info(
                    rpc,
                    validator_account_id.clone(),
                    *stake,
                )
                .await
            })
            .buffer_unordered(concurrency)
            .try_collect::<Vec<_>>(),
    )?;
    validator_list.sort_by(|a, b| b.stake.cmp(&a.stake));
    Ok(validator_list)
}

#[derive(Debug, serde::Deserialize)]
struct StakingPool {
    pool_id: near_kit::AccountId,
}

#[derive(Debug, serde::Deserialize)]
struct StakingResponse {
    pools: Vec<StakingPool>,
}

#[tracing::instrument(name = "Getting historically delegated staking pools ...", skip_all)]
pub fn fetch_historically_delegated_staking_pools(
    fastnear_url: &url::Url,
    account_id: &near_kit::AccountId,
) -> color_eyre::Result<std::collections::BTreeSet<near_kit::AccountId>> {
    tracing::info!(target: "near_teach_me", "Getting historically delegated staking pools ...");
    let request =
        reqwest::blocking::get(fastnear_url.join(&format!("v1/account/{account_id}/staking"))?)?;
    let response: StakingResponse = request.json()?;

    Ok(response
        .pools
        .into_iter()
        .map(|pool| pool.pool_id)
        .collect())
}

#[tracing::instrument(name = "Getting currently active staking pools ...", skip_all)]
pub fn fetch_currently_active_staking_pools(
    network_config: &crate::config::NetworkConfig,
    staking_pools_factory_account_id: &near_kit::AccountId,
) -> color_eyre::Result<std::collections::BTreeSet<near_kit::AccountId>> {
    tracing::info!(target: "near_teach_me", "Getting currently active staking pools ...");

    // near-kit does not expose a view_state helper, so use the raw RPC escape hatch.
    #[derive(serde::Deserialize)]
    struct ViewStateItem {
        value: String, // base64-encoded
    }
    #[derive(serde::Deserialize)]
    struct ViewStateResult {
        values: Vec<ViewStateItem>,
    }

    let prefix_base64 =
        base64::engine::general_purpose::STANDARD.encode(b"se");

    let result: ViewStateResult = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config.client().rpc().call(
                "query",
                serde_json::json!({
                    "request_type": "view_state",
                    "finality": "final",
                    "account_id": staking_pools_factory_account_id.to_string(),
                    "prefix_base64": prefix_base64,
                    "include_proof": false,
                }),
            ),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))?;

    Ok(result
        .values
        .into_iter()
        .filter_map(|item| {
            let bytes = base64::engine::general_purpose::STANDARD.decode(&item.value).ok()?;
            borsh::from_slice(&bytes).ok()
        })
        .collect())
}

#[tracing::instrument(name = "Getting a stake of validators ...", skip_all)]
pub fn get_validators_stake(
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<
    std::collections::HashMap<near_kit::AccountId, near_token::NearToken>,
> {
    tracing::info!(target: "near_teach_me", "Getting a stake of validators ...");
    let epoch_validator_info = blocking_validators(network_config)?;

    Ok(epoch_validator_info
        .current_proposals
        .into_iter()
        .map(|proposal| {
            let v1 = proposal.into_v1();
            (v1.account_id, v1.stake)
        })
        .chain(epoch_validator_info.current_validators.into_iter().map(
            |current_epoch_validator_info| {
                (
                    current_epoch_validator_info.account_id,
                    current_epoch_validator_info.stake,
                )
            },
        ))
        .chain(
            epoch_validator_info
                .next_validators
                .into_iter()
                .map(|next_epoch_validator_info| {
                    (
                        next_epoch_validator_info.account_id,
                        next_epoch_validator_info.stake,
                    )
                }),
        )
        .collect())
}

async fn get_staking_pool_info(
    rpc: &near_kit::RpcClient,
    validator_account_id: near_kit::AccountId,
    stake: near_token::NearToken,
) -> color_eyre::Result<StakingPoolInfo> {
    let fee = match rpc
        .view_function(
            &validator_account_id,
            "get_reward_fee_fraction",
            &[],
            near_kit::BlockReference::final_(),
        )
        .await
    {
        Ok(result) => Some(
            result
                .json::<RewardFeeFraction>()
                .wrap_err(
                    "Failed to parse return value of view function call for RewardFeeFraction.",
                )?,
        ),
        Err(
            near_kit::RpcError::ContractNotDeployed(_)
            | near_kit::RpcError::ContractExecution { .. },
        ) => None,
        Err(err) => return Err(err.into()),
    };

    let delegators = match rpc
        .view_function(
            &validator_account_id,
            "get_number_of_accounts",
            &[],
            near_kit::BlockReference::final_(),
        )
        .await
    {
        Ok(result) => Some(
            result
                .json::<u64>()
                .wrap_err("Failed to parse return value of view function call for u64.")?,
        ),
        Err(
            near_kit::RpcError::ContractNotDeployed(_)
            | near_kit::RpcError::ContractExecution { .. },
        ) => None,
        Err(err) => return Err(err.into()),
    };

    Ok(StakingPoolInfo {
        validator_id: validator_account_id.clone(),
        fee,
        delegators,
        stake,
    })
}

pub fn display_account_info(
    account_id: &near_kit::AccountId,
    delegated_stake: color_eyre::Result<
        std::collections::BTreeMap<near_kit::AccountId, near_token::NearToken>,
    >,
    account_view: &near_kit::AccountView,
    access_key_list: Option<&near_kit::AccessKeyListView>,
    optional_account_profile: Option<&crate::types::socialdb::AccountProfile>,
) {
    eprintln!();
    let mut table: Table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);

    profile_table(
        &account_view.block_hash,
        &account_view.block_height,
        account_id,
        optional_account_profile,
        &mut table,
    );

    table.add_row(prettytable::row![
        Fg->"Native account balance",
        Fy->account_view.amount.exact_amount_display()
    ]);
    table.add_row(prettytable::row![
        Fg->"Validator stake",
        Fy->account_view.locked.exact_amount_display()
    ]);

    match delegated_stake {
        Ok(delegated_stake) => {
            for (validator_id, stake) in delegated_stake {
                table.add_row(prettytable::row![
                    Fg->format!("Delegated stake with <{validator_id}>"),
                    Fy->stake.exact_amount_display()
                ]);
            }
        }
        Err(err) => {
            table.add_row(prettytable::row![
                Fg->"Delegated stake",
                Fr->err
            ]);
        }
    }

    table.add_row(prettytable::row![
        Fg->"Storage used by the account",
        Fy->bytesize::ByteSize(account_view.storage_usage),
    ]);

    let (table_code_message, contract_status) = match (
        &account_view.code_hash,
        &account_view.global_contract_account_id,
        &account_view.global_contract_hash,
    ) {
        (_, Some(global_contract_account_id), None) => (
            "Global Contract (by Account Id)",
            global_contract_account_id.to_string(),
        ),
        (_, None, Some(global_contract_hash)) => (
            "Global Contract (by Hash: SHA-256 checksum hex)",
            hex::encode(global_contract_hash.as_bytes()),
        ),
        (hash, None, None) if hash.is_zero() => {
            ("Contract", "No contract code".to_string())
        }
        (code_hash, None, None) => (
            "Local Contract (SHA-256 checksum hex)",
            hex::encode(code_hash.as_bytes()),
        ),
        (code_hash, global_account_id, global_hash) => (
            "Contract",
            format!(
                "Invalid account contract state. Please contact the developers. code_hash: <{}>, global_account_id: <{:?}>, global_hash: <{:?}>",
                hex::encode(code_hash.as_bytes()),
                global_account_id,
                global_hash.as_ref().map(|h| hex::encode(h.as_bytes()))
            )
            .red().to_string()
        ),
    };

    table.add_row(prettytable::row![
        Fg->table_code_message,
        Fy->contract_status
    ]);

    let access_keys_summary = if let Some(info) = access_key_list {
        let keys = &info.keys;
        if keys.is_empty() {
            "Account is locked (no access keys)".to_string()
        } else {
            let full_access_keys_count = keys
                .iter()
                .filter(|access_key| {
                    matches!(
                        access_key.access_key.permission,
                        near_kit::AccessKeyPermissionView::FullAccess
                    )
                })
                .count();
            format!(
                "{} full access keys and {} function-call-only access keys",
                full_access_keys_count,
                keys.len() - full_access_keys_count
            )
        }
    } else {
        "Warning: Failed to retrieve access keys. Retry later."
            .red()
            .to_string()
    };

    table.add_row(prettytable::row![
        Fg->"Access keys",
        Fy->access_keys_summary
    ]);
    table.printstd();
}

pub fn display_account_profile(
    viewed_at_block_hash: &near_kit::CryptoHash,
    viewed_at_block_height: &u64,
    account_id: &near_kit::AccountId,
    optional_account_profile: Option<&crate::types::socialdb::AccountProfile>,
) {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);
    profile_table(
        viewed_at_block_hash,
        viewed_at_block_height,
        account_id,
        optional_account_profile,
        &mut table,
    );
    table.printstd();
}

fn profile_table(
    viewed_at_block_hash: &near_kit::CryptoHash,
    viewed_at_block_height: &u64,
    account_id: &near_kit::AccountId,
    optional_account_profile: Option<&crate::types::socialdb::AccountProfile>,
    table: &mut Table,
) {
    if let Some(account_profile) = optional_account_profile {
        if let Some(name) = &account_profile.profile.name {
            table.add_row(prettytable::row![
                Fy->format!("{account_id} ({name})"),
                format!("At block #{}\n({})", viewed_at_block_height, viewed_at_block_hash)
            ]);
        } else {
            table.add_row(prettytable::row![
                Fy->account_id,
                format!("At block #{}\n({})", viewed_at_block_height, viewed_at_block_hash)
            ]);
        }
        if let Some(image) = &account_profile.profile.image {
            if let Some(url) = &image.url {
                table.add_row(prettytable::row![
                    Fg->"Image (url)",
                    Fy->url
                ]);
            }
            if let Some(ipfs_cid) = &image.ipfs_cid {
                table.add_row(prettytable::row![
                    Fg->"Image (ipfs_cid)",
                    Fy->ipfs_cid
                ]);
            }
        }
        if let Some(background_image) = &account_profile.profile.background_image {
            if let Some(url) = &background_image.url {
                table.add_row(prettytable::row![
                    Fg->"Background image (url)",
                    Fy->url
                ]);
            }
            if let Some(ipfs_cid) = &background_image.ipfs_cid {
                table.add_row(prettytable::row![
                    Fg->"Background image (ipfs_cid)",
                    Fy->ipfs_cid
                ]);
            }
        }
        if let Some(description) = &account_profile.profile.description {
            table.add_row(prettytable::row![
                Fg->"Description",
                Fy->format!("{}", description)
            ]);
        }
        if let Some(linktree) = &account_profile.profile.linktree {
            table.add_row(prettytable::row![
                Fg->"Linktree",
                Fy->""
            ]);
            for (key, optional_value) in linktree.iter() {
                if let Some(value) = &optional_value {
                    if key == "github" {
                        table.add_row(prettytable::row![
                            Fg->"",
                            Fy->format!("https://github.com/{value}")
                        ]);
                    } else if key == "twitter" {
                        table.add_row(prettytable::row![
                            Fg->"",
                            Fy->format!("https://twitter.com/{value}")
                        ]);
                    } else if key == "telegram" {
                        table.add_row(prettytable::row![
                            Fg->"",
                            Fy->format!("https://t.me/{value}")
                        ]);
                    }
                }
            }
        }
        if let Some(tags) = &account_profile.profile.tags {
            let keys = tags.keys().cloned().collect::<Vec<String>>().join(", ");
            table.add_row(prettytable::row![
                Fg->"Tags",
                Fy->keys
            ]);
        }
    } else {
        table.add_row(prettytable::row![
            Fy->account_id,
            format!("At block #{}\n({})", viewed_at_block_height, viewed_at_block_hash)
        ]);
        table.add_row(prettytable::row![
            Fd->"NEAR Social profile unavailable",
            Fd->format!("The profile can be edited at {}\nor using the cli command: {}\n(https://github.com/bos-cli-rs/bos-cli-rs)",
                "https://near.social".blue(),
                "bos social-db manage-profile".blue()
            )
        ]);
    }
}

pub fn display_access_key_list(access_keys: &[near_kit::AccessKeyInfoView]) {
    let mut table = Table::new();
    table.set_titles(prettytable::row![Fg=>"#", "Public Key", "Nonce", "Permissions"]);

    for (index, access_key) in access_keys.iter().enumerate() {
        let permissions_message = match &access_key.access_key.permission {
            near_kit::AccessKeyPermissionView::FullAccess => "full access".to_owned(),
            near_kit::AccessKeyPermissionView::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            } => {
                let allowance_message = match allowance {
                    Some(allowance) => {
                        format!("with an allowance of {}", allowance.exact_amount_display())
                    }
                    None => "with no limit".to_string(),
                };
                if method_names.is_empty() {
                    format!("do any function calls on {receiver_id} {allowance_message}")
                } else {
                    format!(
                        "only do {method_names:?} function calls on {receiver_id} {allowance_message}"
                    )
                }
            }
            near_kit::AccessKeyPermissionView::GasKeyFunctionCall {
                balance,
                num_nonces,
                allowance: _,
                receiver_id,
                method_names,
            } => {
                let methods = if method_names.is_empty() {
                    "any methods".to_string()
                } else {
                    format!("{method_names:?}")
                };
                format!(
                    "gas key for function calls on {receiver_id} ({methods}), balance: {}, nonces: {num_nonces}",
                    balance.exact_amount_display()
                )
            }
            near_kit::AccessKeyPermissionView::GasKeyFullAccess {
                balance,
                num_nonces,
            } => {
                format!(
                    "gas key with full access, balance: {}, nonces: {num_nonces}",
                    balance.exact_amount_display()
                )
            }
        };

        table.add_row(prettytable::row![
            Fg->index + 1,
            access_key.public_key,
            access_key.access_key.nonce,
            permissions_message
        ]);
    }

    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();
}

/// Interactive prompt for network name.
///
/// If account_ids is provided, show the network connections that are more
/// relevant at the top of the list.
pub fn input_network_name(
    config: &crate::config::Config,
    account_ids: &[near_kit::AccountId],
) -> color_eyre::eyre::Result<Option<String>> {
    if config.network_connection.len() == 1 {
        return Ok(config.network_names().pop());
    }
    let variants = if !account_ids.is_empty() {
        let (mut matches, non_matches): (Vec<_>, Vec<_>) = config
            .network_connection
            .iter()
            .partition(|(_, network_config)| {
                // We use `linkdrop_account_id` as a heuristic to determine if
                // the accounts are on the same network. In the future, we
                // might consider to have a better way to do this.
                network_config
                    .linkdrop_account_id
                    .as_ref()
                    .is_some_and(|linkdrop_account_id| {
                        account_ids.iter().any(|account_id| {
                            account_id.as_str().ends_with(linkdrop_account_id.as_str())
                        })
                    })
            });
        let variants = if matches.is_empty() {
            non_matches
        } else {
            matches.extend(non_matches);
            matches
        };
        variants.into_iter().map(|(k, _)| k).collect()
    } else {
        config.network_connection.keys().collect()
    };

    let select_submit = Select::new("What is the name of the network?", variants).prompt();
    match select_submit {
        Ok(value) => Ok(Some(value.clone())),
        Err(
            inquire::error::InquireError::OperationCanceled
            | inquire::error::InquireError::OperationInterrupted,
        ) => Ok(None),
        Err(err) => Err(err.into()),
    }
}


/// Blocking helper: fetch an access key via near-kit.
///
/// Returns the near-kit `AccessKeyView` which contains `nonce`, `block_hash`,
/// `block_height`, and `permission`.
#[tracing::instrument(name = "Getting access key information:", skip_all)]
pub fn blocking_view_access_key(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    public_key: &near_kit::PublicKey,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::AccessKeyView> {
    tracing::Span::current().pb_set_message(&format!(
        "public key {public_key} on account <{account_id}>..."
    ));
    tracing::info!(target: "near_teach_me", "Getting access key information for public key {public_key} on account <{account_id}>...");
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to get an access key details for public key {} on account <{}>, learn more https://docs.near.org/api/rpc/access-keys#view-access-key",
        public_key,
        account_id
    );

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .view_access_key(account_id, public_key, block_reference),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: fetch all access keys for an account via near-kit.
#[tracing::instrument(name = "Getting a list of", skip_all)]
pub fn blocking_view_access_key_list(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::AccessKeyListView> {
    tracing::Span::current()
        .pb_set_message(&format!("access keys on account <{account_id}>..."));
    tracing::info!(target: "near_teach_me", "Getting a list of access keys on account <{account_id}>...");
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to get a list of keys for account <{}>, learn more https://docs.near.org/api/rpc/access-keys#view-access-key-list",
        account_id
    );

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .view_access_key_list(account_id, block_reference),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: fetch account info via near-kit.
#[tracing::instrument(name = "Getting information about", skip_all)]
pub fn blocking_view_account(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::AccountView> {
    tracing::Span::current().pb_set_message(&format!("account <{account_id}>..."));
    tracing::info!(target: "near_teach_me", "Getting information about account <{account_id}>...");
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to query information about account <{}>, learn more https://docs.near.org/api/rpc/contracts#view-account",
        account_id
    );

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .view_account(account_id, block_reference),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: call a view function via near-kit.
#[tracing::instrument(name = "Getting the result of executing", skip_all)]
pub fn blocking_view_function(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    function_name: &str,
    args: Vec<u8>,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::ViewFunctionResult> {
    tracing::Span::current().pb_set_message(&format!(
        "a read-only function '{function_name}' of the <{account_id}> contract ..."
    ));
    tracing::info!(target: "near_teach_me", "Getting the result of executing a read-only function '{function_name}' of the <{account_id}> contract ...");
    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "I am making HTTP call to NEAR JSON RPC to call a read-only function `{}` on `{}` account, learn more https://docs.near.org/api/rpc/contracts#call-a-contract-function",
        function_name,
        account_id
    );

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .view_function(account_id, function_name, &args, block_reference),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: fetch validator info via near-kit.
pub fn blocking_validators(
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<near_kit::EpochValidatorInfo> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(network_config.client().rpc().validators(None))
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: fetch block info via near-kit.
pub fn blocking_block(
    network_config: &crate::config::NetworkConfig,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<near_kit::BlockView> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(network_config.client().rpc().block(block_reference))
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: send a signed transaction via near-kit.
pub fn blocking_send_tx(
    network_config: &crate::config::NetworkConfig,
    signed_transaction: &near_kit::SignedTransaction,
    wait_until: near_kit::TxExecutionStatus,
) -> Result<near_kit::RawTransactionResponse, near_kit::RpcError> {
    let tx_bytes = borsh::to_vec(signed_transaction)
        .expect("SignedTransaction borsh serialization should never fail");
    let tx_base64 = base64::engine::general_purpose::STANDARD.encode(&tx_bytes);

    let params = serde_json::json!({
        "signed_tx_base64": tx_base64,
        "wait_until": wait_until.as_str(),
    });

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .call::<_, near_kit::RawTransactionResponse>("send_tx", params),
        )
}

/// Blocking helper: get transaction status via near-kit.
pub fn blocking_tx_status(
    network_config: &crate::config::NetworkConfig,
    tx_hash: &near_kit::CryptoHash,
    sender_id: &near_kit::AccountId,
    wait_until: near_kit::TxExecutionStatus,
) -> Result<near_kit::RawTransactionResponse, near_kit::RpcError> {
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .tx_status(tx_hash, sender_id, wait_until),
        )
}



/// Blocking helper: query view state via near-kit escape hatch.
///
/// near-kit does not have a dedicated `view_state` method, so we use the
/// raw RPC call escape hatch with the standard `query` endpoint.
/// Returns a JSON value that must be extracted by the caller.
pub fn blocking_view_state(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    prefix: &[u8],
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<serde_json::Value> {
    let mut params = serde_json::json!({
        "request_type": "view_state",
        "account_id": account_id.to_string(),
        "prefix_base64": base64::engine::general_purpose::STANDARD.encode(prefix),
        "include_proof": false,
    });
    // Merge block reference params
    if let serde_json::Value::Object(block_params) = block_reference.to_rpc_params() {
        if let serde_json::Value::Object(map) = &mut params {
            map.extend(block_params);
        }
    }

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .call::<_, serde_json::Value>("query", params),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

/// Blocking helper: query view code via near-kit escape hatch.
///
/// near-kit does not have a dedicated `view_code` method, so we use the
/// raw RPC call escape hatch.
/// Returns a JSON value that must be extracted by the caller.
pub fn blocking_view_code(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_kit::AccountId,
    block_reference: near_kit::BlockReference,
) -> color_eyre::eyre::Result<serde_json::Value> {
    let mut params = serde_json::json!({
        "request_type": "view_code",
        "account_id": account_id.to_string(),
    });
    if let serde_json::Value::Object(block_params) = block_reference.to_rpc_params() {
        if let serde_json::Value::Object(map) = &mut params {
            map.extend(block_params);
        }
    }

    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(
            network_config
                .client()
                .rpc()
                .call::<_, serde_json::Value>("query", params),
        )
        .map_err(|err| color_eyre::eyre::eyre!("{}", err))
}

pub fn indent_payload(s: &str) -> String {
    use std::fmt::Write;

    let mut indented_string = String::new();
    indenter::indented(&mut indented_string)
        .with_str("│    ")
        .write_str(s)
        .ok();
    indented_string
}

#[easy_ext::ext(CallResultExt)]
pub impl near_kit::ViewFunctionResult {
    fn parse_result_from_json<T>(&self) -> Result<T, color_eyre::eyre::Error>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        serde_json::from_slice(&self.result).wrap_err_with(|| {
            format!(
                "Failed to parse view-function call return value: {}",
                String::from_utf8_lossy(&self.result)
            )
        })
    }

    fn print_logs(&self) {
        let mut info_str = String::new();
        if self.logs.is_empty() {
            info_str.push_str("\nNo logs")
        } else {
            info_str.push_str(&format!("\n  {}", self.logs.join("\n  ")));
        }
        tracing::info!(
            parent: &tracing::Span::none(),
            "Logs:{}",
            indent_payload(&info_str)
        );
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UsedAccount {
    pub account_id: near_kit::AccountId,
    pub used_as_signer: bool,
}

fn get_used_account_list_path(credentials_home_dir: &std::path::Path) -> std::path::PathBuf {
    credentials_home_dir.join("accounts.json")
}

pub fn create_used_account_list_from_legacy_keychain(
    credentials_home_dir: &std::path::Path,
) -> color_eyre::eyre::Result<()> {
    let mut used_account_list: std::collections::BTreeSet<near_kit::AccountId> =
        std::collections::BTreeSet::new();
    let read_dir =
        |dir: &std::path::Path| dir.read_dir().map(Iterator::flatten).into_iter().flatten();
    for network_connection_dir in read_dir(credentials_home_dir) {
        for entry in read_dir(&network_connection_dir.path()) {
            match (entry.path().file_stem(), entry.path().extension()) {
                (Some(file_stem), Some(extension)) if extension == "json" => {
                    if let Ok(account_id) = file_stem.to_string_lossy().parse() {
                        used_account_list.insert(account_id);
                    }
                }
                _ if entry.path().is_dir() => {
                    if let Ok(account_id) = entry.file_name().to_string_lossy().parse() {
                        used_account_list.insert(account_id);
                    }
                }
                _ => {}
            }
        }
    }

    let used_account_list_path = get_used_account_list_path(credentials_home_dir);
    std::fs::create_dir_all(credentials_home_dir)?;
    if !used_account_list_path.exists() {
        std::fs::File::create(&used_account_list_path)
            .wrap_err_with(|| format!("Failed to create file: {:?}", &used_account_list_path))?;
    }
    if !used_account_list.is_empty() {
        let used_account_list_buf = serde_json::to_string(
            &used_account_list
                .into_iter()
                .map(|account_id| UsedAccount {
                    account_id,
                    used_as_signer: true,
                })
                .collect::<Vec<_>>(),
        )?;
        std::fs::write(&used_account_list_path, used_account_list_buf).wrap_err_with(|| {
            format!(
                "Failed to write to file: {}",
                used_account_list_path.display()
            )
        })?;
    }
    Ok(())
}

pub fn update_used_account_list_as_signer(
    credentials_home_dir: &std::path::Path,
    account_id: &near_kit::AccountId,
) {
    let account_is_signer = true;
    update_used_account_list(credentials_home_dir, account_id, account_is_signer);
}

pub fn update_used_account_list_as_non_signer(
    credentials_home_dir: &std::path::Path,
    account_id: &near_kit::AccountId,
) {
    let account_is_signer = false;
    update_used_account_list(credentials_home_dir, account_id, account_is_signer);
}

fn update_used_account_list(
    credentials_home_dir: &std::path::Path,
    account_id: &near_kit::AccountId,
    account_is_signer: bool,
) {
    let mut used_account_list = get_used_account_list(credentials_home_dir);

    let used_account = if let Some(mut used_account) = used_account_list
        .iter()
        .position(|used_account| &used_account.account_id == account_id)
        .and_then(|position| used_account_list.remove(position))
    {
        used_account.used_as_signer |= account_is_signer;
        used_account
    } else {
        UsedAccount {
            account_id: account_id.clone(),
            used_as_signer: account_is_signer,
        }
    };
    used_account_list.push_front(used_account);

    let used_account_list_path = get_used_account_list_path(credentials_home_dir);
    if let Ok(used_account_list_buf) = serde_json::to_string(&used_account_list) {
        let _ = std::fs::write(used_account_list_path, used_account_list_buf);
    }
}

pub fn get_used_account_list(credentials_home_dir: &std::path::Path) -> VecDeque<UsedAccount> {
    let used_account_list_path = get_used_account_list_path(credentials_home_dir);
    serde_json::from_str(
        std::fs::read_to_string(used_account_list_path)
            .as_deref()
            .unwrap_or("[]"),
    )
    .unwrap_or_default()
}

pub fn is_used_account_list_exist(credentials_home_dir: &std::path::Path) -> bool {
    get_used_account_list_path(credentials_home_dir).exists()
}

pub fn input_signer_account_id_from_used_account_list(
    credentials_home_dir: &std::path::Path,
    message: &str,
) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
    let account_is_signer = true;
    input_account_id_from_used_account_list(credentials_home_dir, message, account_is_signer)
}

pub fn input_non_signer_account_id_from_used_account_list(
    credentials_home_dir: &std::path::Path,
    message: &str,
) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
    let account_is_signer = false;
    input_account_id_from_used_account_list(credentials_home_dir, message, account_is_signer)
}

fn input_account_id_from_used_account_list(
    credentials_home_dir: &std::path::Path,
    message: &str,
    account_is_signer: bool,
) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
    let used_account_list = get_used_account_list(credentials_home_dir)
        .into_iter()
        .filter(|account| !account_is_signer || account.used_as_signer)
        .map(|account| account.account_id.to_string())
        .collect::<Vec<_>>();
    let account_id_str = match Text::new(message)
        .with_autocomplete(move |val: &str| {
            Ok(used_account_list
                .iter()
                .filter(|s| s.contains(val))
                .cloned()
                .collect())
        })
        .with_validator(|account_id_str: &str| {
            match near_kit::AccountId::validate(account_id_str) {
                Ok(_) => Ok(inquire::validator::Validation::Valid),
                Err(err) => Ok(inquire::validator::Validation::Invalid(
                    inquire::validator::ErrorMessage::Custom(format!("Invalid account ID: {err}")),
                )),
            }
        })
        .prompt()
    {
        Ok(value) => value,
        Err(
            inquire::error::InquireError::OperationCanceled
            | inquire::error::InquireError::OperationInterrupted,
        ) => return Ok(None),
        Err(err) => return Err(err.into()),
    };
    let account_id = crate::types::account_id::AccountId::from_str(&account_id_str)?;
    update_used_account_list(credentials_home_dir, account_id.as_ref(), account_is_signer);
    Ok(Some(account_id))
}

fn get_used_ft_contract_account_list_path(
    credentials_home_dir: &std::path::Path,
) -> std::path::PathBuf {
    credentials_home_dir.join("ft_contracts.json")
}

pub fn is_used_ft_contract_account_list_exist(credentials_home_dir: &std::path::Path) -> bool {
    get_used_ft_contract_account_list_path(credentials_home_dir).exists()
}

fn get_top_ft_tokens_from_nearblocks()
-> color_eyre::eyre::Result<VecDeque<crate::types::ft_properties::FtContract>, String> {
    #[derive(serde::Deserialize)]
    struct ApiResponse {
        tokens: VecDeque<crate::types::ft_properties::FtContract>,
    }

    let url = url::Url::parse("https://api.nearblocks.io/v1/fts").map_err(|err| err.to_string())?;
    let mut last_error_message = String::new();

    for _ in 0..10 {
        match reqwest::blocking::get(url.clone()) {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<ApiResponse>() {
                        Ok(data) => return Ok(data.tokens),
                        Err(err) => {
                            return Err(format!(
                                "Failed to parse JSON response from nearblocks.io API: {err}"
                            ));
                        }
                    }
                } else {
                    last_error_message = format!(
                        "HTTP error from nearblocks.io API: {} - {}",
                        response.status(),
                        response
                            .text()
                            .unwrap_or_else(|_| "Unable to read response body".to_string())
                    );
                }
            }
            Err(err) => {
                last_error_message =
                    format!("Failed to get response from nearblocks.io API: {err}");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    Err(last_error_message)
}

pub fn create_used_ft_contract_account_list(credentials_home_dir: &std::path::Path) -> CliResult {
    const FT_CONTRACT_ACCOUNTS_LIST_SIZE_CAP: usize = 50;

    let ft_contract_account_list_path =
        get_used_ft_contract_account_list_path(credentials_home_dir);

    std::fs::create_dir_all(credentials_home_dir)?;

    if let Ok(ft_contract_accounts_list) = get_top_ft_tokens_from_nearblocks() {
        let ft_contract_account_list_buf = serde_json::to_string(
            &ft_contract_accounts_list
                .into_iter()
                .take(FT_CONTRACT_ACCOUNTS_LIST_SIZE_CAP)
                .collect::<VecDeque<_>>(),
        )?;
        std::fs::write(&ft_contract_account_list_path, ft_contract_account_list_buf)
            .wrap_err_with(|| {
                format!(
                    "Failed to write to file: {}",
                    ft_contract_account_list_path.display()
                )
            })?;
    }

    Ok(())
}

pub fn get_used_ft_contract_account_list(
    credentials_home_dir: &std::path::Path,
) -> VecDeque<crate::types::ft_properties::FtContract> {
    let ft_contract_account_list_path =
        get_used_ft_contract_account_list_path(credentials_home_dir);
    serde_json::from_str(
        std::fs::read_to_string(ft_contract_account_list_path)
            .as_deref()
            .unwrap_or("[]"),
    )
    .unwrap_or_default()
}

pub fn update_used_ft_contract_account_list(
    credentials_home_dir: &std::path::Path,
    new_ft_contract_account: &crate::types::ft_properties::FtContract,
) {
    let mut ft_contract_accounts_list = get_used_ft_contract_account_list(credentials_home_dir);

    let used_ft_contract_account = if let Some(used_ft_contract_account) = ft_contract_accounts_list
        .iter()
        .position(|ft_contract_account| {
            ft_contract_account.ft_contract_account_id
                == new_ft_contract_account.ft_contract_account_id
        })
        .and_then(|position| ft_contract_accounts_list.remove(position))
    {
        used_ft_contract_account
    } else {
        new_ft_contract_account.clone()
    };

    ft_contract_accounts_list.push_front(used_ft_contract_account);

    let ft_contract_account_list_path =
        get_used_ft_contract_account_list_path(credentials_home_dir);
    if let Ok(ft_contract_account_list_buf) = serde_json::to_string(&ft_contract_accounts_list) {
        let _ = std::fs::write(ft_contract_account_list_path, ft_contract_account_list_buf);
    }
}

pub fn save_cli_command(cli_cmd_str: &str) {
    let tmp_file_path = std::env::temp_dir().join(FINAL_COMMAND_FILE_NAME);

    let Ok(mut tmp_file) = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(tmp_file_path)
    else {
        eprintln!("Failed to open a temporary file to store a cli command");
        return;
    };

    if let Err(err) = writeln!(tmp_file, "{cli_cmd_str}") {
        eprintln!("Failed to store a cli command in a temporary file: {err}");
    }
}
