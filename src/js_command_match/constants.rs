// Accounts and contracts
pub const ACCOUNT_ID_ALIASES: [&str; 3] = ["accountId", "account-id", "account_id"];
pub const MASTER_ACCOUNT_ALIASES: [&str; 6] = ["masterAccount", "master-account", "useAccount", "use-account", "accountId", "account_id"];
pub const CONTRACT_ID_ALIASES: [&str; 2] = ["contractId", "contract-id"];
pub const METHOD_NAMES_ALIASES: [&str; 2] = ["methodNames", "method-names"];

// Keys
pub const PUBLIC_KEY_ALIASES: [&str; 2] = ["publicKey", "public-key"];
pub const SECRET_KEY_ALIASES: [&str; 2] = ["secretKey", "secret-key"];
pub const SEED_PHRASE_ALIASES: [&str; 2] = ["seedPhrase", "seed-phrase"];
pub const SAVE_IMPLICIT_ALIASES: [&str; 2] = ["saveImplicit", "save-implicit"];
pub const DEFAULT_SEED_PHRASE_PATH: &str = "44'/397'/0'/0'/1'";

// Ledger
pub const USE_LEDGER_ALIASES: [&str; 4] = ["signWithLedger", "sign-with-ledger", "useLedgerKey", "use-ledger-key"];
pub const LEDGER_PATH_ALIASES: [&str; 2] = ["ledgerPath", "ledger-path"];

// Balance and faucet
pub const INITIAL_BALANCE_ALIASES: [&str; 2] = ["initialBalance", "initial-balance"];
pub const USE_FAUCET_ALIASES: [&str; 2] = ["useFaucet", "use-faucet"];

// SETTINGS
pub const NETWORK_ID_ALIASES: [&str; 2] = ["networkId", "network-id"];
pub const BLOCK_ID_ALIASES: [&str; 2] = ["blockId", "block_id"];

// Deploy
pub const WASM_FILE_ALIASES: [&str; 3] = ["wasm_file", "wasmFile", "wasm-file"];
pub const INIT_FUNCTION_ALIASES: [&str; 3] = ["init_function", "initFunction", "init-function"];
pub const INIT_ARGS_ALIASES: [&str; 3] = ["init_args", "initArgs", "init-args"];
pub const INIT_GAS_ALIASES: [&str; 3] = ["init_gas", "initGas", "init-gas"];
pub const INIT_DEPOSIT_ALIASES: [&str; 3] = ["init_deposit", "initDeposit", "init-deposit"];

// Login
pub const WALLET_URL_ALIASES: [&str; 3] = ["wallet_url", "walletUrl", "wallet-url"];
pub const DEFAULT_WALLET_URL: &str = "https://testnet.mynearwallet.com";