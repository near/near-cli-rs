use std::convert::{TryFrom, TryInto};
use std::io::Write;
use std::str::FromStr;

use prettytable::Table;

use near_primitives::{
    borsh::BorshDeserialize, hash::CryptoHash, types::BlockReference,
    views::AccessKeyPermissionView,
};

pub type CliResult = color_eyre::eyre::Result<()>;

use inquire::Select;
use strum::IntoEnumIterator;

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
pub struct SignedTransactionAsBase64 {
    pub inner: near_primitives::transaction::SignedTransaction,
}

impl std::str::FromStr for SignedTransactionAsBase64 {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            inner: near_primitives::transaction::SignedTransaction::try_from_slice(
                &near_primitives::serialize::from_base64(s)
                    .map_err(|err| format!("base64 transaction sequence is invalid: {}", err))?,
            )
            .map_err(|err| format!("transaction could not be parsed: {}", err))?,
        })
    }
}

impl std::fmt::Display for SignedTransactionAsBase64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.get_hash())
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
            inner: bs58::decode(s)
                .into_vec()
                .map_err(|err| format!("base58 block hash sequence is invalid: {}", err))?
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

const ONE_NEAR: u128 = 10u128.pow(24);

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd)]
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
        let currency = s.trim().trim_start_matches(num).trim().to_uppercase();
        let yoctonear_amount = match currency.as_str() {
            "N" | "NEAR" => {
                let res_split: Vec<&str> = num.split('.').collect();
                match res_split.len() {
                    2 => {
                        let num_int_yocto = res_split[0]
                            .parse::<u128>()
                            .map_err(|err| format!("Near Balance: {}", err))?
                            .checked_mul(10u128.pow(24))
                            .ok_or("Near Balance: underflow or overflow happens")?;
                        let len_fract = res_split[1].len() as u32;
                        let num_fract_yocto = if len_fract <= 24 {
                            res_split[1]
                                .parse::<u128>()
                                .map_err(|err| format!("Near Balance: {}", err))?
                                .checked_mul(10u128.pow(24 - res_split[1].len() as u32))
                                .ok_or("Near Balance: underflow or overflow happens")?
                        } else {
                            return Err(
                                "Near Balance: too large fractional part of a number".to_string()
                            );
                        };
                        num_int_yocto
                            .checked_add(num_fract_yocto)
                            .ok_or("Near Balance: underflow or overflow happens")?
                    }
                    1 => {
                        if res_split[0].starts_with('0') && res_split[0] != "0" {
                            return Err("Near Balance: incorrect number entered".to_string());
                        };
                        res_split[0]
                            .parse::<u128>()
                            .map_err(|err| format!("Near Balance: {}", err))?
                            .checked_mul(10u128.pow(24))
                            .ok_or("Near Balance: underflow or overflow happens")?
                    }
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

impl interactive_clap::ToCli for NearBalance {
    type CliVariant = NearBalance;
}

const ONE_TERA_GAS: u64 = 10u64.pow(12);
const ONE_GIGA_GAS: u64 = 10u64.pow(9);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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
        let currency = s.trim().trim_start_matches(num).trim().to_uppercase();
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
                    .ok_or("Near Gas: underflow or overflow happens")?;
                let len_fract = res_split[1].len() as u32;
                let num_fract_gas = if len_fract <= 12 {
                    res_split[1]
                        .parse::<u64>()
                        .map_err(|err| format!("Near Gas: {}", err))?
                        .checked_mul(10u64.pow(12 - res_split[1].len() as u32))
                        .ok_or("Near Gas: underflow or overflow happens")?
                } else {
                    return Err("Near Gas: too large fractional part of a number".to_string());
                };
                Ok(num_int_gas
                    .checked_add(num_fract_gas)
                    .ok_or("Near Gas: underflow or overflow happens")?)
            }
            1 => Ok(res_split[0]
                .parse::<u64>()
                .map_err(|err| format!("Near Gas: {}", err))?
                .checked_mul(10u64.pow(12))
                .ok_or("Near Gas: underflow or overflow happens")?),
            _ => Err("Near Gas: incorrect number entered".to_string()),
        }
    }
}

impl interactive_clap::ToCli for NearGas {
    type CliVariant = NearGas;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd)]
