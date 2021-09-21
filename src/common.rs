use std::convert::{TryFrom, TryInto};
use std::io::Write;

use near_jsonrpc_client::new_client;
use near_jsonrpc_primitives::types::query::{QueryResponseKind, RpcQueryRequest};
use near_primitives::{
    borsh::BorshDeserialize,
    hash::CryptoHash,
    types::{AccountId, BlockReference},
    views::{AccessKeyPermissionView, QueryRequest},
};

pub type CliResult = color_eyre::eyre::Result<()>;

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
pub struct TransactionAsBase64 {
    pub inner: near_primitives::transaction::Transaction,
}

impl std::str::FromStr for TransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::transaction::Transaction::try_from_slice(
                &near_primitives::serialize::from_base64(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {}", err))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {}", err))?,
        })
    }
}

impl std::fmt::Display for TransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.get_hash_and_size().0)
    }
}

#[derive(Debug, Clone)]
pub struct BlockHashAsBase58 {
    pub inner: near_primitives::hash::CryptoHash,
}

impl std::str::FromStr for BlockHashAsBase58 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::serialize::from_base(s)
                .map_err(|err| format!("base block hash sequence is invalid: {}", err))?
                .as_slice()
                .try_into()
                .map_err(|err| format!("block hash could not be collected: {}", err))?,
        })
    }
}

impl std::fmt::Display for BlockHashAsBase58 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlockHash {}", self.inner)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AvailableRpcServerUrl {
    pub inner: url::Url,
}

impl std::str::FromStr for AvailableRpcServerUrl {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url: url::Url =
            url::Url::parse(s).map_err(|err| format!("URL is not parsed: {}", err))?;
        actix::System::new()
            .block_on(async {
                near_jsonrpc_client::new_client(&url.as_str())
                    .status()
                    .await
            })
            .map_err(|err| format!("AvailableRpcServerUrl: {:?}", err))?;
        Ok(Self { inner: url })
    }
}

impl std::fmt::Display for AvailableRpcServerUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

const ONE_NEAR: u128 = 10u128.pow(24);

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct NearBalance {
    pub yoctonear_amount: u128,
}

impl NearBalance {
    pub fn from_yoctonear(yoctonear_amount: u128) -> Self {
        Self { yoctonear_amount }
    }

    pub fn to_yoctonear(&self) -> u128 {
        self.yoctonear_amount
    }
}

impl std::fmt::Display for NearBalance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.yoctonear_amount == 0 {
            write!(f, "0 NEAR")
        } else if self.yoctonear_amount % ONE_NEAR == 0 {
            write!(f, "{} NEAR", self.yoctonear_amount / ONE_NEAR,)
        } else {
            write!(
                f,
                "{}.{} NEAR",
                self.yoctonear_amount / ONE_NEAR,
                format!("{:0>24}", (self.yoctonear_amount % ONE_NEAR)).trim_end_matches('0')
            )
        }
    }
}

impl std::str::FromStr for NearBalance {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
        let currency = s.trim().trim_start_matches(&num).trim().to_uppercase();
        let yoctonear_amount = match currency.as_str() {
            "N" | "NEAR" => {
                let res_split: Vec<&str> = num.split('.').collect();
                match res_split.len() {
                    2 => {
                        let num_int_yocto = res_split[0]
                            .parse::<u128>()
                            .map_err(|err| format!("Near Balance: {}", err))?
                            .checked_mul(10u128.pow(24))
                            .ok_or_else(|| "Near Balance: underflow or overflow happens")?;
                        let len_fract = res_split[1].len() as u32;
                        let num_fract_yocto = if len_fract <= 24 {
                            res_split[1]
                                .parse::<u128>()
                                .map_err(|err| format!("Near Balance: {}", err))?
                                .checked_mul(10u128.pow(24 - res_split[1].len() as u32))
                                .ok_or_else(|| "Near Balance: underflow or overflow happens")?
                        } else {
                            return Err(
                                "Near Balance: too large fractional part of a number".to_string()
                            );
                        };
                        num_int_yocto
                            .checked_add(num_fract_yocto)
                            .ok_or_else(|| "Near Balance: underflow or overflow happens")?
                    }
                    1 => res_split[0]
                        .parse::<u128>()
                        .map_err(|err| format!("Near Balance: {}", err))?
                        .checked_mul(10u128.pow(24))
                        .ok_or_else(|| "Near Balance: underflow or overflow happens")?,
                    _ => return Err("Near Balance: incorrect number entered".to_string()),
                }
            }
            "YN" | "YNEAR" | "YOCTONEAR" | "YOCTON" => num
                .parse::<u128>()
                .map_err(|err| format!("Near Balance: {}", err))?,
            _ => return Err("Near Balance: incorrect currency value entered".to_string()),
        };
        Ok(NearBalance { yoctonear_amount })
    }
}

const ONE_TERA_GAS: u64 = 10u64.pow(12);
const ONE_GIGA_GAS: u64 = 10u64.pow(9);

#[derive(Debug, Clone, Default, PartialEq)]
pub struct NearGas {
    pub inner: u64,
}

impl std::fmt::Display for NearGas {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.inner == 0 {
            write!(f, "0 Gas")
        } else if self.inner >= ONE_TERA_GAS {
            write!(
                f,
                "{}.{:0>3} TeraGas",
                self.inner / ONE_TERA_GAS,
                self.inner / (ONE_TERA_GAS / 1000) % 1000
            )
        } else {
            write!(
                f,
                "{}.{:0>3} GigaGas",
                self.inner / ONE_GIGA_GAS,
                self.inner / (ONE_GIGA_GAS / 1000) % 1000
            )
        }
    }
}

impl std::str::FromStr for NearGas {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = s.trim().trim_end_matches(char::is_alphabetic).trim();
        let currency = s.trim().trim_start_matches(&num).trim().to_uppercase();
        let number = match currency.as_str() {
            "T" | "TGAS" | "TERAGAS" => NearGas::into_tera_gas(num)?,
            "GIGAGAS" | "GGAS" => NearGas::into_tera_gas(num)? / 1000,
            _ => return Err("Near Gas: incorrect currency value entered".to_string()),
        };
        Ok(NearGas { inner: number })
    }
}

