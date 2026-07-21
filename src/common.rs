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
use serde_with::{base64::Base64, serde_as};
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
    smart_default::SmartDefault,
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

#[derive(Debug, Clone)]
pub struct BlockHashAsBase58 {
    pub inner: near_kit::CryptoHash,
}

impl std::str::FromStr for BlockHashAsBase58 {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: s.parse().map_err(|err| format!("{err}"))?,
        })
    }
}

impl std::fmt::Display for BlockHashAsBase58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockHash {}", self.inner)
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
    RpcError(near_kit::RpcError),
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
    let storage_amount_per_byte = get_partial_protocol_config(network_config, &block_reference)
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
) -> color_eyre::eyre::Result<near_kit::AccessKeyView, AccountStateError> {
    tracing::info!(target: "near_teach_me", "Account access key verification ...");
    loop {
        match block_on(network_config.client().rpc().view_access_key(
            &account_id,
            &public_key,
            near_kit::BlockReference::optimistic(),
        )) {
            Ok(access_key_view) => {
                return Ok(access_key_view);
            }
            Err(err @ near_kit::RpcError::AccessKeyNotFound { .. }) => {
                return Err(AccountStateError::RpcError(err));
            }
            Err(err) => {
                let category = if matches!(
                    &err,
                    near_kit::RpcError::Http(_)
                        | near_kit::RpcError::Network { .. }
                        | near_kit::RpcError::Timeout(_)
                ) {
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
                    return Err(AccountStateError::RpcError(err));
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

        let result = block_on(get_account_state(
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

pub fn is_receiver_on_wrong_network(
    linkdrop_account_id: Option<&near_kit::AccountId>,
    receiver_account_id: &near_kit::AccountId,
) -> bool {
    // Don't check for implicit accounts on the network,
    // as they can be created on any network and we cannot be sure about the network based on the account ID.
    if receiver_account_id.get_account_type().is_implicit() {
        return false;
    }

    let Some(linkdrop) = linkdrop_account_id else {
        return false;
    };
    let receiver_str = receiver_account_id.as_str();
    match linkdrop.as_str() {
        "near" => receiver_str.ends_with(".testnet"),
        "testnet" => !receiver_str.ends_with(".testnet"),
        _ => false,
    }
}

/// Returns `Ok(true)` if the transaction should proceed, `Ok(false)` if the user cancelled.
#[tracing::instrument(name = "Validating the recipient account", skip_all)]
pub fn validate_receiver_account_id(
    network_config: &crate::config::NetworkConfig,
    receiver_account_id: &near_kit::AccountId,
    verbosity: crate::Verbosity,
    offline_mode: bool,
) -> color_eyre::eyre::Result<bool> {
    tracing::Span::current().pb_set_message(&format!(
        "<{receiver_account_id}> on network <{}> ...",
        network_config.network_name
    ));
    tracing::info!(target: "near_teach_me", "Validating the recipient account <{receiver_account_id}> on network <{}> ...", network_config.network_name);

    if let crate::Verbosity::Quiet = verbosity {
        return Ok(true);
    }

    if is_receiver_on_wrong_network(
        network_config.linkdrop_account_id.as_ref(),
        receiver_account_id,
    ) {
        return handle_validation_warning(format!(
            "<{}> looks like it belongs to a different network than <{}>.",
            receiver_account_id, network_config.network_name
        ));
    }

    if offline_mode {
        return handle_validation_warning(format!(
            "Skipping account validation for <{}> on <{}> in offline mode.",
            receiver_account_id, network_config.network_name
        ));
    }

    match block_on(get_account_state(
        network_config,
        receiver_account_id,
        near_kit::BlockReference::optimistic(),
    )) {
        Ok(_) => Ok(true),
        Err(ViewAccountError::UnknownAccount { .. }) => handle_validation_warning(format!(
            "<{}> does not exist on <{}>.",
            receiver_account_id, network_config.network_name
        )),
        Err(err) => Err(color_eyre::eyre::eyre!("{err}")),
    }
}

fn handle_validation_warning(message: String) -> color_eyre::eyre::Result<bool> {
    tracing::warn!("{}", message.red());
    suspend_tracing_indicatif::<_, color_eyre::eyre::Result<bool>>(ask_if_should_proceed)
}

fn ask_if_should_proceed() -> color_eyre::eyre::Result<bool> {
    #[derive(strum_macros::Display, PartialEq)]
    enum ConfirmOptions {
        #[strum(to_string = "Yes, I want to proceed with this receiver account ID.")]
        Yes,
        #[strum(to_string = "No, I want to cancel the transaction.")]
        No,
    }
    match Select::new(
        "Do you want to proceed?",
        vec![ConfirmOptions::Yes, ConfirmOptions::No],
    )
    .prompt()
    {
        Ok(value) => Ok(value == ConfirmOptions::Yes),
        Err(
            inquire::error::InquireError::OperationCanceled
            | inquire::error::InquireError::OperationInterrupted,
        ) => Ok(false),
        Err(err) => Err(err.into()),
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
    UnknownAccount { account_id: near_kit::AccountId },
    /// A transport / connectivity error occurred.
    TransportError(String),
    /// Any other server-side error.
    ServerError(String),
}

impl std::fmt::Display for ViewAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownAccount { account_id } => {
                write!(f, "account {account_id} does not exist while viewing")
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
    let derived_private_key = near_slip10::derive_key_from_path(
        &master_seed,
        near_slip10::Curve::Ed25519,
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
    seed_phrase_hd_path: near_slip10::BIP32Path,
    master_seed_phrase: &str,
) -> color_eyre::eyre::Result<near_kit::PublicKey> {
    let master_seed = bip39::Mnemonic::parse(master_seed_phrase)?.to_seed("");
    let derived_private_key = near_slip10::derive_key_from_path(
        &master_seed,
        near_slip10::Curve::Ed25519,
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

    let derived_private_key = near_slip10::derive_key_from_path(
        &master_seed,
        near_slip10::Curve::Ed25519,
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

/// Signature scheme to use when autogenerating a new key pair: the classic
/// Ed25519, or the post-quantum ML-DSA-65 (FIPS 204) scheme that protocol 2.13
/// adds as a third access-key type.
#[derive(Debug, Clone, strum::EnumDiscriminants, clap::ValueEnum)]
#[strum_discriminants(derive(strum::EnumMessage, strum::EnumIter))]
pub enum SignatureScheme {
    /// Ed25519 (classic, default NEAR key type)
    #[value(name = "ed25519")]
    Ed25519,
    /// ML-DSA-65 post-quantum signature scheme (FIPS 204)
    #[value(name = "ml-dsa-65")]
    MlDsa65,
}

impl interactive_clap::ToCli for SignatureScheme {
    type CliVariant = SignatureScheme;
}

impl std::fmt::Display for SignatureScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "ed25519"),
            Self::MlDsa65 => write!(f, "ml-dsa-65"),
        }
    }
}

impl std::str::FromStr for SignatureScheme {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ed25519" => Ok(Self::Ed25519),
            "ml-dsa-65" => Ok(Self::MlDsa65),
            _ => Err(format!("SignatureScheme: invalid value `{s}`")),
        }
    }
}

impl std::fmt::Display for SignatureSchemeDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ed25519 => write!(f, "ed25519    - Ed25519 (classic, default NEAR key type)"),
            Self::MlDsa65 => write!(
                f,
                "ml-dsa-65  - ML-DSA-65 post-quantum signature scheme (FIPS 204)"
            ),
        }
    }
}

/// Shared resolver for the `--signature-scheme` argument of every command that
/// autogenerates a key pair. When the flag is omitted we default to Ed25519
/// (so existing non-interactive invocations keep their behaviour) but, in an
/// interactive terminal, prompt the user to pick instead.
pub fn input_signature_scheme() -> color_eyre::eyre::Result<Option<SignatureScheme>> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return Ok(Some(SignatureScheme::Ed25519));
    }
    let variants = SignatureSchemeDiscriminants::iter().collect::<Vec<_>>();
    let selected = Select::new(
        "Which signature scheme should the new key pair use?",
        variants,
    )
    .prompt()?;
    Ok(Some(match selected {
        SignatureSchemeDiscriminants::Ed25519 => SignatureScheme::Ed25519,
        SignatureSchemeDiscriminants::MlDsa65 => SignatureScheme::MlDsa65,
    }))
}

/// A freshly generated key pair for either supported signature scheme. The
/// Ed25519 variant keeps the full [`KeyPairProperties`] (seed phrase, HD path,
/// implicit account id); ML-DSA-65 keys are random and carry only the key
/// strings (there is no seed phrase or implicit-account form for them yet).
#[derive(Debug, Clone)]
pub enum GeneratedKeyPair {
    Ed25519(KeyPairProperties),
    MlDsa65 {
        public_key: String,
        private_key: String,
    },
}

impl GeneratedKeyPair {
    pub fn generate(signature_scheme: &SignatureScheme) -> color_eyre::eyre::Result<Self> {
        match signature_scheme {
            SignatureScheme::Ed25519 => Ok(Self::Ed25519(generate_keypair()?)),
            SignatureScheme::MlDsa65 => {
                let private_key = near_kit::SecretKey::generate_ml_dsa65();
                Ok(Self::MlDsa65 {
                    public_key: private_key.public_key().to_string(),
                    private_key: private_key.to_string(),
                })
            }
        }
    }

    pub fn public_key_str(&self) -> &str {
        match self {
            Self::Ed25519(properties) => &properties.public_key_str,
            Self::MlDsa65 { public_key, .. } => public_key,
        }
    }

    pub fn public_key(&self) -> color_eyre::eyre::Result<near_kit::PublicKey> {
        Ok(near_kit::PublicKey::from_str(self.public_key_str())?)
    }

    /// Identifier under which this key's credentials are stored: the keychain
    /// entry name and the legacy-keychain file name. It must equal the
    /// public-key string the RPC access-key list returns, because that is what
    /// `sign_with_keychain` / the legacy-keychain signer look the saved key up
    /// by — otherwise a key saved here can never be found again.
    ///
    /// For ed25519 this is the full public-key string, unchanged. For ML-DSA-65
    /// the on-chain identifier is the short `ml-dsa-65-hash:...` SHA3-256 handle
    /// (`near_kit::PublicKey::to_ml_dsa65_hash`), not the ~1952-byte full key: the full
    /// key would blow past filesystem name limits and would never match the
    /// handle the chain reports for the key.
    pub fn keychain_key_id(&self) -> color_eyre::eyre::Result<String> {
        Ok(match self {
            Self::Ed25519(properties) => properties.public_key_str.clone(),
            Self::MlDsa65 { .. } => self
                .public_key()?
                .to_ml_dsa65_hash()
                .expect("ML-DSA-65 keys always have a hash handle")
                .to_string(),
        })
    }

    /// JSON written to the keychain / legacy keychain credentials file. Ed25519
    /// preserves the historical full `KeyPairProperties` layout; ML-DSA-65 uses
    /// the minimal `{ public_key, private_key }` credentials format (which the
    /// keychain reader already understands).
    pub fn keychain_json(&self) -> color_eyre::eyre::Result<String> {
        Ok(match self {
            Self::Ed25519(properties) => serde_json::to_string(properties)?,
            Self::MlDsa65 {
                public_key,
                private_key,
            } => serde_json::to_string(&serde_json::json!({
                "public_key": public_key,
                "private_key": private_key,
            }))?,
        })
    }

    /// Human-readable "access key info" block printed to the terminal.
    pub fn terminal_info(&self) -> String {
        match self {
            Self::Ed25519(properties) => format!(
                "\n--------------------  Access key info ------------------\
                 \nMaster Seed Phrase: {}\nSeed Phrase HD Path: {}\nImplicit Account ID: {}\nPublic Key: {}\nSECRET KEYPAIR: {}\
                 \n--------------------------------------------------------",
                properties.master_seed_phrase,
                properties.seed_phrase_hd_path,
                properties.implicit_account_id,
                properties.public_key_str,
                properties.secret_keypair_str,
            ),
            Self::MlDsa65 {
                public_key,
                private_key,
            } => format!(
                "\n--------------------  Access key info ------------------\
                 \nSignature scheme: ML-DSA-65 (post-quantum, FIPS 204)\nPublic Key: {public_key}\nSECRET KEYPAIR: {private_key}\
                 \n--------------------------------------------------------",
            ),
        }
    }
}

pub fn print_full_signed_transaction(transaction: &near_kit::SignedTransactionV1) -> String {
    let mut info_str = format!("\n{:<13} {}", "signature:", transaction.signature);
    info_str.push_str(&format!(
        "\nunsigned transaction hash (Base58-encoded SHA-256 hash): {}",
        transaction.transaction.get_hash()
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "public_key:",
        transaction.transaction.public_key()
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "nonce:",
        transaction.transaction.nonce().nonce()
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "block_hash:",
        transaction.transaction.block_hash()
    ));
    if let Some(nonce_index) = transaction.transaction.nonce().nonce_index() {
        info_str.push_str(&format!("\n{:<13} {}", "nonce_index:", nonce_index));
        info_str.push_str(&format!(
            "\n{:<13} {:?}",
            "nonce_mode:",
            transaction.transaction.nonce_mode()
        ));
    }
    let prepopulated = crate::commands::PrepopulatedTransaction {
        signer_id: transaction.transaction.signer_id().clone(),
        receiver_id: transaction.transaction.receiver_id().clone(),
        actions: transaction.transaction.actions().to_vec(),
    };
    info_str.push_str(&print_unsigned_transaction(&prepopulated));
    info_str
}

pub fn print_full_unsigned_transaction(transaction: &near_kit::VersionedTransaction) -> String {
    let mut info_str = format!(
        "\nunsigned transaction hash (Base58-encoded SHA-256 hash): {}",
        transaction.get_hash()
    );

    info_str.push_str(&format!(
        "\n{:<13} {}",
        "public_key:",
        transaction.public_key()
    ));
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "nonce:",
        transaction.nonce().nonce()
    ));
    if let Some(nonce_index) = transaction.nonce().nonce_index() {
        info_str.push_str(&format!("\n{:<13} {}", "nonce_index:", nonce_index));
        info_str.push_str(&format!(
            "\n{:<13} {:?}",
            "nonce_mode:",
            transaction.nonce_mode()
        ));
    }
    info_str.push_str(&format!(
        "\n{:<13} {}",
        "block_hash:",
        transaction.block_hash()
    ));

    let prepopulated = crate::commands::PrepopulatedTransaction {
        signer_id: transaction.signer_id().clone(),
        receiver_id: transaction.receiver_id().clone(),
        actions: transaction.actions().to_vec(),
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
                    "--", "create account:", transaction.receiver_id
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
                    "", "method name:", function_call_action.method_name
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
                    "", "public key:", stake_action.public_key
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
                    "", "public key:", add_key_action.public_key
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "nonce:", add_key_action.access_key.nonce
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {:?}",
                    "", "permission:", add_key_action.access_key.permission
                ));
            }
            near_kit::Action::DeleteKey(delete_key_action) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "delete access key:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", delete_key_action.public_key
                ));
            }
            near_kit::Action::DeleteAccount(delete_account_action) => {
                info_str.push_str(&format!(
                    "\n{:>5} {:<20} {}",
                    "--", "delete account:", transaction.receiver_id
                ));
                info_str.push_str(&format!(
                    "\n{:>8} {:<17} {}",
                    "", "beneficiary id:", delete_account_action.beneficiary_id
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
                            let bytes = borsh::to_vec(nda)
                                .expect("NonDelegateAction borsh serialization should not fail");
                            borsh::from_slice::<near_kit::Action>(&bytes)
                                .expect("Action borsh deserialization should not fail")
                        })
                        .collect(),
                };
                info_str.push_str(&print_unsigned_transaction(&prepopulated_transaction));
            }
            near_kit::Action::DelegateV2(versioned_signed_delegate_action) => {
                let actions = versioned_signed_delegate_action
                    .delegate_action
                    .get_actions();
                let (signer_id, receiver_id) =
                    match &versioned_signed_delegate_action.delegate_action {
                        near_kit::VersionedDelegateActionPayload::V2(delegate_action) => (
                            delegate_action.sender_id.clone(),
                            delegate_action.receiver_id.clone(),
                        ),
                    };
                let prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                    signer_id,
                    receiver_id,
                    actions,
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
                    near_kit::GlobalContractId::CodeHash(hash) => {
                        format!(
                            "use global <{}> code to deploy from",
                            near_kit::CryptoHash::from_bytes(*hash)
                        )
                    }
                    near_kit::GlobalContractId::AccountId(account_id) => {
                        format!("use global <{account_id}> code to deploy from")
                    }
                };
                info_str.push_str(&format!("{:>5} {:<70}", "--", identifier));
            }
            near_kit::Action::DeterministicStateInit(deterministic_init_action) => {
                let deterministic_account_id =
                    deterministic_init_action.state_init.derive_account_id();
                info_str.push_str(&format!(
                    "\n{:>5} {:<20}",
                    "--",
                    format!("create deterministic account <{deterministic_account_id}>:")
                ));
                info_str.push_str(&format!(
                    "\n{:>18} {:<12}: {}",
                    "", "deposit", deterministic_init_action.deposit
                ));
                let state_init = match &deterministic_init_action.state_init {
                    near_kit::StateInit::V1(v1) => {
                        let mut ret = "V1".to_string();
                        ret.push_str(&format!("\n{:>31} {:<13} {:?}", "", "data", v1.data));
                        ret.push_str(&format!(
                            "\n{:>31} {:<13} {}",
                            "",
                            "code",
                            match &v1.code {
                                near_kit::GlobalContractId::CodeHash(hash) => {
                                    format!(
                                        "use global <{}> code to deploy from",
                                        near_kit::CryptoHash::from_bytes(*hash)
                                    )
                                }
                                near_kit::GlobalContractId::AccountId(account_id) => {
                                    format!("use global <{account_id}> code to deploy from")
                                }
                            }
                        ));
                        ret
                    }
                };

                info_str.push_str(&format!("\n{:>18} {:<13} {}", "", "state:", state_init));
            }
            near_kit::Action::TransferToGasKey(transfer_to_gas_key) => {
                info_str.push_str(&format!("\n{:>5} {:<20}", "--", "transfer to gas key:"));
                info_str.push_str(&format!(
                    "\n{:>18} {:<13} {}",
                    "", "public key:", transfer_to_gas_key.public_key
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
                    "", "public key:", withdraw_from_gas_key.public_key
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

fn print_value_successful_transaction(transaction_info: near_kit::FinalExecutionOutcome) -> String {
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
            near_kit::ActionView::DelegateV2 {
                delegate_action,
                signature: _,
            } => {
                let sender_id = match delegate_action {
                    near_kit::VersionedDelegateActionPayloadView::V2(delegate_action) => {
                        delegate_action.sender_id
                    }
                };
                info_str.push_str(&format!(
                    "Actions delegated for <{sender_id}> completed successfully.",
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

pub fn rpc_transaction_error(err: &near_kit::RpcError) -> color_eyre::Result<String> {
    match err {
        near_kit::RpcError::Http(_) => Ok("Transport error transaction".to_string()),
        near_kit::RpcError::Network {
            status_code: Some(401),
            ..
        } => Err(color_eyre::eyre::eyre!(
            "JSON RPC server requires authentication. Please, authenticate near CLI with the JSON RPC server you use."
        )),
        near_kit::RpcError::Network {
            status_code: Some(429),
            ..
        } => Ok("JSON RPC server is currently busy".to_string()),
        near_kit::RpcError::Network {
            status_code: Some(400),
            ..
        } => Err(color_eyre::eyre::eyre!(
            "JSON RPC server responded with a bad request. Please, check your request parameters."
        )),
        near_kit::RpcError::Network {
            status_code: Some(503),
            ..
        } => Ok("JSON RPC server is currently unavailable".to_string()),
        near_kit::RpcError::Network {
            status_code: Some(status),
            ..
        } => Err(color_eyre::eyre::eyre!(
            "JSON RPC server responded with an unexpected status code: {status}"
        )),
        near_kit::RpcError::Network { .. } => Ok("Transport error transaction".to_string()),
        near_kit::RpcError::Timeout(_) | near_kit::RpcError::RequestTimeout { .. } => {
            Ok("Timeout error transaction".to_string())
        }
        near_kit::RpcError::InvalidTx(invalid_tx_error) => {
            convert_invalid_tx_error_to_cli_result(invalid_tx_error)?;
            Ok(String::new())
        }
        near_kit::RpcError::ShardUnavailable(_)
        | near_kit::RpcError::NodeNotSynced(_)
        | near_kit::RpcError::InternalError(_) => Ok(err.to_string()),
        near_kit::RpcError::Rpc { code, .. } if *code == -32000 || *code == -32603 => {
            Ok(err.to_string())
        }
        _ => Err(color_eyre::eyre::eyre!("RPC Server Error: {err}")),
    }
}

#[cfg(test)]
mod rpc_transaction_error_tests {
    use super::rpc_transaction_error;

    #[test]
    fn invalid_nonce_is_not_classified_as_retryable() {
        let err = near_kit::RpcError::InvalidTx(near_kit::InvalidTxError::InvalidNonce {
            ak_nonce: 10,
            tx_nonce: 9,
        });

        assert!(rpc_transaction_error(&err).is_err());
    }
}

pub fn convert_action_error_to_cli_result(
    action_error: &near_kit::ActionError,
) -> crate::CliResult {
    use near_kit::ActionErrorKind;

    let message = match &action_error.kind {
        ActionErrorKind::AccountAlreadyExists { account_id } => format!(
            "Error: Create Account action tries to create an account with account ID <{account_id}> which already exists in the storage."
        ),
        ActionErrorKind::AccountDoesNotExist { account_id } => format!(
            "Error: TX receiver ID <{account_id}> doesn't exist (but action is not \"Create Account\")."
        ),
        ActionErrorKind::CreateAccountOnlyByRegistrar { .. } => {
            "Error: A top-level account ID can only be created by registrar.".to_string()
        }
        ActionErrorKind::CreateAccountNotAllowed {
            account_id,
            predecessor_id,
        } => format!(
            "Error: A newly created account <{account_id}> must be under a namespace of the creator account <{predecessor_id}>."
        ),
        ActionErrorKind::ActorNoPermission { .. } => {
            "Error: Administrative actions can be proceed only if sender=receiver or the first TX action is a \"Create Account\" action.".to_string()
        }
        ActionErrorKind::DeleteKeyDoesNotExist {
            account_id,
            public_key,
        } => format!(
            "Error: Account <{account_id}>  tries to remove an access key <{public_key}> that doesn't exist."
        ),
        ActionErrorKind::AddKeyAlreadyExists {
            account_id,
            public_key,
        } => format!(
            "Error: Public key <{public_key}> is already used for an existing account ID <{account_id}>."
        ),
        ActionErrorKind::DeleteAccountStaking { account_id } => {
            format!("Error: Account <{account_id}> is staking and can not be deleted")
        }
        ActionErrorKind::LackBalanceForState { account_id, amount } => format!(
            "Error: Receipt action can't be completed, because the remaining balance will not be enough to cover storage.\nAn account which needs balance: <{account_id}>\nBalance required to complete the action: <{}>",
            amount.exact_amount_display()
        ),
        ActionErrorKind::TriesToUnstake { account_id } => format!(
            "Error: Account <{account_id}> is not yet staked, but tries to unstake."
        ),
        ActionErrorKind::TriesToStake {
            account_id,
            stake,
            balance,
            ..
        } => format!(
            "Error: Account <{account_id}> doesn't have enough balance ({}) to increase the stake ({}).",
            balance.exact_amount_display(),
            stake.exact_amount_display()
        ),
        ActionErrorKind::InsufficientStake {
            stake,
            minimum_stake,
            ..
        } => format!(
            "Error: Insufficient stake {}.\nThe minimum rate must be {}.",
            stake.exact_amount_display(),
            minimum_stake.exact_amount_display()
        ),
        ActionErrorKind::FunctionCallError(error) => format!(
            "Error: An error occurred during a `FunctionCall` action.\n{error:?}"
        ),
        ActionErrorKind::NewReceiptValidationError(error) => format!(
            "Error: Error occurs when a new `ActionReceipt` created by the `FunctionCall` action fails.\n{error:?}"
        ),
        ActionErrorKind::OnlyImplicitAccountCreationAllowed { .. } => {
            "Error: `CreateAccount` action is called on hex-characters account of length 64.\nSee implicit account creation NEP: https://github.com/nearprotocol/NEPs/pull/71".to_string()
        }
        ActionErrorKind::DeleteAccountWithLargeState { account_id } => format!(
            "Error: Delete account <{account_id}> whose state is large is temporarily banned."
        ),
        ActionErrorKind::DelegateActionInvalidSignature => {
            "Error: Invalid Signature on DelegateAction".to_string()
        }
        ActionErrorKind::DelegateActionSenderDoesNotMatchTxReceiver {
            sender_id,
            receiver_id,
        } => format!(
            "Error: Delegate Action sender {sender_id} does not match transaction receiver {receiver_id}"
        ),
        ActionErrorKind::DelegateActionExpired => "Error: DelegateAction Expired".to_string(),
        ActionErrorKind::DelegateActionAccessKeyError(_) => {
            "Error: The given public key doesn't exist for the sender".to_string()
        }
        ActionErrorKind::DelegateActionInvalidNonce {
            delegate_nonce,
            ak_nonce,
        } => format!(
            "Error: DelegateAction Invalid Delegate Nonce: {delegate_nonce} ak_nonce: {ak_nonce}"
        ),
        ActionErrorKind::DelegateActionNonceTooLarge {
            delegate_nonce,
            upper_bound,
        } => format!(
            "Error: DelegateAction Invalid Delegate Nonce: {delegate_nonce} upper bound: {upper_bound}"
        ),
        ActionErrorKind::GlobalContractDoesNotExist { identifier } => {
            let identifier = match identifier {
                near_kit::GlobalContractIdentifierView::CodeHash(hash) => {
                    format!("hash<{hash}>")
                }
                near_kit::GlobalContractIdentifierView::AccountId(account_id) => {
                    format!("account id<{account_id}>")
                }
            };
            format!("Error: Global contract with identifier {identifier} does not exist.")
        }
        ActionErrorKind::GasKeyDoesNotExist {
            account_id,
            public_key,
        } => format!("Error: Gas key <{public_key}> does not exist for account <{account_id}>."),
        ActionErrorKind::InsufficientGasKeyBalance {
            account_id,
            public_key,
            balance,
            ..
        } => format!(
            "Error: Gas key <{public_key}> for account <{account_id}> has insufficient balance ({}).",
            balance.exact_amount_display()
        ),
        ActionErrorKind::GasKeyBalanceTooHigh {
            account_id,
            public_key,
            balance,
        } => {
            let key_info = match public_key {
                Some(public_key) => format!("gas key <{public_key}>") ,
                None => "gas keys".to_string(),
            };
            format!(
                "Error: Balance ({}) of {key_info} for account <{account_id}> is too high to perform this action.",
                balance.exact_amount_display()
            )
        }
        ActionErrorKind::Unknown(error) => format!("Error: {error}"),
    };

    Err(color_eyre::Report::msg(message))
}

pub fn convert_invalid_tx_error_to_cli_result(
    invalid_tx_error: &near_kit::InvalidTxError,
) -> crate::CliResult {
    use near_kit::{ActionsValidationError, InvalidAccessKeyError, InvalidTxError, StorageError};

    let message = match invalid_tx_error {
        InvalidTxError::InvalidAccessKeyError(error) => match error {
            InvalidAccessKeyError::AccessKeyNotFound {
                account_id,
                public_key,
            } => format!(
                "Error: Public key {public_key} doesn't exist for the account <{account_id}>."
            ),
            InvalidAccessKeyError::ReceiverMismatch {
                tx_receiver,
                ak_receiver,
            } => format!(
                "Error: Transaction for <{tx_receiver}> doesn't match the access key for <{ak_receiver}>."
            ),
            InvalidAccessKeyError::MethodNameMismatch { method_name } => format!(
                "Error: Transaction method name <{method_name}> isn't allowed by the access key."
            ),
            InvalidAccessKeyError::RequiresFullAccess => {
                "Error: Transaction requires a full permission access key.".to_string()
            }
            InvalidAccessKeyError::NotEnoughAllowance {
                account_id,
                public_key,
                allowance,
                cost,
            } => format!(
                "Error: Access Key <{public_key}> for account <{account_id}> does not have enough allowance ({}) to cover transaction cost ({}).",
                allowance.exact_amount_display(),
                cost.exact_amount_display()
            ),
            InvalidAccessKeyError::DepositWithFunctionCall => {
                "Error: Having a deposit with a function call action is not allowed with a function call access key.".to_string()
            }
            InvalidAccessKeyError::Unknown(error) => format!("Error: {error}"),
        },
        InvalidTxError::InvalidSignerId { signer_id } => format!(
            "Error: TX signer ID <{signer_id}> is not in a valid format or does not satisfy requirements\nSee \"near_runtime_utils::utils::is_valid_account_id\"."
        ),
        InvalidTxError::SignerDoesNotExist { signer_id } => {
            format!("Error: TX signer ID <{signer_id}> is not found in the storage.")
        }
        InvalidTxError::InvalidNonce { tx_nonce, ak_nonce } => format!(
            "Error: Transaction nonce ({tx_nonce}) must be account[access_key].nonce ({ak_nonce}) + 1."
        ),
        InvalidTxError::NonceTooLarge {
            tx_nonce,
            upper_bound,
        } => format!(
            "Error: Transaction nonce ({tx_nonce}) is larger than the upper bound ({upper_bound}) given by the block height."
        ),
        InvalidTxError::InvalidReceiverId { receiver_id } => format!(
            "Error: TX receiver ID ({receiver_id}) is not in a valid format or does not satisfy requirements\nSee \"near_runtime_utils::is_valid_account_id\"."
        ),
        InvalidTxError::InvalidSignature => "Error: TX signature is not valid".to_string(),
        InvalidTxError::NotEnoughBalance {
            signer_id,
            balance,
            cost,
        } => format!(
            "Error: Account <{signer_id}> does not have enough balance ({}) to cover TX cost ({}).",
            balance.exact_amount_display(),
            cost.exact_amount_display()
        ),
        InvalidTxError::LackBalanceForState { signer_id, amount } => format!(
            "Error: Signer account <{signer_id}> doesn't have enough balance ({}) after transaction.",
            amount.exact_amount_display()
        ),
        InvalidTxError::CostOverflow => {
            "Error: An integer overflow occurred during transaction cost estimation.".to_string()
        }
        InvalidTxError::InvalidChain => {
            "Error: Transaction parent block hash doesn't belong to the current chain.".to_string()
        }
        InvalidTxError::Expired => "Error: Transaction has expired.".to_string(),
        InvalidTxError::ActionsValidation(error) => match error {
            ActionsValidationError::DeleteActionMustBeFinal => {
                "Error: The delete action must be the final action in transaction.".to_string()
            }
            ActionsValidationError::TotalPrepaidGasExceeded {
                total_prepaid_gas,
                limit,
            } => format!(
                "Error: The total prepaid gas ({total_prepaid_gas}) for all given actions exceeded the limit ({limit})."
            ),
            ActionsValidationError::TotalNumberOfActionsExceeded {
                total_number_of_actions,
                limit,
            } => format!(
                "Error: The number of actions ({total_number_of_actions}) exceeded the given limit ({limit})."
            ),
            ActionsValidationError::AddKeyMethodNamesNumberOfBytesExceeded {
                total_number_of_bytes,
                limit,
            } => format!(
                "Error: The total number of bytes ({total_number_of_bytes}) of the method names exceeded the limit ({limit}) in a Add Key action."
            ),
            ActionsValidationError::AddKeyMethodNameLengthExceeded { length, limit } => format!(
                "Error: The length ({length}) of some method name exceeded the limit ({limit}) in a Add Key action."
            ),
            ActionsValidationError::IntegerOverflow => "Error: Integer overflow.".to_string(),
            ActionsValidationError::InvalidAccountId { account_id } => {
                format!("Error: Invalid account ID <{account_id}>.")
            }
            ActionsValidationError::ContractSizeExceeded { size, limit } => format!(
                "Error: The size ({size}) of the contract code exceeded the limit ({limit}) in a DeployContract action."
            ),
            ActionsValidationError::FunctionCallMethodNameLengthExceeded { length, limit } => format!(
                "Error: The length ({length}) of the method name exceeded the limit ({limit}) in a Function Call action."
            ),
            ActionsValidationError::FunctionCallArgumentsLengthExceeded { length, limit } => format!(
                "Error: The length ({length}) of the arguments exceeded the limit ({limit}) in a Function Call action."
            ),
            ActionsValidationError::UnsuitableStakingKey { public_key } => format!(
                "Error: An attempt to stake with a public key <{public_key}> that is not convertible to ristretto."
            ),
            ActionsValidationError::FunctionCallZeroAttachedGas => {
                "Error: The attached amount of gas in a FunctionCall action has to be a positive number.".to_string()
            }
            ActionsValidationError::DelegateActionMustBeOnlyOne => {
                "Error: The transaction contains more than one delegation action".to_string()
            }
            ActionsValidationError::UnsupportedProtocolFeature {
                protocol_feature,
                version,
            } => format!(
                "Error: Protocol Feature {protocol_feature} is unsupported in version {version}"
            ),
            ActionsValidationError::InvalidDeterministicStateInitReceiver {
                receiver_id,
                derived_id,
            } => format!(
                "Error: Invalid reciever account id <{receiver_id}> for deterministic account id <{derived_id}>."
            ),
            ActionsValidationError::DeterministicStateInitKeyLengthExceeded { length, limit } => format!(
                "Error: DeterministicStateInit key length is {length} but the limit is {limit}."
            ),
            ActionsValidationError::DeterministicStateInitValueLengthExceeded { length, limit } => format!(
                "Error: DeterministicStateInit contains value of length {length} but at most {limit} is allowed."
            ),
            ActionsValidationError::GasKeyInvalidNumNonces {
                requested_nonces,
                limit,
            } => format!(
                "Error: Gas key requested invalid number of nonces: {requested_nonces} (must be between 1 and {limit})."
            ),
            ActionsValidationError::AddGasKeyWithNonZeroBalance { balance } => format!(
                "Error: Adding a gas key with non-zero balance is not allowed: balance = {}.",
                balance.exact_amount_display()
            ),
            ActionsValidationError::GasKeyFunctionCallAllowanceNotAllowed => {
                "Error: Gas keys with FunctionCall permission cannot have an allowance set.".to_string()
            }
            ActionsValidationError::TotalNumberOfDeployActionsExceeded {
                number_of_deploy_actions,
                limit,
            } => format!(
                "Error: The combined number of DeployContract and DeployGlobalContract actions ({number_of_deploy_actions}) in one receipt exceeded the limit ({limit})."
            ),
            ActionsValidationError::Unknown(error) => format!("Error: {error}"),
        },
        InvalidTxError::TransactionSizeExceeded { size, limit } => format!(
            "Error: The size ({size}) of serialized transaction exceeded the limit ({limit})."
        ),
        InvalidTxError::InvalidTransactionVersion => {
            "Error: Invalid transaction version".to_string()
        }
        InvalidTxError::StorageError(error) => match error {
            StorageError::StorageInternalError => "Error: Internal storage error".to_string(),
            StorageError::MissingTrieValue(value) => format!(
                "Error: Requested trie value by its hash ({}) which is missing in the storage",
                value.hash
            ),
            StorageError::UnexpectedTrieValue => "Error: Unexpected trie value".to_string(),
            StorageError::StorageInconsistentState(message) => {
                format!("Error: The storage is in the inconsistent state: {message}")
            }
            StorageError::FlatStorageBlockNotSupported(message) => {
                format!("Error: The block is not supported by flat storage: {message}")
            }
            StorageError::MemTrieLoadingError(message) => {
                format!("Error: The trie is not loaded in memory: {message}")
            }
            StorageError::Unknown(error) => format!("Error: {error}"),
        },
        InvalidTxError::ShardCongested {
            shard_id,
            congestion_level,
        } => format!(
            "Error: The shard ({shard_id}) is too congested ({congestion_level:.2}/1.00) and can't accept new transaction"
        ),
        InvalidTxError::ShardStuck {
            shard_id,
            missed_chunks,
        } => format!(
            "Error: The shard ({shard_id}) is {missed_chunks} blocks behind and can't accept new transaction until it will be in the sync"
        ),
        InvalidTxError::InvalidNonceIndex {
            tx_nonce_index,
            num_nonces,
        } => format!(
            "Error: Invalid nonce_index {tx_nonce_index:?} for key with {num_nonces} nonces."
        ),
        InvalidTxError::NotEnoughGasKeyBalance {
            signer_id,
            balance,
            cost,
        } => format!(
            "Error: Gas key for <{signer_id}> does not have enough balance ({}) for gas cost ({}).",
            balance.exact_amount_display(),
            cost.exact_amount_display()
        ),
        InvalidTxError::NotEnoughBalanceForDeposit {
            signer_id,
            balance,
            cost,
            ..
        } => format!(
            "Error: Sender <{signer_id}> does not have enough balance ({}) to cover deposit cost ({}).",
            balance.exact_amount_display(),
            cost.exact_amount_display()
        ),
        InvalidTxError::Unknown(error) => format!("Error: {error}"),
    };

    Err(color_eyre::Report::msg(message))
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

    let mut success_data = String::new();
    #[allow(unused_assignments)]
    let mut return_value = String::new();
    let mut returned_value_bytes: Vec<u8> = Vec::new();

    let retries_number = 5;
    let mut retries = (1..=retries_number).rev();
    let mut status = transaction_info.status.clone();
    let result = loop {
        match &status {
            near_kit::FinalExecutionStatus::NotStarted
            | near_kit::FinalExecutionStatus::Started => {
                let message = if let near_kit::FinalExecutionStatus::NotStarted = &status {
                    "The execution has not yet started."
                } else {
                    "The execution has started and still going."
                };

                if let Some(retries_left) = retries.next() {
                    crate::transaction_signature_options::send::sleep_before_retry(format!(
                        "{} ({} Will retry {} more times)",
                        network_config.rpc_url,
                        message.red(),
                        retries_left
                    ));
                } else if let crate::Verbosity::Quiet = verbosity {
                    return Ok(());
                } else {
                    tracing::warn!(
                        parent: &tracing::Span::none(),
                        "{}{}",
                        message.red(),
                        indent_payload(&format!(
                            "\nPlease, check the transaction status later using the transaction ID: {}",
                            transaction_info.transaction_outcome.id
                        )).yellow()
                    );
                    return Ok(());
                }

                let rpc_transaction_response =
                    crate::commands::transaction::view_status::get_transaction_info(
                        network_config,
                        transaction_info.transaction_outcome.id,
                    )?;

                if let Some(final_execution_outcome) =
                    rpc_transaction_response.final_execution_outcome()
                {
                    status = final_execution_outcome.status;
                }
            }
            near_kit::FinalExecutionStatus::Failure(tx_execution_error) => {
                return match tx_execution_error {
                    near_kit::TxExecutionError::ActionError(action_error) => {
                        convert_action_error_to_cli_result(action_error)
                    }
                    near_kit::TxExecutionError::InvalidTxError(invalid_tx_error) => {
                        convert_invalid_tx_error_to_cli_result(invalid_tx_error)
                    }
                };
            }
            near_kit::FinalExecutionStatus::SuccessValue(bytes_result) => {
                let bytes_result = base64::engine::general_purpose::STANDARD
                    .decode(bytes_result)
                    .wrap_err("Failed to decode transaction return value")?;
                if let crate::Verbosity::Quiet = verbosity {
                    std::io::stdout().write_all(&bytes_result)?;
                    return Ok(());
                };
                returned_value_bytes.extend_from_slice(&bytes_result);
                return_value = if bytes_result.is_empty() {
                    "Empty return value".to_string()
                } else if let Ok(json_result) =
                    serde_json::from_slice::<serde_json::Value>(&bytes_result)
                {
                    serde_json::to_string_pretty(&json_result)?
                } else if let Ok(string_result) = String::from_utf8(bytes_result.clone()) {
                    string_result
                } else {
                    "The returned value is not printable (binary data)".to_string()
                };
                success_data.push_str(&return_value);
                break Ok(());
            }
        };
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

    if success_data.is_empty() {
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

    if !success_data.is_empty() {
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
            path_with_key_name.display()
        )
    } else {
        std::fs::File::create(&path_with_key_name)
            .wrap_err_with(|| format!("Failed to create file: {path_with_key_name:?}"))?
            .write(key_pair_properties_buf.as_bytes())
            .wrap_err_with(|| format!("Failed to write to file: {path_with_key_name:?}"))?;
        format!(
            "The data for the access key is saved in a file {}",
            path_with_key_name.display()
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
            path_with_account_name.display()
        ))
    } else {
        std::fs::File::create(&path_with_account_name)
            .wrap_err_with(|| format!("Failed to create file: {path_with_account_name:?}"))?
            .write(key_pair_properties_buf.as_bytes())
            .wrap_err_with(|| format!("Failed to write to file: {path_with_account_name:?}"))?;
        Ok(format!(
            "{}\nThe data for the access key is saved in a file {}",
            message_1,
            path_with_account_name.display()
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
        .map(|x| format!("{:?}", x).to_lowercase())
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

    let epoch_validator_info =
        block_on(network_config.client().rpc().validators(None)).into_eyre()?;

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
    let mut used_delegated_validator_list: VecDeque<near_kit::AccountId> = VecDeque::new();

    for used_account in used_account_list {
        if delegated_validator_list.remove(&used_account.account_id) {
            used_delegated_validator_list.push_back(used_account.account_id);
        }
    }

    used_delegated_validator_list.extend(delegated_validator_list);
    Ok(used_delegated_validator_list)
}

pub fn input_staking_pool_validator_account_id(
    context: &crate::GlobalContext,
) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
    let used_delegated_validator_list = if !context.offline {
        get_used_delegated_validator_list(&context.config)?
            .into_iter()
            .map(String::from)
            .collect::<Vec<_>>()
    } else {
        vec![]
    };
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
        &context.config.credentials_home_dir,
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
                get_staking_pool_info(rpc, validator_account_id.clone(), *stake).await
            })
            .buffer_unordered(concurrency)
            .try_collect::<Vec<_>>(),
    )?;
    validator_list.sort_by_key(|b| std::cmp::Reverse(b.stake));
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

    let values = block_on(network_config.client().rpc().view_state_all(
        staking_pools_factory_account_id,
        b"se",
        0,
        near_kit::BlockReference::final_(),
    ))
    .into_eyre()?;

    Ok(values
        .into_iter()
        .filter_map(|item| borsh::from_slice(&item.value).ok())
        .collect())
}

#[tracing::instrument(name = "Getting a stake of validators ...", skip_all)]
pub fn get_validators_stake(
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<std::collections::HashMap<near_kit::AccountId, near_token::NearToken>>
{
    tracing::info!(target: "near_teach_me", "Getting a stake of validators ...");
    let epoch_validator_info =
        block_on(network_config.client().rpc().validators(None)).into_eyre()?;

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
        Ok(result) => Some(result.json::<RewardFeeFraction>().wrap_err(
            "Failed to parse return value of view function call for RewardFeeFraction.",
        )?),
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

pub fn display_gas_key_nonces(public_key: &near_kit::PublicKey, nonces: &[u64]) {
    eprintln!("Gas key nonces for public key {public_key}:");
    let mut table = Table::new();
    table.set_titles(prettytable::row![Fg=>"Nonce index", "Nonce"]);

    for (nonce_index, nonce) in nonces.iter().enumerate() {
        table.add_row(prettytable::row![Fg->nonce_index, nonce]);
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

/// Run a future on a new single-threaded tokio runtime.
///
/// This is a thin helper for calling near-kit async methods from synchronous
/// code paths.  All of the old `blocking_*` wrappers have been replaced by
/// `block_on(…)` calls at the respective callsites.
pub fn block_on<F: std::future::Future>(f: F) -> F::Output {
    tokio::runtime::Runtime::new().unwrap().block_on(f)
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

/// Extension trait to convert `Result<T, near_kit::RpcError>` into
/// `Result<T, color_eyre::eyre::Error>` via `.into_eyre()`.
pub trait RpcResultExt<T> {
    fn into_eyre(self) -> color_eyre::eyre::Result<T>;
}

impl<T> RpcResultExt<T> for Result<T, near_kit::RpcError> {
    fn into_eyre(self) -> color_eyre::eyre::Result<T> {
        self.map_err(|err| color_eyre::eyre::eyre!("{}", err))
    }
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
            .wrap_err_with(|| format!("Failed to create file: {:?}", used_account_list_path))?;
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
    !get_used_ft_contract_account_list(credentials_home_dir).is_empty()
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
    new_ft_contract: &crate::types::ft_properties::FtContract,
) {
    let mut ft_contract_accounts_list = get_used_ft_contract_account_list(credentials_home_dir);

    let used_ft_contract = ft_contract_accounts_list
        .iter()
        .position(|ft_contract| {
            ft_contract.ft_contract_account_id == new_ft_contract.ft_contract_account_id
        })
        .and_then(|position| ft_contract_accounts_list.remove(position))
        .unwrap_or_else(|| new_ft_contract.clone());

    ft_contract_accounts_list.push_front(used_ft_contract);

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

/// Parses a JSON object with base64-encoded keys and values into a `BTreeMap<Vec<u8>, Vec<u8>>`.
///
/// Example input: `{"AAEC": "AwQF"}` (standard base64, padding optional)
/// Empty state: `{}`
pub fn parse_base64_kv_map(
    input: &str,
) -> color_eyre::eyre::Result<std::collections::BTreeMap<Vec<u8>, Vec<u8>>> {
    #[serde_as]
    #[derive(serde::Deserialize)]
    struct Data(
        #[serde_as(as = "std::collections::BTreeMap<Base64, Base64>")]
        std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    );

    let data: Data = serde_json::from_str(input)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to parse base64 KV map: {e}"))?;
    Ok(data.0)
}

/// Deserializes a `StateInit` from borsh-serialized bytes.
pub fn parse_borsh_base64_state_init(
    bytes: &[u8],
) -> color_eyre::eyre::Result<near_kit::StateInit> {
    use borsh::BorshDeserialize;
    near_kit::StateInit::try_from_slice(bytes)
        .map_err(|e| color_eyre::eyre::eyre!("Failed to borsh-deserialize state init: {e}"))
}