pub struct TransferAmount {
    amount: NearBalance,
}

impl interactive_clap::ToCli for TransferAmount {
    type CliVariant = NearBalance;
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

pub async fn get_account_transfer_allowance(
    network_config: crate::config::NetworkConfig,
    account_id: near_primitives::types::AccountId,
    block_reference: BlockReference,
) -> color_eyre::eyre::Result<AccountTransferAllowance> {
    let account_view = if let Ok(account_view) =
        get_account_state(network_config.clone(), account_id.clone(), block_reference).await
    {
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
    let storage_amount_per_byte = network_config
        .json_rpc_client()
        .call(
            near_jsonrpc_client::methods::EXPERIMENTAL_protocol_config::RpcProtocolConfigRequest {
                block_reference: near_primitives::types::BlockReference::Finality(
                    near_primitives::types::Finality::Final,
                ),
            },
        )
        .await
        .map_err(|err| color_eyre::Report::msg(format!("RpcError: {:?}", err)))?
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

pub async fn verify_account_access_key(
    account_id: near_primitives::types::AccountId,
    public_key: near_crypto::PublicKey,
    network_config: crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<
    near_primitives::views::AccessKeyView,
    near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>,
> {
    loop {
        match network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKey {
                    account_id: account_id.clone(),
                    public_key: public_key.clone(),
                },
            })
            .await
        {
            Ok(rpc_query_response) => {
                if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKey(result) =
                    rpc_query_response.kind
                {
                    return Ok(result);
                } else {
                    return Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(near_jsonrpc_client::errors::RpcTransportError::RecvError(
                        near_jsonrpc_client::errors::JsonRpcTransportRecvError::UnexpectedServerResponse(
                            near_jsonrpc_primitives::message::Message::error(near_jsonrpc_primitives::errors::RpcError::parse_error("Transport error: unexpected server response".to_string()))
                        ),
                    )));
                }
            }
            Err(
                err @ near_jsonrpc_client::errors::JsonRpcError::ServerError(
                    near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                        near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccessKey {
                            ..
                        },
                    ),
                ),
            ) => {
                return Err(err);
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(err)) => {
                println!("\nAccount information ({}) cannot be fetched on <{}> network due to connectivity issue.",
                    account_id, network_config.network_name
                );
                if !need_check_account() {
                    return Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(
                        err,
                    ));
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(err)) => {
                println!("\nAccount information ({}) cannot be fetched on <{}> network due to server error.",
                    account_id, network_config.network_name
                );
                if !need_check_account() {
                    return Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(err));
                }
            }
        }
    }
}

pub async fn get_account_state(
    network_config: crate::config::NetworkConfig,
    account_id: near_primitives::types::AccountId,
    block_reference: BlockReference,
) -> color_eyre::eyre::Result<
    near_primitives::views::AccountView,
    near_jsonrpc_client::errors::JsonRpcError<near_jsonrpc_primitives::types::query::RpcQueryError>,
> {
    loop {
        let query_view_method_response = network_config
            .json_rpc_client()
            .call(near_jsonrpc_client::methods::query::RpcQueryRequest {
                block_reference: block_reference.clone(),
                request: near_primitives::views::QueryRequest::ViewAccount {
                    account_id: account_id.clone(),
                },
            })
            .await;
        match query_view_method_response {
            Ok(rpc_query_response) => {
                if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(
                    account_view,
                ) = rpc_query_response.kind
                {
                    return Ok(account_view);
                } else {
                    return Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(near_jsonrpc_client::errors::RpcTransportError::RecvError(
                        near_jsonrpc_client::errors::JsonRpcTransportRecvError::UnexpectedServerResponse(
                            near_jsonrpc_primitives::message::Message::error(near_jsonrpc_primitives::errors::RpcError::parse_error("Transport error: unexpected server response".to_string()))
                        ),
                    )));
                }
            }
            Err(
                err @ near_jsonrpc_client::errors::JsonRpcError::ServerError(
                    near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                        near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccount {
                            ..
                        },
                    ),
                ),
            ) => {
                return Err(err);
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(err)) => {
                println!("\nAccount information ({}) cannot be fetched on <{}> network due to connectivity issue.",
                    account_id, network_config.network_name
                );
                if !need_check_account() {
                    return Err(near_jsonrpc_client::errors::JsonRpcError::TransportError(
                        err,
                    ));
                }
            }
            Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(err)) => {
                println!("\nAccount information ({}) cannot be fetched on <{}> network due to server error.",
                    account_id, network_config.network_name
                );
                if !need_check_account() {
                    return Err(near_jsonrpc_client::errors::JsonRpcError::ServerError(err));
                }
            }
        }
    }
}