impl From<u64> for NearGas {
    fn from(num: u64) -> Self {
        Self { inner: num }
    }
}

impl NearGas {
    fn into_tera_gas(num: &str) -> Result<u64, String> {
        let res_split: Vec<&str> = num.split('.').collect();
        match res_split.len() {
            2 => {
                let num_int_gas: u64 = res_split[0]
                    .parse::<u64>()
                    .map_err(|err| format!("Near Gas: {}", err))?
                    .checked_mul(10u64.pow(12))
                    .ok_or_else(|| "Near Gas: underflow or overflow happens")?;
                let len_fract = res_split[1].len() as u32;
                let num_fract_gas = if len_fract <= 12 {
                    res_split[1]
                        .parse::<u64>()
                        .map_err(|err| format!("Near Gas: {}", err))?
                        .checked_mul(10u64.pow(12 - res_split[1].len() as u32))
                        .ok_or_else(|| "Near Gas: underflow or overflow happens")?
                } else {
                    return Err("Near Gas: too large fractional part of a number".to_string());
                };
                Ok(num_int_gas
                    .checked_add(num_fract_gas)
                    .ok_or_else(|| "Near Gas: underflow or overflow happens")?)
            }
            1 => Ok(res_split[0]
                .parse::<u64>()
                .map_err(|err| format!("Near Gas: {}", err))?
                .checked_mul(10u64.pow(12))
                .ok_or_else(|| "Near Gas: underflow or overflow happens")?),
            _ => return Err("Near Gas: incorrect number entered".to_string()),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, PartialOrd)]
pub struct TransferAmount {
    amount: NearBalance,
}

impl std::fmt::Display for TransferAmount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.amount)
    }
}

impl TransferAmount {
    pub fn from(
        amount: NearBalance,
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

    pub fn from_unchecked(amount: NearBalance) -> Self {
        Self { amount }
    }

    pub fn to_yoctonear(&self) -> u128 {
        self.amount.to_yoctonear()
    }
}

impl From<TransferAmount> for NearBalance {
    fn from(item: TransferAmount) -> Self {
        item.amount
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionConfig {
    Testnet,
    Mainnet,
    Betanet,
    Custom { url: url::Url },
}

impl ConnectionConfig {
    pub fn rpc_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_API_SERVER_URL.parse().unwrap(),
            Self::Mainnet => crate::consts::MAINNET_API_SERVER_URL.parse().unwrap(),
            Self::Betanet => crate::consts::BETANET_API_SERVER_URL.parse().unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn archival_rpc_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_ARCHIVAL_API_SERVER_URL
                .parse()
                .unwrap(),
            Self::Mainnet => crate::consts::MAINNET_ARCHIVAL_API_SERVER_URL
                .parse()
                .unwrap(),
            Self::Betanet => crate::consts::BETANET_ARCHIVAL_API_SERVER_URL
                .parse()
                .unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn wallet_url(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_WALLET_URL.parse().unwrap(),
            Self::Mainnet => crate::consts::MAINNET_WALLET_URL.parse().unwrap(),
            Self::Betanet => crate::consts::BETANET_WALLET_URL.parse().unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn transaction_explorer(&self) -> url::Url {
        match self {
            Self::Testnet => crate::consts::TESTNET_TRANSACTION_URL.parse().unwrap(),
            Self::Mainnet => crate::consts::MAINNET_TRANSACTION_URL.parse().unwrap(),
            Self::Betanet => crate::consts::BETANET_TRANSACTION_URL.parse().unwrap(),
            Self::Custom { url } => url.clone(),
        }
    }

    pub fn dir_name(&self) -> &str {
        match self {
            Self::Testnet => crate::consts::DIR_NAME_TESTNET,
            Self::Mainnet => crate::consts::DIR_NAME_MAINNET,
            Self::Betanet => crate::consts::DIR_NAME_BETANET,
            Self::Custom { url: _ } => crate::consts::DIR_NAME_CUSTOM,
        }
    }
}

#[derive(Debug)]
pub struct AccountTransferAllowance {
    account_id: near_primitives::types::AccountId,
    account_liquid_balance: NearBalance,
    account_locked_balance: NearBalance,
    storage_stake: NearBalance,
    pessimistic_transaction_fee: NearBalance,
}

impl std::fmt::Display for AccountTransferAllowance {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt,
            "\n{} account has {} available for transfer (the total balance is {}, but {} is locked for storage and the transfer transaction fee is ~{})",
            self.account_id,
            self.transfer_allowance(),
            self.account_liquid_balance,
            self.liquid_storage_stake(),
            self.pessimistic_transaction_fee
        )
    }
}

impl AccountTransferAllowance {
    pub fn liquid_storage_stake(&self) -> NearBalance {
        NearBalance::from_yoctonear(
            self.storage_stake
                .to_yoctonear()
                .saturating_sub(self.account_locked_balance.to_yoctonear()),
        )
    }

