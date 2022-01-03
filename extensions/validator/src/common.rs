use std::convert::TryInto;

use near_jsonrpc_client::methods::EXPERIMENTAL_genesis_config::RpcGenesisConfigResponse;
use near_primitives::{borsh::BorshDeserialize, views::EpochValidatorInfo};

pub type CliResult = color_eyre::eyre::Result<()>;

use dialoguer::{theme::ColorfulTheme, Select};
use strum::{EnumMessage, IntoEnumIterator};
pub fn prompt_variant<T>(prompt: &str) -> T
where
    T: IntoEnumIterator + EnumMessage,
    T: Copy + Clone,
{
    let variants = T::iter().collect::<Vec<_>>();
    let actions = variants
        .iter()
        .map(|p| {
            p.get_message()
                .unwrap_or_else(|| "error[This entry does not have an option message!!]")
                .to_owned()
        })
        .collect::<Vec<_>>();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&actions)
        .default(0)
        .interact()
        .unwrap();

    variants[selected]
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
                let client = near_jsonrpc_client::JsonRpcClient::connect(&url.as_str());
                let status_method = near_jsonrpc_client::methods::status::RpcStatusRequest;
                client.call(&status_method).await
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

impl interactive_clap::ToCli for AvailableRpcServerUrl {
    type CliVariant = AvailableRpcServerUrl;
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

    pub fn from_custom_url(custom_url: &AvailableRpcServerUrl) -> Self {
        Self::Custom {
            url: custom_url.inner.clone(),
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

#[derive(Debug, Clone)]
pub struct KeyPairProperties {
    pub seed_phrase_hd_path: slip10::BIP32Path,
    pub master_seed_phrase: String,
    pub implicit_account_id: near_primitives::types::AccountId,
    pub public_key_str: String,
    pub secret_keypair_str: String,
}

pub async fn validators_info(
    epoch: near_primitives::types::EpochReference,
    network_connection_config: &crate::common::ConnectionConfig,
) -> (RpcGenesisConfigResponse, EpochValidatorInfo) {
    let client =
        near_jsonrpc_client::JsonRpcClient::connect(network_connection_config.rpc_url().as_str());

    let genesis_config_request =
        near_jsonrpc_client::methods::EXPERIMENTAL_genesis_config::RpcGenesisConfigRequest;

    let genesis_config = client.clone().call(&genesis_config_request).await.unwrap();

    let validators_request = near_jsonrpc_client::methods::validators::RpcValidatorRequest {
        epoch_reference: epoch,
    };

    let validator_info = client.call(&validators_request).await.unwrap();

    (genesis_config, validator_info)
}

pub async fn display_validators_info(
    epoch: near_primitives::types::EpochReference,
    network_connection_config: &crate::common::ConnectionConfig,
) -> crate::CliResult {
    let (genesis_config, validator_info) = validators_info(epoch, network_connection_config).await;

    //TODO: make it pretty
    println!("-------------- Validators info (should be in table) ----------------------");
    println!("Genesis config: {:?}", genesis_config);
    println!("------------------------------------");
    println!("Validator info: {:?}", validator_info);

    Ok(())
}

pub async fn display_proposals_info(
    network_connection_config: &crate::common::ConnectionConfig,
) -> crate::CliResult {
    let (genesis_config, validator_info) = validators_info(
        near_primitives::types::EpochReference::Latest,
        network_connection_config,
    )
    .await;

    //TODO: make it pretty
    println!("-------------- Proposals info (should be in table) ----------------------");
    println!("Genesis config: {:?}", genesis_config);
    println!("------------------------------------");
    println!("Validator info: {:?}", validator_info);

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