fn need_check_account() -> bool {
    #[derive(strum_macros::Display, PartialEq)]
    enum ConfirmOptions {
        #[strum(to_string = "Yes, I want to check the account again.")]
        Yes,
        #[strum(to_string = "No, I want to skip the check and use the specified account ID.")]
        No,
    }
    let select_choose_input = Select::new(
        "Do you want to try again?",
        vec![ConfirmOptions::Yes, ConfirmOptions::No],
    )
    .prompt()
    .unwrap_or(ConfirmOptions::Yes);
    select_choose_input == ConfirmOptions::Yes
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct KeyPairProperties {
    pub seed_phrase_hd_path: crate::types::slip10::BIP32Path,
    pub master_seed_phrase: String,
    pub implicit_account_id: near_primitives::types::AccountId,
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
    let derived_private_key = slip10::derive_key_from_path(
        &master_seed,
        slip10::Curve::Ed25519,
        &seed_phrase_hd_path.clone().into(),
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
        near_primitives::types::AccountId::try_from(hex::encode(secret_keypair.public))?;
    let public_key_str = format!(
        "ed25519:{}",
        bs58::encode(&secret_keypair.public).into_string()
    );
    let secret_keypair_str = format!(
        "ed25519:{}",
        bs58::encode(secret_keypair.to_bytes()).into_string()
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
    seed_phrase_hd_path: slip10::BIP32Path,
    master_seed_phrase: &str,
) -> color_eyre::eyre::Result<near_crypto::PublicKey> {
    let master_seed = bip39::Mnemonic::parse(master_seed_phrase)?.to_seed("");
    let derived_private_key =
        slip10::derive_key_from_path(&master_seed, slip10::Curve::Ed25519, &seed_phrase_hd_path)
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
    let public_key_str = format!(
        "ed25519:{}",
        bs58::encode(&secret_keypair.public).into_string()
    );
    Ok(near_crypto::PublicKey::from_str(&public_key_str)?)
}

pub async fn generate_keypair() -> color_eyre::eyre::Result<KeyPairProperties> {
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
            let master_seed_phrase = mnemonic.word_iter().collect::<Vec<&str>>().join(" ");
            (master_seed_phrase, mnemonic.to_seed(""))
        };

    let derived_private_key = slip10::derive_key_from_path(
        &master_seed,
        slip10::Curve::Ed25519,
        &generate_keypair.seed_phrase_hd_path.clone().into(),
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
        near_primitives::types::AccountId::try_from(hex::encode(secret_keypair.public))?;
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
                    "Access key <{}> for account <{}> has been successfully deleted.",
                    public_key, transaction_info.transaction.signer_id,
                );
            }
            near_primitives::views::ActionView::DeleteAccount { beneficiary_id: _ } => {
                println!(
                    "Account <{}> has been successfully deleted.",
                    transaction_info.transaction.signer_id,
                );
            }
        }
    }
}

