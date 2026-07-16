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
    // nfts: Vec<NFTInventory>,
}

impl Inventory {
    pub fn fts(&self) -> Vec<FTInventory> {
        self.fts.clone()
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct FTInventory {
    pub amount: String,
    #[serde(rename = "contract")]
    pub ft_contract_account_id: near_primitives::types::AccountId,
    pub ft_meta: FTMeta,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct FTMeta {
    pub decimals: u8,
    pub name: String,
    pub price: Option<f64>,
    pub symbol: String,
}

#[tracing::instrument(name = "Getting FT/NFT token inventory information for", skip_all)]
pub fn get_account_ft_nft_token_inventory(
    network_config: &crate::config::NetworkConfig,
    account_id: &near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<Inventory> {
    tracing::Span::current().pb_set_message(&format!("account <{account_id}>..."));
    tracing::info!(target: "near_teach_me", "Getting FT/NFT token inventory information for account <{account_id}>...");

    #[derive(Debug, Clone, serde::Deserialize)]
    struct ApiResponse {
        inventory: Inventory,
    }

    let base_url = network_config.nearblocks_url.as_ref().ok_or_else(|| {
        color_eyre::eyre::eyre!(
            "The nearblocks_url is not configured for the network <{}>. The FT/NFT token inventory information is provided by the NearBlocks API.",
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
                    "contract": "intel.tkn.near",
                    "amount": "30000000000000000000",
                    "ft_meta": {
                        "name": "INTEAR",
                        "symbol": "INTEL",
                        "decimals": 18,
                        "icon": "data:image/jpeg;base64,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",
                        "reference": null,
                        "price": null
                    }
                },
                {
                    "contract": "001.laboratory.jumpfinance.near",
                    "amount": "1",
                    "ft_meta": {
                        "name": "001",
                        "symbol": "🎉You won 500 NEAR!🎉To claim your gain:👉www.nearraffle.com",
                        "decimals": 0,
                        "icon": "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0idXRmLTgiPz4KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDI0LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPgo8c3ZnIHZlcnNpb249IjEuMSIgaWQ9IkxheWVyXzEiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgeG1sbnM6eGxpbms9Imh0dHA6Ly93d3cudzMub3JnLzE5OTkveGxpbmsiIHg9IjBweCIgeT0iMHB4IgoJIHZpZXdCb3g9IjAgMCA5MC4xIDkwIiBzdHlsZT0iZW5hYmxlLWJhY2tncm91bmQ6bmV3IDAgMCA5MC4xIDkwOyIgeG1sOnNwYWNlPSJwcmVzZXJ2ZSI+CjxwYXRoIGQ9Ik03Mi4yLDQuNkw1My40LDMyLjVjLTEuMywxLjksMS4yLDQuMiwzLDIuNkw3NC45LDE5YzAuNS0wLjQsMS4yLTAuMSwxLjIsMC42djUwLjNjMCwwLjctMC45LDEtMS4zLDAuNWwtNTYtNjcKCUMxNywxLjIsMTQuNCwwLDExLjUsMGgtMkM0LjMsMCwwLDQuMywwLDkuNnY3MC44QzAsODUuNyw0LjMsOTAsOS42LDkwYzMuMywwLDYuNC0xLjcsOC4yLTQuNmwxOC44LTI3LjljMS4zLTEuOS0xLjItNC4yLTMtMi42CglsLTE4LjUsMTZjLTAuNSwwLjQtMS4yLDAuMS0xLjItMC42VjIwLjFjMC0wLjcsMC45LTEsMS4zLTAuNWw1Niw2N2MxLjgsMi4yLDQuNSwzLjQsNy4zLDMuNGgyYzUuMywwLDkuNi00LjMsOS42LTkuNlY5LjYKCWMwLTUuMy00LjMtOS42LTkuNi05LjZDNzcuMSwwLDc0LDEuNyw3Mi",
                        "reference": null,
                        "price": null
                    }
                }
                ],
                "nfts": [
                {
                    "contract": "proof-of-memories-nearcon-2022.snft.near",
                    "quantity": "6",
                    "nft_meta": {
                        "name": "Proof of Memories NEARCON 2022",
                        "symbol": "SNFT",
                        "icon": "data:image/svg+xml,%3Csvg width='89' height='87' viewBox='0 0 89 87' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M17.5427 48.1358C16.0363 48.1994 14.5323 47.9631 13.1165 47.4402C11.7006 46.9174 10.4007 46.1182 9.29096 45.0884C8.18118 44.0586 7.2833 42.8184 6.64855 41.4384C6.0138 40.0585 5.65465 38.5659 5.59156 37.0459C5.52847 35.5259 5.76267 34.0083 6.28084 32.5796C6.79901 31.151 7.59098 29.8393 8.61153 28.7194C9.63208 27.5996 10.8612 26.6936 12.2288 26.0531C13.5963 25.4126 15.0755 25.0502 16.5819 24.9865C24.9751 24.6329 35.6235 28.7963 45.0454 33.5128H45.1584C45.3247 33.5017 45.4826 33.4353 45.6073 33.3239C45.732 33.2125 45.8166 33.0624 45.8476 32.8973C45.8787 32.7322 45.8544 32.5613 45.7788 32.4115C45.7032 32.2618 45.5804 32.1416 45.4298 32.0699C34.3631 26.937 21.7648 22.4372 12.0376 23.1957C10.3305 23.3283 8.66598 23.7988 7.13906 24.5805C5.61215 25.3622 4.25275 26.4397 3.13852 27.7515C2.02429 29.0633 1.17706 30.5837 0.645141 32.2259C0.113223 33.8681 -0.0929378 35.5999 0.0384375 37.3225C0.169813 39.0451 0.636138 40.7247 1.41081 42.2655C2.18547 43.8062 3.25329 45.1779 4.55332 46.3022C5.85334 47.4265 7.36013 48.2815 8.98759 48.8182C10.6151 49.3549 12.3313 49.563 14.0385 49.4304C15.6964 49.2805 17.3083 48.7998 18.7805 48.016C18.3708 48.0818 17.9574 48.1218 17.5427 48.1358Z' fill='%23D5D4D8'/%3E%3Cpath d='M70.6208 62.6276C69.1906 61.7674 67.6059 61.2014 65.9579 60.9622C66.2954 61.1347 66.6237 61.3251 66.9414 61.5326C69.4762 63.2327 71.2378 65.8793 71.8388 68.8901C72.4398 71.9009 71.8309 75.0293 70.146 77.587C68.4612 80.1448 65.8383 81.9225 62.8545 82.5289C59.8708 83.1353 56.7704 82.5209 54.2356 80.8208C47.2384 76.1328 41.0438 66.4373 36.1491 57.0271C36.0056 56.9422 35.8383 56.9077 35.6734 56.9291C35.5084 56.9504 35.3551 57.0264 35.2374 57.1451C35.1198 57.2637 35.0446 57.4184 35.0234 57.5849C35.0022 57.7514 35.0364 57.9202 35.1205 58.065C41.0947 68.7699 48.6853 79.8968 56.9655 85.0525C58.4248 85.9573 60.0463 86.5631 61.7376 86.8355C63.4289 87.1079 65.1567 87.0415 66.8226 86.6401C68.4884 86.2386 70.0596 85.51 71.4464 84.4959C72.8332 83.4818 74.0084 82.2019 74.905 80.7295C75.8016 79.2571 76.4021 77.6208 76.672 75.9143C76.9419 74.2077 76.8761 72.4641 76.4783 70.7832C76.0805 69.1023 75.3584 67.5169 74.3534 66.1175C73.3484 64.7182 72.08 63.5323 70.6208 62.6276Z' fill='%23D5D4D8'/%3E%3Cpath d='M85.8925 28.0491C83.6519 25.3945 80.4581 23.7464 77.0135 23.4673C73.5688 23.1881 70.1553 24.3008 67.5235 26.5606C66.3246 27.6147 65.3366 28.8904 64.6127 30.319C64.8388 30.1023 65.0705 29.8913 65.3192 29.6917C66.498 28.7232 67.8557 28.0006 69.3135 27.5659C70.7713 27.1312 72.3001 26.9929 73.8113 27.1592C75.3224 27.3255 76.7859 27.7929 78.1165 28.5345C79.4472 29.276 80.6187 30.2769 81.5629 31.4789C82.5072 32.681 83.2054 34.0603 83.6171 35.5369C84.0289 37.0134 84.1459 38.5578 83.9613 40.0803C83.7768 41.6028 83.2944 43.0732 82.5421 44.4061C81.7899 45.739 80.7828 46.9079 79.5792 47.8449C73.0173 53.0861 62.058 56.029 51.6922 57.8084L51.6074 57.8825C51.4778 57.9889 51.3873 58.136 51.3504 58.3005C51.3135 58.4649 51.3324 58.637 51.404 58.7893C51.4762 58.9429 51.5971 59.0678 51.7476 59.1442C51.8981 59.2207 52.0695 59.2443 52.2348 59.2114C64.1662 56.7875 76.9906 52.9664 84.4174 46.5845C87.0482 44.3235 88.6815 41.1008 88.9581 37.625C89.2348 34.1492 88.1321 30.7048 85.8925 28.0491Z' fill='%23D5D4D8'/%3E%3Cpath d='M56.649 8.35602C56.0177 6.7294 55.0717 5.24598 53.866 3.99237C52.6603 2.73876 51.2192 1.7401 49.6268 1.05467C48.0344 0.369244 46.3227 0.0107821 44.5915 0.000239517C42.8603 -0.010303 41.1443 0.327284 39.5439 0.99327C37.9434 1.65926 36.4905 2.6403 35.2699 3.87914C34.0493 5.11797 33.0856 6.58976 32.4349 8.20857C31.7842 9.82738 31.4596 11.5608 31.4802 13.3075C31.5007 15.0543 31.8659 16.7795 32.5544 18.3822C33.1795 19.8541 34.0751 21.1932 35.194 22.3288C35.047 22.0266 34.9114 21.7186 34.7927 21.3992C34.2388 19.9674 33.9729 18.4387 34.0104 16.9022C34.048 15.3657 34.3881 13.8521 35.0112 12.4496C35.6342 11.047 36.5277 9.78363 37.6394 8.73301C38.7512 7.68238 40.0591 6.86554 41.4868 6.33006C42.9146 5.79458 44.4337 5.55116 45.9556 5.61402C47.4776 5.67688 48.9719 6.04475 50.3515 6.69618C51.7311 7.34761 52.9684 8.26957 53.9914 9.40836C55.0144 10.5472 55.8025 11.88 56.3099 13.3292C59.2207 21.2395 58.599 32.6858 57.0842 43.1569C57.0842 43.2139 57.1351 43.271 57.1577 43.3337C57.2187 43.4914 57.3302 43.624 57.4746 43.7103C57.6189 43.7966 57.7876 43.8318 57.954 43.8101C58.1204 43.7885 58.2748 43.7113 58.3927 43.5909C58.5106 43.4704 58.5852 43.3136 58.6046 43.1455C60.0063 30.9406 60.368 17.4526 56.649 8.35602Z' fill='%23D5D4D8'/%3E%3Cpath d='M37.6695 71.65C37.6148 72.0889 37.5298 72.5234 37.4152 72.9503C36.5737 75.8831 34.6186 78.362 31.9753 79.8479C29.3319 81.3338 26.2141 81.7065 23.2999 80.8849C20.3856 80.0633 17.9108 78.1139 16.4135 75.4606C14.9162 72.8074 14.5177 69.6649 15.3045 66.7168C17.5653 58.5327 24.8168 49.573 32.1984 41.9706C32.2366 41.8076 32.2203 41.6364 32.1519 41.4837C32.0835 41.331 31.967 41.2054 31.8205 41.1266C31.6739 41.0478 31.5057 41.0202 31.342 41.048C31.1782 41.0759 31.0282 41.1576 30.9154 41.2805C22.6748 50.3258 14.5245 61.0193 12.2298 70.5892C11.8279 72.2676 11.7575 74.0095 12.0227 75.7153C12.288 77.4212 12.8835 79.0576 13.7755 80.5312C14.6675 82.0048 15.8383 83.2867 17.2213 84.3036C18.6042 85.3206 20.1721 86.0528 21.8354 86.4584C23.4988 86.8639 25.225 86.9349 26.9155 86.6673C28.6061 86.3997 30.2278 85.7987 31.6882 84.8987C33.1485 83.9986 34.4189 82.8172 35.4268 81.4217C36.4346 80.0263 37.1602 78.4442 37.5621 76.7658C37.9426 75.0857 37.9792 73.3449 37.6695 71.65Z' fill='%23D5D4D8'/%3E%3C/svg%3E%0A",
                        "reference": null
                    }
                },
                {
                    "contract": "exxatest.near",
                    "quantity": "2",
                    "nft_meta": {
                        "name": "Exxaverse",
                        "symbol": "EXXA",
                        "icon": "TBD",
                        "reference": null
                    }
                },
                {
                    "contract": "h00kd.near",
                    "quantity": "1",
                    "nft_meta": {
                        "name": "h00kd",
                        "symbol": "h00kd",
                        "icon": "data:image/png;base64,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",
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
        let inventory = get_account_ft_nft_token_inventory(&network_config, &account_id).unwrap();

        assert_eq!(inventory.fts().len(), 3);
        let fts = inventory.fts();
        let first = fts.first().unwrap();
        assert_eq!(first.ft_contract_account_id.to_string(), "wrap.near");
        assert_eq!(first.amount, "5000000000000000000000000");
        assert_eq!(first.ft_meta.symbol, "wNEAR");

        server_handle.join().unwrap();
    }
}