    pub fn transfer_allowance(&self) -> NearBalance {
        NearBalance::from_yoctonear(
            self.account_liquid_balance.to_yoctonear()
                - self.liquid_storage_stake().to_yoctonear()
                - self.pessimistic_transaction_fee.to_yoctonear(),
        )
    }
}

pub fn get_account_transfer_allowance(
    connection_config: &ConnectionConfig,
    account_id: near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<AccountTransferAllowance> {
    let account_view =
        if let Some(account_view) = get_account_state(connection_config, account_id.clone())? {
            account_view
        } else {
            return Ok(AccountTransferAllowance {
                account_id,
                account_liquid_balance: NearBalance::from_yoctonear(0),
                account_locked_balance: NearBalance::from_yoctonear(0),
                storage_stake: NearBalance::from_yoctonear(0),
                pessimistic_transaction_fee: NearBalance::from_yoctonear(0),
            });
        };
    let storage_amount_per_byte = actix::System::new()
        .block_on(async {
            near_jsonrpc_client::new_client(connection_config.rpc_url().as_str())
                .EXPERIMENTAL_protocol_config(
                    near_jsonrpc_primitives::types::config::RpcProtocolConfigRequest {
                        block_reference:
                            near_jsonrpc_primitives::types::blocks::BlockReference::Finality(
                                near_primitives::types::Finality::Final,
                            ),
                    },
                )
                .await
        })
        .map_err(|err| color_eyre::Report::msg(format!("RpcError: {:?}", err)))?
        .config_view
        .runtime_config
        .storage_amount_per_byte;

    Ok(AccountTransferAllowance {
        account_id,
        account_liquid_balance: NearBalance::from_yoctonear(account_view.amount),
        account_locked_balance: NearBalance::from_yoctonear(account_view.locked),
        storage_stake: NearBalance::from_yoctonear(
            u128::from(account_view.storage_usage) * storage_amount_per_byte,
        ),
        // pessimistic_transaction_fee = 10^21 - this value is set temporarily
        // In the future, its value will be calculated by the function: fn tx_cost(...)
        // https://github.com/near/nearcore/blob/8a377fda0b4ce319385c463f1ae46e4b0b29dcd9/runtime/runtime/src/config.rs#L178-L232
        pessimistic_transaction_fee: NearBalance::from_yoctonear(10u128.pow(21)),
    })
}

pub fn get_account_state(
    connection_config: &ConnectionConfig,
    account_id: near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Option<near_primitives::views::AccountView>> {
    let query_view_method_response = actix::System::new().block_on(async {
        near_jsonrpc_client::new_client(connection_config.rpc_url().as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccount { account_id },
            })
            .await
    });
    match query_view_method_response {
        Ok(rpc_query_response) => {
            let account_view =
                if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(
                    result,
                ) = rpc_query_response.kind
                {
                    result
                } else {
                    return Err(color_eyre::Report::msg(format!("Error call result")));
                };
            Ok(Some(account_view.into()))
        }
        Err(_) => return Ok(None),
    }
}

/// Returns true if the account ID length is 64 characters and it's a hex representation. This is used to check the implicit account.
pub fn is_64_len_hex(account_id: impl AsRef<str>) -> bool {
    let account_id = account_id.as_ref();
    account_id.len() == 64
        && account_id
            .as_bytes()
            .iter()
            .all(|b| matches!(b, b'a'..=b'f' | b'0'..=b'9'))
}

#[derive(Debug, Clone)]
pub struct KeyPairProperties {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub master_seed_phrase: String,
    pub implicit_account_id: near_primitives::types::AccountId,
    pub public_key_str: String,
    pub secret_keypair_str: String,
}

pub async fn generate_keypair() -> color_eyre::eyre::Result<KeyPairProperties> {
    let generate_keypair: crate::commands::utils_command::generate_keypair_subcommand::CliGenerateKeypair =
        crate::commands::utils_command::generate_keypair_subcommand::CliGenerateKeypair::default();
    let (master_seed_phrase, master_seed) =
        if let Some(master_seed_phrase) = generate_keypair.master_seed_phrase.as_deref() {
            (
                master_seed_phrase.to_owned(),
                bip39::Mnemonic::parse(master_seed_phrase)?.to_seed(""),
            )
        } else {
            let mnemonic =
                bip39::Mnemonic::generate(generate_keypair.new_master_seed_phrase_words_count)?;
            let master_seed_phrase = mnemonic.word_iter().collect::<Vec<&str>>().join(" ");
            (master_seed_phrase, mnemonic.to_seed(""))
        };

    let derived_private_key = slip10::derive_key_from_path(
        &master_seed,
        slip10::Curve::Ed25519,
        &generate_keypair.seed_phrase_hd_path,
    )
    .map_err(|err| {
        color_eyre::Report::msg(format!(
            "Failed to derive a key from the master key: {}",
            err
        ))
    })?;

    let secret_keypair = {
        let secret = ed25519_dalek::SecretKey::from_bytes(&derived_private_key.key)?;
        let public = ed25519_dalek::PublicKey::from(&secret);
        ed25519_dalek::Keypair { secret, public }
    };

    let implicit_account_id =
        near_primitives::types::AccountId::try_from(hex::encode(&secret_keypair.public))?;
    let public_key_str = format!(
        "ed25519:{}",
        bs58::encode(&secret_keypair.public).into_string()
    );
    let secret_keypair_str = format!(
        "ed25519:{}",
        bs58::encode(secret_keypair.to_bytes()).into_string()
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

pub fn print_transaction(transaction: near_primitives::transaction::Transaction) {
    println!("{:<13} {}", "signer_id:", &transaction.signer_id);
    println!("{:<13} {}", "public_key:", &transaction.public_key);
    println!("{:<13} {}", "nonce:", &transaction.nonce);
    println!("{:<13} {}", "receiver_id:", &transaction.receiver_id);
    println!("{:<13} {}", "block_hash:", &transaction.block_hash);
    println!("actions:");
    let actions = transaction.actions.clone();
    for action in actions {
        match action {
            near_primitives::transaction::Action::CreateAccount(_) => {
                println!(
                    "{:>5} {:<20} {}",
                    "--", "create account:", &transaction.receiver_id
                )
            }
            near_primitives::transaction::Action::DeployContract(_) => {
                println!("{:>5} {:<20}", "--", "deploy contract")
            }
            near_primitives::transaction::Action::FunctionCall(function_call_action) => {
                println!("{:>5} {:<20}", "--", "function call:");
                println!(
                    "{:>18} {:<13} {}",
                    "", "method name:", &function_call_action.method_name
                );
                println!(
                    "{:>18} {:<13} {:?}",
                    "", "args:", &function_call_action.args
                );
                println!(
                    "{:>18} {:<13} {}",
                    "",
                    "gas:",
                    crate::common::NearGas {
                        inner: function_call_action.gas
                    }
                );
                println!(
                    "{:>18} {:<13} {}",
                    "",
                    "deposit:",
                    crate::common::NearBalance::from_yoctonear(function_call_action.deposit)
                );
            }
            near_primitives::transaction::Action::Transfer(transfer_action) => {
                println!(
                    "{:>5} {:<20} {}",
                    "--",
                    "transfer deposit:",
                    crate::common::NearBalance::from_yoctonear(transfer_action.deposit)
                );
            }
            near_primitives::transaction::Action::Stake(stake_action) => {
                println!("{:>5} {:<20}", "--", "stake:");
                println!(
                    "{:>18} {:<13} {}",
                    "", "public key:", &stake_action.public_key
                );
                println!(
                    "{:>18} {:<13} {}",
                    "",
                    "stake:",
                    crate::common::NearBalance::from_yoctonear(stake_action.stake)
                );
            }
            near_primitives::transaction::Action::AddKey(add_key_action) => {
                println!("{:>5} {:<20}", "--", "add access key:");
                println!(
                    "{:>18} {:<13} {}",
                    "", "public key:", &add_key_action.public_key
                );
                println!(
                    "{:>18} {:<13} {}",
                    "", "nonce:", &add_key_action.access_key.nonce
                );
                println!(
                    "{:>18} {:<13} {:?}",
                    "", "permission:", &add_key_action.access_key.permission
                );
            }
            near_primitives::transaction::Action::DeleteKey(delete_key_action) => {
                println!("{:>5} {:<20}", "--", "delete access key:");
                println!(
                    "{:>18} {:<13} {}",
                    "", "public key:", &delete_key_action.public_key
                );
            }
            near_primitives::transaction::Action::DeleteAccount(delete_account_action) => {
                println!(
                    "{:>5} {:<20} {}",
                    "--", "delete account:", &transaction.receiver_id
                );
                println!(
                    "{:>5} {:<20} {}",
                    "", "beneficiary id:", &delete_account_action.beneficiary_id
                );
            }
        }
    }
}

fn print_value_successful_transaction(
    transaction_info: near_primitives::views::FinalExecutionOutcomeView,
) {
    println!("Successful transaction");
    for action in transaction_info.transaction.actions {
        match action {
            near_primitives::views::ActionView::CreateAccount => {
                println!(
                    "New account <{}> has been successfully created.",
                    transaction_info.transaction.receiver_id,
                );
            }
            near_primitives::views::ActionView::DeployContract { code: _ } => {
                println!("Contract code has been successfully deployed.",);
            }
            near_primitives::views::ActionView::FunctionCall {
                method_name,
                args: _,
                gas: _,
                deposit: _,
            } => {
                println!(
                    "The \"{}\" call to <{}> on behalf of <{}> succeeded.",
                    method_name,
                    transaction_info.transaction.receiver_id,
                    transaction_info.transaction.signer_id,
                );
            }
            near_primitives::views::ActionView::Transfer { deposit } => {
                println!(
                    "<{}> has transferred {} to <{}> successfully.",
                    transaction_info.transaction.signer_id,
                    crate::common::NearBalance::from_yoctonear(deposit),
                    transaction_info.transaction.receiver_id,
                );
            }
            near_primitives::views::ActionView::Stake {
                stake,
                public_key: _,
            } => {
                println!(
                    "Validator <{}> has successfully staked {}.",
                    transaction_info.transaction.signer_id,
                    crate::common::NearBalance::from_yoctonear(stake),
                );
            }
            near_primitives::views::ActionView::AddKey {
                public_key,
                access_key: _,
            } => {
                println!(
                    "Added access key = {} to {}.",
                    public_key, transaction_info.transaction.receiver_id,
                );
            }
            near_primitives::views::ActionView::DeleteKey { public_key } => {
                println!(
                    "Access key <{}> for account <{}> has been successfully deletted.",
                    public_key, transaction_info.transaction.signer_id,
                );
            }
            near_primitives::views::ActionView::DeleteAccount { beneficiary_id: _ } => {
                println!(
                    "Account <{}> has been successfully deletted.",
                    transaction_info.transaction.signer_id,
                );
            }
        }
    }
}

pub fn print_transaction_error(tx_execution_error: near_primitives::errors::TxExecutionError) {
    println!("Failed transaction");
    match tx_execution_error {
        near_primitives::errors::TxExecutionError::ActionError(action_error) => {
            match action_error.kind {
                near_primitives::errors::ActionErrorKind::AccountAlreadyExists{account_id} => {
                    println!("Error: Create Account action tries to create an account with account ID <{}> which is already exists in the storage.", account_id)
                },
                near_primitives::errors::ActionErrorKind::AccountDoesNotExist{account_id} => {
                    println!("Error: TX receiver ID <{}> doesn't exist (but action is not \"Create Account\").", account_id)
                },
                near_primitives::errors::ActionErrorKind::CreateAccountOnlyByRegistrar{account_id:_, registrar_account_id:_, predecessor_id:_} => {
                    println!("Error: A top-level account ID can only be created by registrar.")
                },
                near_primitives::errors::ActionErrorKind::CreateAccountNotAllowed{account_id, predecessor_id} => {
                    println!("Error: A newly created account <{}> must be under a namespace of the creator account <{}>.", account_id, predecessor_id)
                },
                near_primitives::errors::ActionErrorKind::ActorNoPermission{account_id:_, actor_id:_} => {
                    println!("Error: Administrative actions can be proceed only if sender=receiver or the first TX action is a \"Create Account\" action.")
                },
                near_primitives::errors::ActionErrorKind::DeleteKeyDoesNotExist{account_id, public_key} => {
                    println!("Error: Account <{}>  tries to remove an access key <{}> that doesn't exist.", account_id, public_key)
                },
                near_primitives::errors::ActionErrorKind::AddKeyAlreadyExists{account_id, public_key} => {
                    println!("Error: Public key <{}> is already used for an existing account ID <{}>.", public_key, account_id)
                },
                near_primitives::errors::ActionErrorKind::DeleteAccountStaking{account_id} => {
                    println!("Error: Account <{}> is staking and can not be deleted", account_id)
                },
                near_primitives::errors::ActionErrorKind::LackBalanceForState{account_id, amount} => {
                    println!("Error: Receipt action can't be completed, because the remaining balance will not be enough to cover storage.\nAn account which needs balance: <{}>\nBalance required to complete an action: <{}>",
                        account_id,
                        crate::common::NearBalance::from_yoctonear(amount)
                    )
                },
                near_primitives::errors::ActionErrorKind::TriesToUnstake{account_id} => {
                    println!("Error: Account <{}> is not yet staked, but tries to unstake.", account_id)
                },
                near_primitives::errors::ActionErrorKind::TriesToStake{account_id, stake, locked:_, balance} => {
                    println!("Error: Account <{}> doesn't have enough balance ({}) to increase the stake ({}).",
                    account_id,
                    crate::common::NearBalance::from_yoctonear(balance),
                    crate::common::NearBalance::from_yoctonear(stake)
                    )
                },
                near_primitives::errors::ActionErrorKind::InsufficientStake{account_id:_, stake, minimum_stake} => {
                    println!("Error: Insufficient stake {}.\nThe minimum rate must be {}.",
                        crate::common::NearBalance::from_yoctonear(stake),
                        crate::common::NearBalance::from_yoctonear(minimum_stake)
                    )
                },
                near_primitives::errors::ActionErrorKind::FunctionCallError(function_call_error_ser) => {
                    println!("Error: An error occurred during a `FunctionCall` Action, parameter is debug message.\n{:?}", function_call_error_ser)
                },
                near_primitives::errors::ActionErrorKind::NewReceiptValidationError(receipt_validation_error) => {
                    println!("Error: Error occurs when a new `ActionReceipt` created by the `FunctionCall` action fails.\n{:?}", receipt_validation_error)
                },
                near_primitives::errors::ActionErrorKind::OnlyImplicitAccountCreationAllowed{account_id:_} => {
                    println!("Error: Error occurs when a `CreateAccount` action is called on hex-characters account of length 64.\nSee implicit account creation NEP: https://github.com/nearprotocol/NEPs/pull/71")
                },
                near_primitives::errors::ActionErrorKind::DeleteAccountWithLargeState{account_id} => {
                    println!("Error: Delete account <{}> whose state is large is temporarily banned.", account_id)
                },
            }
        },
        near_primitives::errors::TxExecutionError::InvalidTxError(invalid_tx_error) => {
            match invalid_tx_error {
                near_primitives::errors::InvalidTxError::InvalidAccessKeyError(invalid_access_key_error) => {
                    match invalid_access_key_error {
                        near_primitives::errors::InvalidAccessKeyError::AccessKeyNotFound{account_id, public_key} => {
                            println!("Error: Public key {} doesn't exist for the account <{}>.", public_key, account_id)
                        },
                        near_primitives::errors::InvalidAccessKeyError::ReceiverMismatch{tx_receiver, ak_receiver} => {
                            println!("Error: Transaction for <{}> doesn't match the access key for <{}>.", tx_receiver, ak_receiver)
                        },
                        near_primitives::errors::InvalidAccessKeyError::MethodNameMismatch{method_name} => {
                            println!("Error: Transaction method name <{}> isn't allowed by the access key.", method_name)
                        },
                        near_primitives::errors::InvalidAccessKeyError::RequiresFullAccess => {
                            println!("Error: Transaction requires a full permission access key.")
                        },
                        near_primitives::errors::InvalidAccessKeyError::NotEnoughAllowance{account_id, public_key, allowance, cost} => {
                            println!("Error: Access Key <{}> for account <{}> does not have enough allowance ({}) to cover transaction cost ({}).",
                                public_key,
                                account_id,
                                crate::common::NearBalance::from_yoctonear(allowance),
                                crate::common::NearBalance::from_yoctonear(cost)
                            )
                        },
                        near_primitives::errors::InvalidAccessKeyError::DepositWithFunctionCall => {
                            println!("Error: Having a deposit with a function call action is not allowed with a function call access key.")
                        }
                    }
                },
                near_primitives::errors::InvalidTxError::InvalidSignerId { signer_id } => {
                    println!("Error: TX signer ID <{}> is not in a valid format or not satisfy requirements see \"near_runtime_utils::utils::is_valid_account_id\".", signer_id)
                },
                near_primitives::errors::InvalidTxError::SignerDoesNotExist { signer_id } => {
                    println!("Error: TX signer ID <{}> is not found in a storage.", signer_id)
                },
                near_primitives::errors::InvalidTxError::InvalidNonce { tx_nonce, ak_nonce } => {
                    println!("Error: Transaction nonce ({}) must be account[access_key].nonce ({}) + 1.", tx_nonce, ak_nonce)
                },
                near_primitives::errors::InvalidTxError::NonceTooLarge { tx_nonce, upper_bound } => {
                    println!("Error: Transaction nonce ({}) is larger than the upper bound ({}) given by the block height.", tx_nonce, upper_bound)
                },
                near_primitives::errors::InvalidTxError::InvalidReceiverId { receiver_id } => {
                    println!("Error: TX receiver ID ({}) is not in a valid format or not satisfy requirements see \"near_runtime_utils::is_valid_account_id\".", receiver_id)
                },
                near_primitives::errors::InvalidTxError::InvalidSignature => {
                    println!("Error: TX signature is not valid")
                },
                near_primitives::errors::InvalidTxError::NotEnoughBalance {signer_id, balance, cost} => {
                    println!("Error: Account <{}> does not have enough balance ({}) to cover TX cost ({}).",
                        signer_id,
                        crate::common::NearBalance::from_yoctonear(balance),
                        crate::common::NearBalance::from_yoctonear(cost)
                    )
                },
                near_primitives::errors::InvalidTxError::LackBalanceForState {signer_id, amount} => {
                    println!("Error: Signer account <{}> doesn't have enough balance ({}) after transaction.",
                        signer_id,
                        crate::common::NearBalance::from_yoctonear(amount)
                    )
                },
                near_primitives::errors::InvalidTxError::CostOverflow => {
                    println!("Error: An integer overflow occurred during transaction cost estimation.")
                },
                near_primitives::errors::InvalidTxError::InvalidChain => {
                    println!("Error: Transaction parent block hash doesn't belong to the current chain.")
                },
                near_primitives::errors::InvalidTxError::Expired => {
                    println!("Error: Transaction has expired.")
                },
                near_primitives::errors::InvalidTxError::ActionsValidation(actions_validation_error) => {
                    match actions_validation_error {
                        near_primitives::errors::ActionsValidationError::DeleteActionMustBeFinal => {
                            println!("Error: The delete action must be a final action in transaction.")
                        },
                        near_primitives::errors::ActionsValidationError::TotalPrepaidGasExceeded {total_prepaid_gas, limit} => {
                            println!("Error: The total prepaid gas ({}) for all given actions exceeded the limit ({}).",
                            total_prepaid_gas,
                            limit
                            )
                        },
                        near_primitives::errors::ActionsValidationError::TotalNumberOfActionsExceeded {total_number_of_actions, limit} => {
                            println!("Error: The number of actions ({}) exceeded the given limit ({}).", total_number_of_actions, limit)
                        },
                        near_primitives::errors::ActionsValidationError::AddKeyMethodNamesNumberOfBytesExceeded {total_number_of_bytes, limit} => {
                            println!("Error: The total number of bytes ({}) of the method names exceeded the limit ({}) in a Add Key action.", total_number_of_bytes, limit)
                        },
                        near_primitives::errors::ActionsValidationError::AddKeyMethodNameLengthExceeded {length, limit} => {
                            println!("Error: The length ({}) of some method name exceeded the limit ({}) in a Add Key action.", length, limit)
                        },
                        near_primitives::errors::ActionsValidationError::IntegerOverflow => {
                            println!("Error: Integer overflow during a compute.")
                        },
                        near_primitives::errors::ActionsValidationError::InvalidAccountId {account_id} => {
                            println!("Error: Invalid account ID <{}>.", account_id)
                        },
                        near_primitives::errors::ActionsValidationError::ContractSizeExceeded {size, limit} => {
                            println!("Error: The size ({}) of the contract code exceeded the limit ({}) in a DeployContract action.", size, limit)
                        },
                        near_primitives::errors::ActionsValidationError::FunctionCallMethodNameLengthExceeded {length, limit} => {
                            println!("Error: The length ({}) of the method name exceeded the limit ({}) in a Function Call action.", length, limit)
                        },
                        near_primitives::errors::ActionsValidationError::FunctionCallArgumentsLengthExceeded {length, limit} => {
                            println!("Error: The length ({}) of the arguments exceeded the limit ({}) in a Function Call action.", length, limit)
                        },
                        near_primitives::errors::ActionsValidationError::UnsuitableStakingKey {public_key} => {
                            println!("Error: An attempt to stake with a public key <{}> that is not convertible to ristretto.", public_key)
                        },
                        near_primitives::errors::ActionsValidationError::FunctionCallZeroAttachedGas => {
                            println!("Error: The attached amount of gas in a FunctionCall action has to be a positive number.")
                        }
                    }
                },
                near_primitives::errors::InvalidTxError::TransactionSizeExceeded { size, limit } => {
                    println!("Error: The size ({}) of serialized transaction exceeded the limit ({}).", size, limit)
                }
            }
        },
    }
}

pub fn print_transaction_status(
    transaction_info: near_primitives::views::FinalExecutionOutcomeView,
    network_connection_config: Option<crate::common::ConnectionConfig>,
) {
    match transaction_info.status {
        near_primitives::views::FinalExecutionStatus::NotStarted
        | near_primitives::views::FinalExecutionStatus::Started => unreachable!(),
        near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
            print_transaction_error(tx_execution_error)
        }
        near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
            print_value_successful_transaction(transaction_info.clone())
        }
    };
    let transaction_explorer: url::Url = match network_connection_config {
        Some(connection_config) => connection_config.transaction_explorer(),
        None => unreachable!("Error"),
    };
    println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
        id=transaction_info.transaction_outcome.id,
        path=transaction_explorer
    );
}

pub async fn save_access_key_to_keychain(
    network_connection_config: Option<crate::common::ConnectionConfig>,
    key_pair_properties: crate::common::KeyPairProperties,
    account_id: &str,
) -> crate::CliResult {
    let buf = format!(
        "{}",
        serde_json::json!({
            "master_seed_phrase": key_pair_properties.master_seed_phrase,
            "seed_phrase_hd_path": key_pair_properties.seed_phrase_hd_path.to_string(),
            "account_id": account_id,
            "public_key": key_pair_properties.public_key_str,
            "private_key": key_pair_properties.secret_keypair_str,
        })
    );
    let home_dir = dirs::home_dir().expect("Impossible to get your home dir!");
    let dir_name = match &network_connection_config {
        Some(connection_config) => connection_config.dir_name(),
        None => crate::consts::DIR_NAME_KEY_CHAIN,
    };
    let file_with_key_name: std::path::PathBuf = format!(
        "{}.json",
        key_pair_properties.public_key_str.replace(":", "_")
    )
    .into();
    let mut path_with_key_name = std::path::PathBuf::from(&home_dir);
    path_with_key_name.push(dir_name);
    path_with_key_name.push(account_id);
    std::fs::create_dir_all(&path_with_key_name)?;
    path_with_key_name.push(file_with_key_name);
    std::fs::File::create(&path_with_key_name)
        .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
        .write(buf.as_bytes())
        .map_err(|err| color_eyre::Report::msg(format!("Failed to write to file: {:?}", err)))?;
    println!(
        "The data for the access key is saved in a file {}",
        &path_with_key_name.display()
    );

    let file_with_account_name: std::path::PathBuf = format!("{}.json", account_id).into();
    let mut path_with_account_name = std::path::PathBuf::from(&home_dir);
    path_with_account_name.push(dir_name);
    path_with_account_name.push(file_with_account_name);
    if path_with_account_name.exists() {
        println!(
            "The file: {} already exists! Therefore it was not overwritten.",
            &path_with_account_name.display()
        );
    } else {
        std::fs::File::create(&path_with_account_name)
            .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
            .write(buf.as_bytes())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
            })?;
        println!(
            "The data for the access key is saved in a file {}",
            &path_with_account_name.display()
        );
    };
    Ok(())
}