pub fn rpc_transaction_error(
    err: near_jsonrpc_client::errors::JsonRpcError<
        near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError,
    >,
) -> CliResult {
    match &err {
        near_jsonrpc_client::errors::JsonRpcError::TransportError(_rpc_transport_error) => {
            println!("Transport error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
        }
        near_jsonrpc_client::errors::JsonRpcError::ServerError(rpc_server_error) => match rpc_server_error {
            near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(rpc_transaction_error) => match rpc_transaction_error {
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::TimeoutError => {
                    println!("Timeout error transaction.\nPlease wait. The next try to send this transaction is happening right now ...");
                }
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::InvalidTransaction { context } => {
                    let err_invalid_transaction = crate::common::handler_invalid_tx_error(context.clone());
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("{}", err_invalid_transaction));
                }
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::DoesNotTrackShard => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("RPC Server Error: {}", err));
                }
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::RequestRouted{transaction_hash} => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("RPC Server Error for transaction with hash {}\n{}", transaction_hash, err));
                }
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::UnknownTransaction{requested_transaction_hash} => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("RPC Server Error for transaction with hash {}\n{}", requested_transaction_hash, err));
                }
                near_jsonrpc_client::methods::broadcast_tx_commit::RpcTransactionError::InternalError{debug_info} => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("RPC Server Error: {}", debug_info));
                }
            }
            near_jsonrpc_client::errors::JsonRpcServerError::RequestValidationError(rpc_request_validation_error) => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Incompatible request with the server: {:#?}",  rpc_request_validation_error));
            }
            near_jsonrpc_client::errors::JsonRpcServerError::InternalError{ info } => {
                println!("Internal server error: {}.\nPlease wait. The next try to send this transaction is happening right now ...", info.clone().unwrap_or_default());
            }
            near_jsonrpc_client::errors::JsonRpcServerError::NonContextualError(rpc_error) => {
                return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("Unexpected response: {}", rpc_error));
            }
            near_jsonrpc_client::errors::JsonRpcServerError::ResponseStatusError(json_rpc_server_response_status_error) => match json_rpc_server_response_status_error {
                near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::Unauthorized => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("JSON RPC server requires authentication. Please, authenticate near CLI with the JSON RPC server you use."));
                }
                near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::TooManyRequests => {
                    println!("JSON RPC server is currently busy.\nPlease wait. The next try to send this transaction is happening right now ...");
                }
                near_jsonrpc_client::errors::JsonRpcServerResponseStatusError::Unexpected{status} => {
                    return color_eyre::eyre::Result::Err(color_eyre::eyre::eyre!("JSON RPC server responded with an unexpected status code: {}", status));
                }
            }
        }
    }
    Ok(())
}

pub fn print_action_error(action_error: near_primitives::errors::ActionError) {
    match action_error.kind {
        near_primitives::errors::ActionErrorKind::AccountAlreadyExists { account_id } => {
            println!("Error: Create Account action tries to create an account with account ID <{}> which already exists in the storage.", account_id)
        }
        near_primitives::errors::ActionErrorKind::AccountDoesNotExist { account_id } => {
            println!(
                "Error: TX receiver ID <{}> doesn't exist (but action is not \"Create Account\").",
                account_id
            )
        }
        near_primitives::errors::ActionErrorKind::CreateAccountOnlyByRegistrar {
            account_id: _,
            registrar_account_id: _,
            predecessor_id: _,
        } => {
            println!("Error: A top-level account ID can only be created by registrar.")
        }
        near_primitives::errors::ActionErrorKind::CreateAccountNotAllowed {
            account_id,
            predecessor_id,
        } => {
            println!("Error: A newly created account <{}> must be under a namespace of the creator account <{}>.", account_id, predecessor_id)
        }
        near_primitives::errors::ActionErrorKind::ActorNoPermission {
            account_id: _,
            actor_id: _,
        } => {
            println!("Error: Administrative actions can be proceed only if sender=receiver or the first TX action is a \"Create Account\" action.")
        }
        near_primitives::errors::ActionErrorKind::DeleteKeyDoesNotExist {
            account_id,
            public_key,
        } => {
            println!(
                "Error: Account <{}>  tries to remove an access key <{}> that doesn't exist.",
                account_id, public_key
            )
        }
        near_primitives::errors::ActionErrorKind::AddKeyAlreadyExists {
            account_id,
            public_key,
        } => {
            println!(
                "Error: Public key <{}> is already used for an existing account ID <{}>.",
                public_key, account_id
            )
        }
        near_primitives::errors::ActionErrorKind::DeleteAccountStaking { account_id } => {
            println!(
                "Error: Account <{}> is staking and can not be deleted",
                account_id
            )
        }
        near_primitives::errors::ActionErrorKind::LackBalanceForState { account_id, amount } => {
            println!("Error: Receipt action can't be completed, because the remaining balance will not be enough to cover storage.\nAn account which needs balance: <{}>\nBalance required to complete the action: <{}>",
                account_id,
                crate::common::NearBalance::from_yoctonear(amount)
            )
        }
        near_primitives::errors::ActionErrorKind::TriesToUnstake { account_id } => {
            println!(
                "Error: Account <{}> is not yet staked, but tries to unstake.",
                account_id
            )
        }
        near_primitives::errors::ActionErrorKind::TriesToStake {
            account_id,
            stake,
            locked: _,
            balance,
        } => {
            println!(
                "Error: Account <{}> doesn't have enough balance ({}) to increase the stake ({}).",
                account_id,
                crate::common::NearBalance::from_yoctonear(balance),
                crate::common::NearBalance::from_yoctonear(stake)
            )
        }
        near_primitives::errors::ActionErrorKind::InsufficientStake {
            account_id: _,
            stake,
            minimum_stake,
        } => {
            println!(
                "Error: Insufficient stake {}.\nThe minimum rate must be {}.",
                crate::common::NearBalance::from_yoctonear(stake),
                crate::common::NearBalance::from_yoctonear(minimum_stake)
            )
        }
        near_primitives::errors::ActionErrorKind::FunctionCallError(function_call_error_ser) => {
            println!("Error: An error occurred during a `FunctionCall` Action, parameter is debug message.\n{:?}", function_call_error_ser)
        }
        near_primitives::errors::ActionErrorKind::NewReceiptValidationError(
            receipt_validation_error,
        ) => {
            println!("Error: Error occurs when a new `ActionReceipt` created by the `FunctionCall` action fails.\n{:?}", receipt_validation_error)
        }
        near_primitives::errors::ActionErrorKind::OnlyImplicitAccountCreationAllowed {
            account_id: _,
        } => {
            println!("Error: `CreateAccount` action is called on hex-characters account of length 64.\nSee implicit account creation NEP: https://github.com/nearprotocol/NEPs/pull/71")
        }
        near_primitives::errors::ActionErrorKind::DeleteAccountWithLargeState { account_id } => {
            println!(
                "Error: Delete account <{}> whose state is large is temporarily banned.",
                account_id
            )
        }
    }
}

