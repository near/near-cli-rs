use rust_decimal::Decimal;
use tracing_indicatif::span_ext::IndicatifSpanExt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FTContract {
    /// Select a specific FT contract to view
    SingleContract(crate::types::account_id::AccountId),
    /// View all FT contracts
    AllContracts,
}

impl interactive_clap::ToCli for FTContract {
    type CliVariant = FTContract;
}

impl std::fmt::Display for FTContract {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SingleContract(ft) => ft.fmt(f),
            Self::AllContracts => write!(f, "all"),
        }
    }
}

impl std::str::FromStr for FTContract {
    type Err = <near_primitives::types::AccountId as std::str::FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.to_lowercase() == "all" {
            Ok(Self::AllContracts)
        } else {
            Ok(Self::SingleContract(
                crate::types::account_id::AccountId::from_str(s)?,
            ))
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Inventory {
    fts: Vec<FTInventory>,
}

impl Inventory {
    pub fn fts(&self) -> Vec<FTInventory> {
        self.fts.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct FTInventory {
    pub amount: String,
    #[serde(rename = "contract")]
    pub ft_contract_account_id: near_primitives::types::AccountId,
    pub ft_meta: FTMeta,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
pub struct FTMeta {
    pub decimals: u8,
    pub name: String,
    pub price: Option<Decimal>,
    pub symbol: String,
}

#[tracing::instrument(name = "Getting FT token inventory information for", skip_all)]
pub fn get_account_ft_inventory(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Inventory> {
    tracing::Span::current().pb_set_message(&format!("account <{account_id}>..."));
    tracing::info!(target: "near_teach_me", "Getting FT token inventory information for account <{account_id}>...");

    #[derive(Debug, Clone, serde::Deserialize)]
    struct ApiResponse {
        inventory: Inventory,
    }

    let base_url = network_config.nearblocks_url.as_ref().ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "The nearblocks_url is not configured for the network <{}>. The FT token inventory information is provided by the NearBlocks API.",
            network_config.network_name
        )
    })?;
    let url = base_url.join(&format!("v1/account/{}/inventory", account_id))?;

    tracing::info!(
        target: "near_teach_me",
        parent: &tracing::Span::none(),
        "HTTP GET {url}",
    );
    match reqwest::blocking::get(url.clone()) {
        Ok(response) => {
            tracing::info!(
                target: "near_teach_me",
                parent: &tracing::Span::none(),
                "JSON RPC Response:\n{}",
                crate::common::indent_payload(&format!("{response:#?}"))
            );
            if response.status().is_success() {
                match response.json::<ApiResponse>() {
                    Ok(data) => Ok(data.inventory),
                    Err(err) => Err(color_eyre::eyre::eyre!(
                        "Failed to parse JSON response from nearblocks.io API: {}",
                        err
                    )),
                }
            } else {
                Err(color_eyre::eyre::eyre!(
                    "HTTP error from nearblocks.io API: {} - {}",
                    response.status(),
                    response
                        .text()
                        .unwrap_or_else(|_| "Unable to read response body".to_string())
                ))
            }
        }
        Err(err) => Err(color_eyre::eyre::eyre!(
            "Failed to get response from nearblocks.io API: {err}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::str::FromStr;
    use std::thread;

    use crate::config::NetworkConfig;
    use crate::types::account_id::AccountId;

    fn spawn_mock_nearblocks_server() -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = [0u8; 1024];
            let _ = stream.read(&mut buffer).unwrap();

            let body = r#"{"inventory": {
                "fts": [
                {
                    "contract": "wrap.near",
                    "amount": "5000000000000000000000000",
                    "ft_meta": {
                        "name": "Wrapped NEAR fungible token",
                        "symbol": "wNEAR",
                        "decimals": 24,
                        "icon": null,
                        "reference": null,
                        "price": 2.07
                    }
                },
                {
                    "contract": "17208628f84f5d6ad33f0da3bbbeb27ffcb398eac501a31bd6ad2011e36133a1",
                    "amount": "100000000",
                    "ft_meta": {
                        "name": "USDC",
                        "symbol": "USDC",
                        "decimals": 6,
                        "icon": "data:image/svg+xml,%3C%3Fxml version=%221.0%22 encoding=%22utf-8%22%3F%3E%3C!-- Generator: Adobe Illustrator 22.0.1, SVG Export Plug-In . SVG Version: 6.00 Build 0) --%3E%3Csvg version=%221.1%22 id=%22Layer_1%22 xmlns=%22http://www.w3.org/2000/svg%22 xmlns:xlink=%22http://www.w3.org/1999/xlink%22 x=%220px%22 y=%220px%22 viewBox=%220 0 256 256%22 style=%22enable-background:new 0 0 256 256;%22 xml:space=%22preserve%22%3E%3Cstyle type=%22text/css%22%3E .st0%7Bfill:%232775CA;%7D .st1%7Bfill:%23FFFFFF;%7D%0A%3C/style%3E%3Ccircle class=%22st0%22 cx=%22128%22 cy=%22128%22 r=%22128%22/%3E%3Cpath class=%22st1%22 d=%22M104,217c0,3-2.4,4.7-5.2,3.8C60,208.4,32,172.2,32,129.3c0-42.8,28-79.1,66.8-91.5c2.9-0.9,5.2,0.8,5.2,3.8 v7.5c0,2-1.5,4.3-3.4,5C69.9,65.4,48,94.9,48,129.3c0,34.5,21.9,63.9,52.6,75.1c1.9,0.7,3.4,3,3.4,5V217z%22/%3E%3Cpath class=%22st1%22 d=%22M136,189.3c0,2.2-1.8,4-4,4h-8c-2.2,0-4-1.8-4-4v-12.6c-17.5-2.4-26-12.1-28.3-25.5c-0.4-2.3,1.4-4.3,3.7-4.3 h9.1c1.9,0,3.5,1.4,3.9,3.2c1.7,7.9,6.3,14,20.3,14c10.3,0,17.7-5.8,17.7-14.4c0-8.6-4.3-11.9-19.5-14.4c-22.4-3-33-9.8-33-27.3 c0-13.5,10.3-24.1,26.1-26.3V69.3c0-2.2,1.8-4,4-4h8c2.2,0,4,1.8,4,4v12.7c12.9,2.3,21.1,9.6,23.8,21.8c0.5,2.3-1.3,4.4-3.7,4.4 h-8.4c-1.8,0-3.3-1.2-3.8-2.9c-2.3-7.7-7.8-11.1-17.4-11.1c-10.6,0-16.1,5.1-16.1,12.3c0,7.6,3.1,11.4,19.4,13.7 c22,3,33.4,9.3,33.4,28c0,14.2-10.6,25.7-27.1,28.3V189.3z%22/%3E%3Cpath class=%22st1%22 d=%22M157.2,220.8c-2.9,0.9-5.2-0.8-5.2-3.8v-7.5c0-2.2,1.3-4.3,3.4-5c30.6-11.2,52.6-40.7,52.6-75.1 c0-34.5-21.9-63.9-52.6-75.1c-1.9-0.7-3.4-3-3.4-5v-7.5c0-3,2.4-4.7,5.2-3.8C196,50.2,224,86.5,224,129.3 C224,172.2,196,208.4,157.2,220.8z%22/%3E%3C/svg%3E%0A",
                        "reference": null,
                        "price": 0.999808
                    }
                },
                {
                    "contract": "usdt.tether-token.near",
                    "amount": "90000000",
                    "ft_meta": {
                        "name": "Tether USD",
                        "symbol": "USDt",
                        "decimals": 6,
                        "icon": "data:image/svg+xml,%3Csvg width='111' height='90' viewBox='0 0 111 90' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M24.4825 0.862305H88.0496C89.5663 0.862305 90.9675 1.64827 91.7239 2.92338L110.244 34.1419C111.204 35.7609 110.919 37.8043 109.549 39.1171L58.5729 87.9703C56.9216 89.5528 54.2652 89.5528 52.6139 87.9703L1.70699 39.1831C0.305262 37.8398 0.0427812 35.7367 1.07354 34.1077L20.8696 2.82322C21.6406 1.60483 23.0087 0.862305 24.4825 0.862305ZM79.8419 14.8003V23.5597H61.7343V29.6329C74.4518 30.2819 83.9934 32.9475 84.0642 36.1425L84.0638 42.803C83.993 45.998 74.4518 48.6635 61.7343 49.3125V64.2168H49.7105V49.3125C36.9929 48.6635 27.4513 45.998 27.3805 42.803L27.381 36.1425C27.4517 32.9475 36.9929 30.2819 49.7105 29.6329V23.5597H31.6028V14.8003H79.8419ZM55.7224 44.7367C69.2943 44.7367 80.6382 42.4827 83.4143 39.4727C81.0601 36.9202 72.5448 34.9114 61.7343 34.3597V40.7183C59.7966 40.8172 57.7852 40.8693 55.7224 40.8693C53.6595 40.8693 51.6481 40.8172 49.7105 40.7183V34.3597C38.8999 34.9114 30.3846 36.9202 28.0304 39.4727C30.8066 42.4827 42.1504 44.7367 55.7224 44.7367Z' fill='%23009393'/%3E%3C/svg%3E",
                        "reference": null,
                        "price": 0.999079
                    }
                }
                ],
                "nfts": [
                {
                    "contract": "example-1.near",
                    "quantity": "6",
                    "nft_meta": {
                        "name": "Example-1",
                        "symbol": "EXAMPLE-1",
                        "icon": "data:image/svg+xml,%3Csvg width='89' height='87' viewBox='0 0 89 87' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M17.5427 48.1358C16.0363 48.1994 14.5323 47.9631 13.1165 47.4402C11.7006 46.9174 10.4007 46.1182 9.29096 45.0884C8.18118 44.0586 7.2833 42.8184 6.64855 41.4384C6.0138 40.0585 5.65465 38.5659 5.59156 37.0459C5.52847 35.5259 5.76267 34.0083 6.28084 32.5796C6.79901 31.151 7.59098 29.8393 8.61153 28.7194C9.63208 27.5996 10.8612 26.6936 12.2288 26.0531C13.5963 25.4126 15.0755 25.0502 16.5819 24.9865C24.9751 24.6329 35.6235 28.7963 45.0454 33.5128H45.1584C45.3247 33.5017 45.4826 33.4353 45.6073 33.3239C45.732 33.2125 45.8166 33.0624 45.8476 32.8973C45.8787 32.7322 45.8544 32.5613 45.7788 32.4115C45.7032 32.2618 45.5804 32.1416 45.4298 32.0699C34.3631 26.937 21.7648 22.4372 12.0376 23.1957C10.3305 23.3283 8.66598 23.7988 7.13906 24.5805C5.61215 25.3622 4.25275 26.4397 3.13852 27.7515C2.02429 29.0633 1.17706 30.5837 0.645141 32.2259C0.113223 33.8681 -0.0929378 35.5999 0.0384375 37.3225C0.169813 39.0451 0.636138 40.7247 1.41081 42.2655C2.18547 43.8062 3.25329 45.1779 4.55332 46.3022C5.85334 47.4265 7.36013 48.2815 8.98759 48.8182C10.6151 49.3549 12.3313 49.563 14.0385 49.4304C15.6964 49.2805 17.3083 48.7998 18.7805 48.016C18.3708 48.0818 17.9574 48.1218 17.5427 48.1358Z' fill='%23D5D4D8'/%3E%3Cpath d='M70.6208 62.6276C69.1906 61.7674 67.6059 61.2014 65.9579 60.9622C66.2954 61.1347 66.6237 61.3251 66.9414 61.5326C69.4762 63.2327 71.2378 65.8793 71.8388 68.8901C72.4398 71.9009 71.8309 75.0293 70.146 77.587C68.4612 80.1448 65.8383 81.9225 62.8545 82.5289C59.8708 83.1353 56.7704 82.5209 54.2356 80.8208C47.2384 76.1328 41.0438 66.4373 36.1491 57.0271C36.0056 56.9422 35.8383 56.9077 35.6734 56.9291C35.5084 56.9504 35.3551 57.0264 35.2374 57.1451C35.1198 57.2637 35.0446 57.4184 35.0234 57.5849C35.0022 57.7514 35.0364 57.9202 35.1205 58.065C41.0947 68.7699 48.6853 79.8968 56.9655 85.0525C58.4248 85.9573 60.0463 86.5631 61.7376 86.8355C63.4289 87.1079 65.1567 87.0415 66.8226 86.6401C68.4884 86.2386 70.0596 85.51 71.4464 84.4959C72.8332 83.4818 74.0084 82.2019 74.905 80.7295C75.8016 79.2571 76.4021 77.6208 76.672 75.9143C76.9419 74.2077 76.8761 72.4641 76.4783 70.7832C76.0805 69.1023 75.3584 67.5169 74.3534 66.1175C73.3484 64.7182 72.08 63.5323 70.6208 62.6276Z' fill='%23D5D4D8'/%3E%3Cpath d='M85.8925 28.0491C83.6519 25.3945 80.4581 23.7464 77.0135 23.4673C73.5688 23.1881 70.1553 24.3008 67.5235 26.5606C66.3246 27.6147 65.3366 28.8904 64.6127 30.319C64.8388 30.1023 65.0705 29.8913 65.3192 29.6917C66.498 28.7232 67.8557 28.0006 69.3135 27.5659C70.7713 27.1312 72.3001 26.9929 73.8113 27.1592C75.3224 27.3255 76.7859 27.7929 78.1165 28.5345C79.4472 29.276 80.6187 30.2769 81.5629 31.4789C82.5072 32.681 83.2054 34.0603 83.6171 35.5369C84.0289 37.0134 84.1459 38.5578 83.9613 40.0803C83.7768 41.6028 83.2944 43.0732 82.5421 44.4061C81.7899 45.739 80.7828 46.9079 79.5792 47.8449C73.0173 53.0861 62.058 56.029 51.6922 57.8084L51.6074 57.8825C51.4778 57.9889 51.3873 58.136 51.3504 58.3005C51.3135 58.4649 51.3324 58.637 51.404 58.7893C51.4762 58.9429 51.5971 59.0678 51.7476 59.1442C51.8981 59.2207 52.0695 59.2443 52.2348 59.2114C64.1662 56.7875 76.9906 52.9664 84.4174 46.5845C87.0482 44.3235 88.6815 41.1008 88.9581 37.625C89.2348 34.1492 88.1321 30.7048 85.8925 28.0491Z' fill='%23D5D4D8'/%3E%3Cpath d='M56.649 8.35602C56.0177 6.7294 55.0717 5.24598 53.866 3.99237C52.6603 2.73876 51.2192 1.7401 49.6268 1.05467C48.0344 0.369244 46.3227 0.0107821 44.5915 0.000239517C42.8603 -0.010303 41.1443 0.327284 39.5439 0.99327C37.9434 1.65926 36.4905 2.6403 35.2699 3.87914C34.0493 5.11797 33.0856 6.58976 32.4349 8.20857C31.7842 9.82738 31.4596 11.5608 31.4802 13.3075C31.5007 15.0543 31.8659 16.7795 32.5544 18.3822C33.1795 19.8541 34.0751 21.1932 35.194 22.3288C35.047 22.0266 34.9114 21.7186 34.7927 21.3992C34.2388 19.9674 33.9729 18.4387 34.0104 16.9022C34.048 15.3657 34.3881 13.8521 35.0112 12.4496C35.6342 11.047 36.5277 9.78363 37.6394 8.73301C38.7512 7.68238 40.0591 6.86554 41.4868 6.33006C42.9146 5.79458 44.4337 5.55116 45.9556 5.61402C47.4776 5.67688 48.9719 6.04475 50.3515 6.69618C51.7311 7.34761 52.9684 8.26957 53.9914 9.40836C55.0144 10.5472 55.8025 11.88 56.3099 13.3292C59.2207 21.2395 58.599 32.6858 57.0842 43.1569C57.0842 43.2139 57.1351 43.271 57.1577 43.3337C57.2187 43.4914 57.3302 43.624 57.4746 43.7103C57.6189 43.7966 57.7876 43.8318 57.954 43.8101C58.1204 43.7885 58.2748 43.7113 58.3927 43.5909C58.5106 43.4704 58.5852 43.3136 58.6046 43.1455C60.0063 30.9406 60.368 17.4526 56.649 8.35602Z' fill='%23D5D4D8'/%3E%3Cpath d='M37.6695 71.65C37.6148 72.0889 37.5298 72.5234 37.4152 72.9503C36.5737 75.8831 34.6186 78.362 31.9753 79.8479C29.3319 81.3338 26.2141 81.7065 23.2999 80.8849C20.3856 80.0633 17.9108 78.1139 16.4135 75.4606C14.9162 72.8074 14.5177 69.6649 15.3045 66.7168C17.5653 58.5327 24.8168 49.573 32.1984 41.9706C32.2366 41.8076 32.2203 41.6364 32.1519 41.4837C32.0835 41.331 31.967 41.2054 31.8205 41.1266C31.6739 41.0478 31.5057 41.0202 31.342 41.048C31.1782 41.0759 31.0282 41.1576 30.9154 41.2805C22.6748 50.3258 14.5245 61.0193 12.2298 70.5892C11.8279 72.2676 11.7575 74.0095 12.0227 75.7153C12.288 77.4212 12.8835 79.0576 13.7755 80.5312C14.6675 82.0048 15.8383 83.2867 17.2213 84.3036C18.6042 85.3206 20.1721 86.0528 21.8354 86.4584C23.4988 86.8639 25.225 86.9349 26.9155 86.6673C28.6061 86.3997 30.2278 85.7987 31.6882 84.8987C33.1485 83.9986 34.4189 82.8172 35.4268 81.4217C36.4346 80.0263 37.1602 78.4442 37.5621 76.7658C37.9426 75.0857 37.9792 73.3449 37.6695 71.65Z' fill='%23D5D4D8'/%3E%3C/svg%3E%0A",
                        "reference": null
                    }
                },
                {
                    "contract": "example-2.near",
                    "quantity": "2",
                    "nft_meta": {
                        "name": "Example-2",
                        "symbol": "EXAMPLE-2",
                        "icon": "XXX",
                        "reference": null
                    }
                }
                ]
            }
            }"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
        });

        (format!("http://127.0.0.1:{}/", addr.port()), handle)
    }

    fn network_config(base_url: url::Url) -> NetworkConfig {
        NetworkConfig {
            network_name: "mainnet".to_string(),
            rpc_url: "http://127.0.0.1:1/".parse().unwrap(),
            rpc_api_key: None,
            wallet_url: "http://127.0.0.1:2/".parse().unwrap(),
            explorer_transaction_url: "http://127.0.0.1:3/".parse().unwrap(),
            linkdrop_account_id: None,
            near_social_db_contract_account_id: None,
            faucet_url: None,
            meta_transaction_relayer_url: None,
            fastnear_url: None,
            staking_pools_factory_account_id: None,
            coingecko_url: None,
            mpc_contract_account_id: None,
            tx_wait_until: None,
            nearblocks_url: Some(base_url),
        }
    }

    #[test]
    fn all_contracts_to_string() {
        assert_eq!("all".to_string(), FTContract::AllContracts.to_string());
    }
    #[test]
    fn ft_contract_from_str_all() {
        let ft_contract: FTContract = "all".parse().unwrap();
        assert_eq!(ft_contract, FTContract::AllContracts);
    }
    #[test]
    fn ft_contract_wnear_to_string() {
        let ft_contract = AccountId::from_str("wrap.near").unwrap();
        assert_eq!(
            "wrap.near".to_string(),
            FTContract::SingleContract(ft_contract).to_string()
        );
    }
    #[test]
    fn ft_contract_wnear_from_str() {
        let ft_contract: FTContract = "wrap.near".parse().unwrap();
        let account_id = AccountId::from_str("wrap.near").unwrap();
        assert_eq!(ft_contract, FTContract::SingleContract(account_id));
    }
    #[test]
    fn get_account_ft_nft_token_inventory_parses_mocked_response() {
        let (base_url, server_handle) = spawn_mock_nearblocks_server();

        let network_config = network_config(base_url.parse().unwrap());
        let account_id = near_primitives::types::AccountId::from_str("test.near").unwrap();
        let inventory = get_account_ft_inventory(&network_config, &account_id).unwrap();

        assert_eq!(inventory.fts().len(), 3);
        let fts = inventory.fts();
        let first = fts.first().unwrap();
        assert_eq!(first.ft_contract_account_id.to_string(), "wrap.near");
        assert_eq!(first.amount, "5000000000000000000000000");
        assert_eq!(first.ft_meta.symbol, "wNEAR");

        server_handle.join().unwrap();
    }
}