pub fn try_external_subcommand_execution() -> CliResult {
    let (subcommand, args) = {
        let mut args = std::env::args().skip(1);
        let subcommand = args
            .next()
            .ok_or_else(|| color_eyre::eyre::eyre!("subcommand is not provided"))?;
        (subcommand, args.collect::<Vec<String>>())
    };
    let subcommand_exe = format!("near-cli-{}{}", subcommand, std::env::consts::EXE_SUFFIX);

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

    let err = match cargo_util::ProcessBuilder::new(&command)
        .args(&args)
        .exec_replace()
    {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };

    if let Some(perr) = err.downcast_ref::<cargo_util::ProcessError>() {
        if let Some(code) = perr.code {
            return Err(color_eyre::eyre::eyre!("perror occured, code: {}", code));
        }
    }
    return Err(color_eyre::eyre::eyre!(err));
}

fn is_executable<P: AsRef<std::path::Path>>(path: P) -> bool {
    if cfg!(target_family = "unix") {
        use std::os::unix::prelude::*;
        std::fs::metadata(path)
            .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    } else if cfg!(target_family = "windows") {
        path.as_ref().is_file()
    } else {
        panic!("Unsupported for wasm");
    }
}

fn path_directories() -> Vec<std::path::PathBuf> {
    let mut dirs = vec![];
    if let Some(val) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&val));
    }
    dirs
}