pub fn handler_invalid_tx_error(
    invalid_tx_error: near_primitives::errors::InvalidTxError,
) -> String {
    match invalid_tx_error {
        near_primitives::errors::InvalidTxError::InvalidAccessKeyError(invalid_access_key_error) => {
            match invalid_access_key_error {
                near_primitives::errors::InvalidAccessKeyError::AccessKeyNotFound{account_id, public_key} => {
                    format!("Error: Public key {} doesn't exist for the account <{}>.", public_key, account_id)
                },
                near_primitives::errors::InvalidAccessKeyError::ReceiverMismatch{tx_receiver, ak_receiver} => {
                    format!("Error: Transaction for <{}> doesn't match the access key for <{}>.", tx_receiver, ak_receiver)
                },
                near_primitives::errors::InvalidAccessKeyError::MethodNameMismatch{method_name} => {
                    format!("Error: Transaction method name <{}> isn't allowed by the access key.", method_name)
                },
                near_primitives::errors::InvalidAccessKeyError::RequiresFullAccess => {
                    "Error: Transaction requires a full permission access key.".to_string()
                },
                near_primitives::errors::InvalidAccessKeyError::NotEnoughAllowance{account_id, public_key, allowance, cost} => {
                    format!("Error: Access Key <{}> for account <{}> does not have enough allowance ({}) to cover transaction cost ({}).",
                        public_key,
                        account_id,
                        crate::common::NearBalance::from_yoctonear(allowance),
                        crate::common::NearBalance::from_yoctonear(cost)
                    )
                },
                near_primitives::errors::InvalidAccessKeyError::DepositWithFunctionCall => {
                    "Error: Having a deposit with a function call action is not allowed with a function call access key.".to_string()
                }
            }
        },
        near_primitives::errors::InvalidTxError::InvalidSignerId { signer_id } => {
            format!("Error: TX signer ID <{}> is not in a valid format or does not satisfy requirements\nSee \"near_runtime_utils::utils::is_valid_account_id\".", signer_id)
        },
        near_primitives::errors::InvalidTxError::SignerDoesNotExist { signer_id } => {
            format!("Error: TX signer ID <{}> is not found in the storage.", signer_id)
        },
        near_primitives::errors::InvalidTxError::InvalidNonce { tx_nonce, ak_nonce } => {
            format!("Error: Transaction nonce ({}) must be account[access_key].nonce ({}) + 1.", tx_nonce, ak_nonce)
        },
        near_primitives::errors::InvalidTxError::NonceTooLarge { tx_nonce, upper_bound } => {
            format!("Error: Transaction nonce ({}) is larger than the upper bound ({}) given by the block height.", tx_nonce, upper_bound)
        },
        near_primitives::errors::InvalidTxError::InvalidReceiverId { receiver_id } => {
            format!("Error: TX receiver ID ({}) is not in a valid format or does not satisfy requirements\nSee \"near_runtime_utils::is_valid_account_id\".", receiver_id)
        },
        near_primitives::errors::InvalidTxError::InvalidSignature => {
            "Error: TX signature is not valid".to_string()
        },
        near_primitives::errors::InvalidTxError::NotEnoughBalance {signer_id, balance, cost} => {
            format!("Error: Account <{}> does not have enough balance ({}) to cover TX cost ({}).",
                signer_id,
                crate::common::NearBalance::from_yoctonear(balance),
                crate::common::NearBalance::from_yoctonear(cost)
            )
        },
        near_primitives::errors::InvalidTxError::LackBalanceForState {signer_id, amount} => {
            format!("Error: Signer account <{}> doesn't have enough balance ({}) after transaction.",
                signer_id,
                crate::common::NearBalance::from_yoctonear(amount)
            )
        },
        near_primitives::errors::InvalidTxError::CostOverflow => {
            "Error: An integer overflow occurred during transaction cost estimation.".to_string()
        },
        near_primitives::errors::InvalidTxError::InvalidChain => {
            "Error: Transaction parent block hash doesn't belong to the current chain.".to_string()
        },
        near_primitives::errors::InvalidTxError::Expired => {
            "Error: Transaction has expired.".to_string()
        },
        near_primitives::errors::InvalidTxError::ActionsValidation(actions_validation_error) => {
            match actions_validation_error {
                near_primitives::errors::ActionsValidationError::DeleteActionMustBeFinal => {
                    "Error: The delete action must be the final action in transaction.".to_string()
                },
                near_primitives::errors::ActionsValidationError::TotalPrepaidGasExceeded {total_prepaid_gas, limit} => {
                    format!("Error: The total prepaid gas ({}) for all given actions exceeded the limit ({}).",
                    total_prepaid_gas,
                    limit
                    )
                },
                near_primitives::errors::ActionsValidationError::TotalNumberOfActionsExceeded {total_number_of_actions, limit} => {
                    format!("Error: The number of actions ({}) exceeded the given limit ({}).", total_number_of_actions, limit)
                },
                near_primitives::errors::ActionsValidationError::AddKeyMethodNamesNumberOfBytesExceeded {total_number_of_bytes, limit} => {
                    format!("Error: The total number of bytes ({}) of the method names exceeded the limit ({}) in a Add Key action.", total_number_of_bytes, limit)
                },
                near_primitives::errors::ActionsValidationError::AddKeyMethodNameLengthExceeded {length, limit} => {
                    format!("Error: The length ({}) of some method name exceeded the limit ({}) in a Add Key action.", length, limit)
                },
                near_primitives::errors::ActionsValidationError::IntegerOverflow => {
                    "Error: Integer overflow.".to_string()
                },
                near_primitives::errors::ActionsValidationError::InvalidAccountId {account_id} => {
                    format!("Error: Invalid account ID <{}>.", account_id)
                },
                near_primitives::errors::ActionsValidationError::ContractSizeExceeded {size, limit} => {
                    format!("Error: The size ({}) of the contract code exceeded the limit ({}) in a DeployContract action.", size, limit)
                },
                near_primitives::errors::ActionsValidationError::FunctionCallMethodNameLengthExceeded {length, limit} => {
                    format!("Error: The length ({}) of the method name exceeded the limit ({}) in a Function Call action.", length, limit)
                },
                near_primitives::errors::ActionsValidationError::FunctionCallArgumentsLengthExceeded {length, limit} => {
                    format!("Error: The length ({}) of the arguments exceeded the limit ({}) in a Function Call action.", length, limit)
                },
                near_primitives::errors::ActionsValidationError::UnsuitableStakingKey {public_key} => {
                    format!("Error: An attempt to stake with a public key <{}> that is not convertible to ristretto.", public_key)
                },
                near_primitives::errors::ActionsValidationError::FunctionCallZeroAttachedGas => {
                    "Error: The attached amount of gas in a FunctionCall action has to be a positive number.".to_string()
                }
            }
        },
        near_primitives::errors::InvalidTxError::TransactionSizeExceeded { size, limit } => {
            format!("Error: The size ({}) of serialized transaction exceeded the limit ({}).", size, limit)
        }
    }
}

pub fn print_transaction_error(tx_execution_error: near_primitives::errors::TxExecutionError) {
    println!("Failed transaction");
    match tx_execution_error {
        near_primitives::errors::TxExecutionError::ActionError(action_error) => {
            print_action_error(action_error)
        }
        near_primitives::errors::TxExecutionError::InvalidTxError(invalid_tx_error) => {
            println!("{}", handler_invalid_tx_error(invalid_tx_error))
        }
    }
}

pub fn print_transaction_status(
    transaction_info: near_primitives::views::FinalExecutionOutcomeView,
    network_config: crate::config::NetworkConfig,
) -> crate::CliResult {
    println!("-------------- Logs ----------------");
    for receipt in transaction_info.receipts_outcome.iter() {
        if receipt.outcome.logs.is_empty() {
            println!("Logs [{}]:   No logs", receipt.outcome.executor_id);
        } else {
            println!("Logs [{}]:", receipt.outcome.executor_id);
            println!("  {}", receipt.outcome.logs.join("\n  "));
        };
    }
    println!("------------------------------------");
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
    println!("Transaction ID: {id}\nTo see the transaction in the transaction explorer, please open this url in your browser:\n{path}{id}\n",
        id=transaction_info.transaction_outcome.id,
        path=network_config.explorer_transaction_url
    );
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn save_access_key_to_macos_keychain(
    network_config: crate::config::NetworkConfig,
    key_pair_properties_buf: &str,
    public_key_str: &str,
    account_id: &str,
) -> color_eyre::eyre::Result<String> {
    let keychain = security_framework::os::macos::keychain::SecKeychain::default()
        .map_err(|err| color_eyre::Report::msg(format!("Failed to open keychain: {:?}", err)))?;
    let service_name = std::borrow::Cow::Owned(format!(
        "near-{}-{}",
        network_config.network_name, account_id
    ));
    keychain
        .set_generic_password(
            &service_name,
            &format!("{}:{}", account_id, public_key_str),
            key_pair_properties_buf.as_bytes(),
        )
        .map_err(|err| {
            color_eyre::Report::msg(format!("Failed to save password to keychain: {:?}", err))
        })?;
    Ok("The data for the access key is saved in macOS Keychain".to_string())
}

pub fn save_access_key_to_keychain(
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
            .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
            .write(key_pair_properties_buf.as_bytes())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
            })?;
        format!(
            "The data for the access key is saved in a file {}",
            &path_with_key_name.display()
        )
    };

    let file_with_account_name: std::path::PathBuf = format!("{}.json", account_id).into();
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
            .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
            .write(key_pair_properties_buf.as_bytes())
            .map_err(|err| {
                color_eyre::Report::msg(format!("Failed to write to file: {:?}", err))
            })?;
        Ok(format!(
            "{}\nThe data for the access key is saved in a file {}",
            message_1,
            &path_with_account_name.display()
        ))
    }
}