pub async fn display_account_info(
    account_id: AccountId,
    conf: &ConnectionConfig,
    block_ref: BlockReference,
) -> crate::CliResult {
    let resp = new_client(&conf.archival_rpc_url().as_str())
        .query(RpcQueryRequest {
            block_reference: block_ref,
            request: QueryRequest::ViewAccount {
                account_id: account_id.clone(),
            },
        })
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to fetch query for view account: {:?}", err))
        })?;

    let account_view = match resp.kind {
        QueryResponseKind::ViewAccount(view) => view,
        _ => return Err(color_eyre::Report::msg("Error call result")),
    };

    println!(
        "Account details for '{}' at block #{} ({})\n\
        Native account balance: {}\n\
        Validator stake: {}\n\
        Storage used by the account: {} bytes",
        account_id,
        resp.block_height,
        resp.block_hash,
        NearBalance::from_yoctonear(account_view.amount),
        NearBalance::from_yoctonear(account_view.locked),
        account_view.storage_usage
    );

    if account_view.code_hash == CryptoHash::default() {
        println!("Contract code is not deployed to this account.");
    } else {
        println!(
            "Contract code SHA-256 checksum (hex): {}",
            hex::encode(account_view.code_hash.as_ref())
        );
    }
    Ok(())
}

pub async fn display_access_key_list(
    account_id: AccountId,
    conf: &ConnectionConfig,
    block_ref: BlockReference,
) -> crate::CliResult {
    let resp = new_client(&conf.archival_rpc_url().as_str())
        .query(RpcQueryRequest {
            block_reference: block_ref,
            request: QueryRequest::ViewAccessKeyList { account_id },
        })
        .await
        .map_err(|err| {
            color_eyre::Report::msg(format!(
                "Failed to fetch query for view key list: {:?}",
                err
            ))
        })?;

    let view = match resp.kind {
        QueryResponseKind::AccessKeyList(result) => result,
        _ => return Err(color_eyre::Report::msg(format!("Error call result"))),
    };

    println!("Number of access keys: {}", view.keys.len());
    for (index, access_key) in view.keys.iter().enumerate() {
        let permissions_message = match &access_key.access_key.permission {
            AccessKeyPermissionView::FullAccess => "full access".to_owned(),
            AccessKeyPermissionView::FunctionCall {
                allowance,
                receiver_id,
                method_names,
            } => {
                let allowance_message = match allowance {
                    Some(amount) => format!(
                        "with an allowance of {}",
                        NearBalance::from_yoctonear(*amount)
                    ),
                    None => format!("with no limit"),
                };
                format!(
                    "only do {:?} function calls on {} {}",
                    method_names, receiver_id, allowance_message
                )
            }
        };

        println!(
            "{: >4}. {} (nonce: {}) is granted to {}",
            index + 1,
            access_key.public_key,
            access_key.access_key.nonce,
            permissions_message
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn near_balance_to_string_0_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 0
            }
            .to_string(),
            "0 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_0dot02_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 20000000000000000000000 // 23 digits
            }
            .to_string(),
            "0.02 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_0dot1_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 100000000000000000000000 // 24 digits
            }
            .to_string(),
            "0.1 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_0dot00001230045600789_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 12300456007890000000 // 20 digits
            }
            .to_string(),
            "0.00001230045600789 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_10_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 10000000000000000000000000 // 26 digits
            }
            .to_string(),
            "10 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_10dot02_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 10020000000000000000000000 // 26 digits
            }
            .to_string(),
            "10.02 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_1yocto_near() {
        let yocto_near = NearBalance::from_yoctonear(1);
        assert_eq!(
            yocto_near.to_string(),
            "0.000000000000000000000001 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_1_yocto_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 1
            }
            .to_string(),
            "0.000000000000000000000001 NEAR".to_string()
        )
    }
    #[test]
    fn near_balance_to_string_100_yocto_near() {
        assert_eq!(
            NearBalance {
                yoctonear_amount: 100
            }
            .to_string(),
            "0.0000000000000000000001 NEAR".to_string()
        )
    }

    #[test]
    fn near_balance_from_str_currency_near() {
        assert_eq!(
            NearBalance::from_str("10 near").unwrap(),
            NearBalance {
                yoctonear_amount: 10000000000000000000000000 // 26 digits
            }
        );
        assert_eq!(
            NearBalance::from_str("10.055NEAR").unwrap(),
            NearBalance {
                yoctonear_amount: 10055000000000000000000000 // 26 digits
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_n() {
        assert_eq!(
            NearBalance::from_str("10 n").unwrap(),
            NearBalance {
                yoctonear_amount: 10000000000000000000000000 // 26 digits
            }
        );
        assert_eq!(
            NearBalance::from_str("10N ").unwrap(),
            NearBalance {
                yoctonear_amount: 10000000000000000000000000 // 26 digits
            }
        );
    }
    #[test]
    fn near_balance_from_str_f64_near() {
        assert_eq!(
            NearBalance::from_str("0.000001 near").unwrap(),
            NearBalance {
                yoctonear_amount: 1000000000000000000 // 18 digits
            }
        );
    }
    #[test]
    fn near_balance_from_str_f64_near_without_int() {
        let near_balance = NearBalance::from_str(".055NEAR");
        assert_eq!(
            near_balance,
            Err("Near Balance: cannot parse integer from empty string".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_currency_ynear() {
        assert_eq!(
            NearBalance::from_str("100 ynear").unwrap(),
            NearBalance {
                yoctonear_amount: 100
            }
        );
        assert_eq!(
            NearBalance::from_str("100YNEAR ").unwrap(),
            NearBalance {
                yoctonear_amount: 100
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_yn() {
        assert_eq!(
            NearBalance::from_str("9000 YN  ").unwrap(),
            NearBalance {
                yoctonear_amount: 9000
            }
        );
        assert_eq!(
            NearBalance::from_str("0 yn").unwrap(),
            NearBalance {
                yoctonear_amount: 0
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_yoctonear() {
        assert_eq!(
            NearBalance::from_str("111YOCTONEAR").unwrap(),
            NearBalance {
                yoctonear_amount: 111
            }
        );
        assert_eq!(
            NearBalance::from_str("333 yoctonear").unwrap(),
            NearBalance {
                yoctonear_amount: 333
            }
        );
    }
    #[test]
    fn near_balance_from_str_currency_yocton() {
        assert_eq!(
            NearBalance::from_str("10YOCTON").unwrap(),
            NearBalance {
                yoctonear_amount: 10
            }
        );
        assert_eq!(
            NearBalance::from_str("10 yocton      ").unwrap(),
            NearBalance {
                yoctonear_amount: 10
            }
        );
    }
    #[test]
    fn near_balance_from_str_f64_ynear() {
        let near_balance = NearBalance::from_str("0.055yNEAR");
        assert_eq!(
            near_balance,
            Err("Near Balance: invalid digit found in string".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_without_currency() {
        let near_balance = NearBalance::from_str("100");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_incorrect_currency() {
        let near_balance = NearBalance::from_str("100 UAH");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_invalid_double_dot() {
        let near_balance = NearBalance::from_str("100.55.");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_large_fractional_part() {
        let near_balance = NearBalance::from_str("100.1111122222333334444455555 n"); // 25 digits after "."
        assert_eq!(
            near_balance,
            Err("Near Balance: too large fractional part of a number".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_large_int_part() {
        let near_balance = NearBalance::from_str("1234567890123456.0 n");
        assert_eq!(
            near_balance,
            Err("Near Balance: underflow or overflow happens".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_without_fractional_part() {
        let near_balance = NearBalance::from_str("100. n");
        assert_eq!(
            near_balance,
            Err("Near Balance: cannot parse integer from empty string".to_string())
        );
    }
    #[test]
    fn near_balance_from_str_negative_value() {
        let near_balance = NearBalance::from_str("-100 n");
        assert_eq!(
            near_balance,
            Err("Near Balance: invalid digit found in string".to_string())
        );
    }

    #[test]
    fn near_balance_from_str_currency_tgas() {
        assert_eq!(
            NearGas::from_str("10 tgas").unwrap(),
            NearGas {
                inner: 10000000000000 // 14 digits
            }
        );
        assert_eq!(
            NearGas::from_str("10.055TERAGAS").unwrap(),
            NearGas {
                inner: 10055000000000 // 14 digits
            }
        );
    }
    #[test]
    fn near_gas_from_str_currency_gigagas() {
        assert_eq!(
            NearGas::from_str("10 gigagas").unwrap(),
            NearGas { inner: 10000000000 } // 11 digits
        );
        assert_eq!(
            NearGas::from_str("10GGAS ").unwrap(),
            NearGas { inner: 10000000000 } // 11 digits
        );
    }
    #[test]
    fn near_gas_from_str_f64_tgas() {
        assert_eq!(
            NearGas::from_str("0.000001 tgas").unwrap(),
            NearGas { inner: 1000000 } // 7 digits
        );
    }
    #[test]
    fn near_gas_from_str_f64_gas_without_int() {
        let near_gas = NearGas::from_str(".055ggas");
        assert_eq!(
            near_gas,
            Err("Near Gas: cannot parse integer from empty string".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_without_currency() {
        let near_gas = NearGas::from_str("100");
        assert_eq!(
            near_gas,
            Err("Near Gas: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_incorrect_currency() {
        let near_gas = NearGas::from_str("100 UAH");
        assert_eq!(
            near_gas,
            Err("Near Gas: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_invalid_double_dot() {
        let near_gas = NearGas::from_str("100.55.");
        assert_eq!(
            near_gas,
            Err("Near Gas: incorrect currency value entered".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_large_fractional_part() {
        let near_gas = NearGas::from_str("100.1111122222333 ggas"); // 13 digits after "."
        assert_eq!(
            near_gas,
            Err("Near Gas: too large fractional part of a number".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_large_int_part() {
        let near_gas = NearGas::from_str("200123456789123.0 tgas");
        assert_eq!(
            near_gas,
            Err("Near Gas: underflow or overflow happens".to_string())
        );
    }
    #[test]
    fn near_gas_from_str_negative_value() {
        let near_gas = NearGas::from_str("-100 ggas");
        assert_eq!(
            near_gas,
            Err("Near Gas: invalid digit found in string".to_string())
        );
    }
}