pub fn get_config_toml() -> color_eyre::eyre::Result<crate::config::Config> {
    if let Some(mut path_config_toml) = dirs::config_dir() {
        path_config_toml.push("near-cli");
        std::fs::create_dir_all(&path_config_toml)?;
        path_config_toml.push("config.toml");

        if !path_config_toml.is_file() {
            write_config_toml(crate::config::Config::default())?;
        };
        let config_toml = std::fs::read_to_string(path_config_toml)?;
        Ok(toml::from_str(&config_toml)?)
    } else {
        Ok(crate::config::Config::default())
    }
}

pub fn write_config_toml(config: crate::config::Config) -> CliResult {
    let config_toml = toml::to_string(&config)?;
    let mut path_config_toml = dirs::config_dir().expect("Impossible to get your config dir!");
    path_config_toml.push("near-cli/config.toml");
    std::fs::File::create(&path_config_toml)
        .map_err(|err| color_eyre::Report::msg(format!("Failed to create file: {:?}", err)))?
        .write(config_toml.as_bytes())
        .map_err(|err| color_eyre::Report::msg(format!("Failed to write to file: {:?}", err)))?;
    println!(
        "Configuration data is stored in a file {:?}",
        &path_config_toml
    );
    Ok(())
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

    let err = match cargo_util::ProcessBuilder::new(command)
        .args(&args)
        .exec_replace()
    {
        Ok(()) => return Ok(()),
        Err(e) => e,
    };

    if let Some(perr) = err.downcast_ref::<cargo_util::ProcessError>() {
        if let Some(code) = perr.code {
            return Err(color_eyre::eyre::eyre!("perror occurred, code: {}", code));
        }
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
    let mut dirs = vec![];
    if let Some(val) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&val));
    }
    dirs
}

pub fn display_account_info(
    viewed_at_block_hash: &CryptoHash,
    viewed_at_block_height: &near_primitives::types::BlockHeight,
    account_id: &crate::types::account_id::AccountId,
    account_view: &near_primitives::views::AccountView,
    access_keys: &[near_primitives::views::AccessKeyInfoView],
) {
    let mut table = Table::new();
    table.set_format(*prettytable::format::consts::FORMAT_NO_COLSEP);

    table.add_row(prettytable::row![
        Fy->account_id,
        format!("At block #{}\n({})", viewed_at_block_height, viewed_at_block_hash)
    ]);
    table.add_row(prettytable::row![
        Fg->"Native account balance",
        Fy->NearBalance::from_yoctonear(account_view.amount)
    ]);
    table.add_row(prettytable::row![
        Fg->"Validator stake",
        Fy->NearBalance::from_yoctonear(account_view.locked)
    ]);
    table.add_row(prettytable::row![
        Fg->"Storage used by the account",
        Fy->bytesize::ByteSize(account_view.storage_usage),
    ]);

    let contract_status = if account_view.code_hash == CryptoHash::default() {
        "No contract code".to_string()
    } else {
        hex::encode(account_view.code_hash.as_ref())
    };
    table.add_row(prettytable::row![
        Fg->"Contract (SHA-256 checksum hex)",
        Fy->contract_status
    ]);

    let access_keys_summary = if access_keys.is_empty() {
        "Account is locked (no access keys)".to_string()
    } else {
        let full_access_keys_count = access_keys
            .iter()
            .filter(|access_key| {
                matches!(
                    access_key.access_key.permission,
                    near_primitives::views::AccessKeyPermissionView::FullAccess
                )
            })
            .count();
        format!(
            "{} full access keys and {} function-call-only access keys",
            full_access_keys_count,
            access_keys.len() - full_access_keys_count
        )
    };
    table.add_row(prettytable::row![
        Fg->"Access keys",
        Fy->access_keys_summary
    ]);

    table.printstd();

    if !access_keys.is_empty() {
        display_access_key_list(access_keys);
    }
}

pub fn display_access_key_list(access_keys: &[near_primitives::views::AccessKeyInfoView]) {
    let mut table = Table::new();
    table.set_titles(prettytable::row![Fg=>"#", "Public Key", "Nonce", "Permissions"]);

    for (index, access_key) in access_keys.iter().enumerate() {
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
                    None => "with no limit".to_string(),
                };
                if method_names.is_empty() {
                    format!(
                        "do any function calls on {} {}",
                        receiver_id, allowance_message
                    )
                } else {
                    format!(
                        "only do {:?} function calls on {} {}",
                        method_names, receiver_id, allowance_message
                    )
                }
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

pub fn input_network_name(context: &crate::GlobalContext) -> color_eyre::eyre::Result<String> {
    let variants = context.0.networks.keys().collect::<Vec<_>>();
    let select_submit = Select::new("What is the name of the network?", variants).prompt()?;
    Ok(select_submit.clone())
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
    fn near_balance_from_str_0_near() {
        assert_eq!(
            NearBalance::from_str("0 near").unwrap(),
            NearBalance {
                yoctonear_amount: 0
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
    fn near_balance_from_str_05_near() {
        let near_balance = NearBalance::from_str("05NEAR");
        assert_eq!(
            near_balance,
            Err("Near Balance: incorrect number entered".to_string())
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
    fn near_gas_from_str_currency_tgas() {
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
