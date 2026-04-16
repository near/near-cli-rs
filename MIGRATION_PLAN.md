# near-cli-rs → near-kit Migration Plan

**Branch:** `migrate/near-kit`
**Strategy:** single mega-PR, 7 commits for narrative, squash on merge
**Owner:** @r-near

---

## Goal

Replace direct use of `near-primitives`, `near-jsonrpc-client`, `near-jsonrpc-primitives`, `near-crypto`, and `near-parameters` in near-cli-rs with `near-kit`'s higher-level API. End state: near-cli-rs depends on near-kit for all blockchain interaction, dropping the raw primitive crates entirely where possible.

## Non-goals

- Rewriting CLI command structure / UX
- Changing `interactive-clap` derive patterns
- Touching keyring / legacy-credentials formats
- Changing SocialDB, Ledger HID, or contract-verify integrations beyond what type-migration requires
- Adding new features

## Success criteria

1. `cargo tree -e normal | grep -E '^(├── |└── )near-(primitives|jsonrpc-client|jsonrpc-primitives|crypto|parameters)'` returns nothing at the top level (may still appear transitively via near-kit)
2. All existing CI passes
3. Measurable LOC reduction, primarily in `src/common.rs` and `src/types/`
4. `cargo tree` materially shorter — capture a before/after diff in PR

---

## 🔴 Critical gate before any real work

### G1. Transaction borsh hash compatibility

near-cli-rs today signs via `borsh::to_vec(&Transaction::V0(inner))` (enum-wrapped, emits a 0x00 discriminant byte + struct fields). near-kit's `Transaction` is a flat struct with derived `BorshSerialize` and no `TransactionV0` wrapper (`/home/ricky/near-kit-rs/crates/near-kit/src/types/transaction.rs:8-22`).

The two *may or may not* produce identical bytes. near-kit itself works end-to-end against sandbox in CI, so its format is *valid on-chain*. But "valid on-chain" does not imply "byte-identical to Ledger firmware's expected format" or "byte-identical to what the MPC contract was designed for."

**Mandatory first-commit test** — before any migration code lands, add a test under `src/transaction_signature_options/tests.rs` (or similar) that:

1. Constructs a fixed-fields transaction (known signer, receiver, nonce, block_hash, single Transfer action)
2. Builds both `near_primitives::transaction::Transaction::V0(TransactionV0 { ... })` and `near_kit::Transaction { ... }`
3. Asserts `borsh::to_vec(&near_primitives_tx) == borsh::to_vec(&near_kit_tx)`
4. Asserts `near_primitives_tx.get_hash() == near_kit_tx.get_hash()`

**If the test fails**, migration cannot proceed without upstream change to near-kit to add a V0-discriminant-preserving serialization. Options in that case:
- (a) Upstream: add `Transaction::V0(TransactionV0)` enum to near-kit with custom BorshSerialize that matches near-primitives byte-for-byte
- (b) Keep `near_primitives::transaction::Transaction` specifically for the signing-hash computation in near-cli-rs, migrate everything else
- (c) Abort migration

We do this **first**, and it's the go/no-go signal.

---

## Blocker investigation results (all resolvable)

| Blocker | Status | Notes |
|---|---|---|
| Validator RPC + EpochReference | ✅ Covered | `near.rpc().validators(Some(BlockReference::Height(h)))` for historical, `near.validators()` or `rpc.validators(None)` for latest. `EpochValidatorInfo`, `CurrentEpochValidatorInfo`, `ValidatorStakeView`, `ValidatorKickoutView` all present in near-kit. **Decision per user: use direct rpc call, skip builder.** |
| External signing (Ledger, MPC) | ✅ Covered (pending G1) | `Transaction::complete(signature)`, public Transaction fields, `Signature::Secp256k1([u8; 65])`, `rpc.send_tx()`, `rpc.view_access_key()` all present. |
| `near_primitives::views::*` types | ✅ ~95% covered | AccountView, AccessKey*View, FinalExecutionStatus, all Action variants, TxExecutionError taxonomy — present with minor renames. |
| Generic `QueryRequest` enum | ✅ Not needed | **User confirmed**: direction is away from generic QueryRequest, toward `EXPERIMENTAL_*` endpoints. near-kit has an escape hatch: `rpc.call<P, R>(method, params)`. |
| `view_state` RPC | ✅ Via escape hatch | **User decision**: use `near.rpc().call("EXPERIMENTAL_view_state", params)`. No upstream change needed. |
| `view_code` RPC | ✅ Via escape hatch | Same: `rpc.call("EXPERIMENTAL_view_code", params)` |

---

## Dependency plan

### Drop from workspace Cargo.toml

| Crate | Use sites | Replacement |
|---|---|---|
| `near-primitives` | ~140 files | near-kit re-exports / types module |
| `near-crypto` | ~5 files | `near_kit::{PublicKey, SecretKey, Signature}` |
| `near-jsonrpc-client` | ~30 files | `near_kit::Near` + `near.rpc()` for low-level |
| `near-jsonrpc-primitives` | small | near-kit error + response types |
| `near-parameters` | minimal | Grep suggests this may already be unused — verify |

### Keep (with rationale)

| Crate | Why |
|---|---|
| `near-verify-rs` | Separate contract-build verification tool |
| `near-socialdb-client` | Separate crate, not absorbed into near-kit |
| `near-ledger` | HID protocol for hardware wallet — orthogonal to signing |
| `near-abi` | Contract ABI inspection |
| `near-token`, `near-gas` | near-kit re-exports these same crates; keep direct dep for `interactive-clap` features |

---

## `NetworkConfig` decision (was Q3)

**Investigation**: `NetworkConfig` at `src/config/mod.rs:233-251` holds **15 fields**, 5 of which concern RPC (url, api key, tx_wait_until, fastnear_url, chain concerns). The remaining **10 are CLI-ecosystem concerns** near-kit doesn't touch:

1. `wallet_url` — MyNearWallet redirect base
2. `explorer_transaction_url` — explorer link prefix
3. `faucet_url` — testnet funding service
4. `linkdrop_account_id` — account creation helper
5. `near_social_db_contract_account_id` — social.near / v1.social08.testnet
6. `coingecko_url` — USD price lookups in `display_account_info`
7. `meta_transaction_relayer_url` — meta-tx relayer
8. `fastnear_url` — FastNEAR indexer API
9. `staking_pools_factory_account_id` — staking factory
10. `mpc_contract_account_id` — MPC signer contract

near-kit's `Near::custom(url)` handles URL/retry config but **does not currently support custom HTTP headers**, which means `rpc_api_key` header injection needs either (a) a tiny upstream near-kit addition for custom headers, or (b) a thin wrapper on our side using `reqwest::Client` directly and passing through.

**Decision: thin `NetworkConfig`.** Keep it as a pure data container for the 10 CLI-concerns above. Drop `json_rpc_client()` and all RPC plumbing from it. Construct `Near` fresh at call sites via `Near::custom(url).retry_config(...).build()` (or reuse via the signer-split pattern). Rationale:

- Decouples CLI UX concerns (wallet/explorer/faucet URLs, account IDs) from RPC client lifecycle
- Lets near-kit's retry/nonce logic be single source of truth — no double-layer retries
- Config file format on disk stays the same; only the Rust struct shrinks

`rpc_api_key` handling: for the first PR, accept a near-kit limitation. If Lava/FastNEAR require headers, add a `custom_headers` option to near-kit's builder as a **prerequisite sibling PR** (small, ~10 LOC), then land this migration.

---

## Command-family migration order

Per-directory analysis ranks the work from simplest → hardest. Migrate in this order within the PR so each phase has a stable foundation:

1. **config/** (LOW) — pure local JSON, no RPC calls. Trivial type-path updates.
2. **extensions/**, **extensions_commands/** (LOW) — self-update via GitHub API, no NEAR RPC.
3. **tokens/** (LOW) — FT/NFT = standard contract view/call patterns. `near.ft(...)` helpers in near-kit map cleanly.
4. **message/** (MEDIUM) — NEP-413 off-chain signing. No RPC. Six signer variants (same as account/get_public_key).
5. **contract/** (MEDIUM) — view/call queries direct. ABI needs zstd decompression after `view<Vec<u8>>`. `view_storage`, `download_wasm`, `inspect` use `EXPERIMENTAL_view_state` / `EXPERIMENTAL_view_code` via escape hatch.
6. **account/** (MEDIUM-HIGH) — 11 subcommands, heavy Action construction, 6 key-derivation variants in `get_public_key`.
7. **staking/** (MEDIUM-HIGH) — validator RPC plus concurrent pool state queries (10-way async fan-out in `get_validator_list`).
8. **transaction/** (HIGH) — retry loops in `send/mod.rs:130-167`, offline signing, meta-tx delegate actions, TransactionV0 serialization. Do last.

---

## Structured phases within one PR

Each phase is one commit on `migrate/near-kit`. All ship together; squash on merge.

### Phase 1 — Plumbing + golden-hash test (the gate)

**Commit:** `chore: add near-kit dependency and transaction-hash compat test`

- Add `near-kit = { version = "0.8", features = ["interactive-clap"] }` to `Cargo.toml`
- Write **G1 test** (see Critical gate section) — must pass before proceeding
- No behavior change in CLI itself

### Phase 2 — `src/types/` wrappers

**Commit:** `refactor: migrate src/types/ wrappers to near-kit`

Files in `src/types/`:
- **Delete outright / become type re-exports** (near-kit re-exports these):
  - `public_key.rs` (33 LOC) → use `near_kit::PublicKey`
  - `secret_key.rs` (27 LOC) → use `near_kit::SecretKey`
  - `signature.rs` (23 LOC) → use `near_kit::Signature`
  - `crypto_hash.rs` → use `near_kit::CryptoHash`
  - `near_token.rs` (141 LOC) → shim around `near_kit::NearToken`
  - `transaction.rs` (66 LOC), `signed_transaction.rs` (45 LOC) → use `near_kit::{Transaction, SignedTransaction}` (gated on G1)
- **Keep but retarget inner types:**
  - `account_id.rs`, `signed_delegate_action.rs`, `tx_execution_status.rs`, `api_key.rs`
- **Keep as-is** (CLI-specific, no NEAR types):
  - `base64_bytes.rs`, `file_bytes.rs`, `ft_properties.rs`, `json.rs`, `nonce32_bytes.rs`, `partial_protocol_config.rs`, `path_buf.rs`, `public_key_list.rs`, `slip10.rs`, `url.rs`, `vec_string.rs`, `near_allowance.rs`, `contract_properties.rs`

### Phase 3 — RPC layer: `JsonRpcClientExt` removal + thin `NetworkConfig`

**Commit:** `refactor: replace JsonRpcClientExt with near-kit Near client`

The heart of the migration. Touches `common.rs` lines 2627-2917 (~290 LOC).

Per `common.rs` section breakdown:
- **DELETE (~720 LOC total)** — either no-longer-needed or replaced by near-kit's built-in Display impls:
  - `JsonRpcClientExt` trait + impl (2627-2917, 290 LOC) — delete entirely, replace call sites with direct `near.rpc().*` or high-level `near.*`
  - `rpc_transaction_error` (1130-1194, 65 LOC) — replace call sites with `eprintln!("{:#}", err)` using near-kit's error Display impls
  - `convert_action_error_to_cli_result` (1196-1392, 197 LOC) — same, near-kit has comprehensive Display on ActionErrorKind (28+ arms with context-rich messages)
  - `convert_invalid_tx_error_to_cli_result` (1394-1565, 172 LOC) — same, near-kit's InvalidTxError / InvalidAccessKeyError / ActionsValidationError / StorageError all have Display impls
  - `BlockHashAsBase58` wrapper (62-84, 23 LOC) — near-kit's `CryptoHash::Display` already base58-encodes
- **RETARGET (~770 LOC)**:
  - `AccountTransferAllowance` (133-247, 115 LOC), `verify_account_access_key` (251-334), `is_account_exist` (337-365), `find_network_where_account_exist` (368-444), `get_account_state` (463-530), `view_account` (533-606), `print_transaction_status` (1614-1773, 160 LOC — slim to ~80 LOC using near-kit accessors `is_success()` / `failure_message()` / `total_gas_used()` / `.logs`), `RpcQueryResponseExt` (3048-3101, 54 LOC)
- Thin `NetworkConfig`: drop `json_rpc_client()` method, drop `rpc_url`/`rpc_api_key`/`tx_wait_until` wiring to RPC construction, instead add a helper `fn client(&self) -> Near` that builds fresh

**Net LOC impact**: `common.rs` currently 3458 LOC. After migration, estimate ~2350-2500 LOC (net delete of ~1000 LOC). The file stops being an RPC shim and an error-formatting hall-of-mirrors.

### Phase 4 — Action/transaction builders (in command modules)

**Commit:** `refactor: use near-kit Action and Transaction types in commands`

- Swap `near_primitives::transaction::{Transaction, TransactionV0, Action}` → near-kit equivalents everywhere in `src/commands/`
- Touch: `tokens/` (trivial), `account/` (AddKey/DeleteKey/DeleteAccount Actions), `contract/` (DeployContract/FunctionCall Actions), `transaction/construct_transaction/` (all Action variants)
- `reconstruct_transaction/mod.rs` (27 near-primitives uses) — field-by-field port
- Message signing in `message/sign_nep413/` retargets to `near_kit::Signature` and hash helpers

### Phase 5 — Views migration (CLI display retargeting)

**Commit:** `refactor: migrate CLI display functions to near-kit view types`

Error-formatting functions are being deleted outright in Phase 3 (not ported). This phase is only about retargeting CLI display code:

- `display_account_info` (2268-2390, 123 LOC) — keep, swap inner types from `near_primitives::views::AccountView` → `near_kit::AccountView`
- `display_account_profile` (2392-2408, 17 LOC) — keep, type swap
- `display_access_key_list` (2511-2575, 65 LOC) — keep, swap to `near_kit::AccessKeyInfoView`
- `print_unsigned_transaction` (775-1001, 227 LOC) — keep, exhaustive match swapped to `near_kit::Action` variants. Port one variant at a time.
- `print_full_signed_transaction` / `print_full_unsigned_transaction` (734-773, 40 LOC) — keep, type swap
- `print_value_successful_transaction` (1002-1128, 127 LOC) — keep, swap `ActionView` / `ExecutionStatus` types

These ~600 LOC are genuine CLI UX (prettytable rendering + OutputFormat plaintext/json toggle + Coingecko USD conversion) and stay in the CLI layer — they don't belong in near-kit and near-kit doesn't expose Display impls for these types by design.

**Risk**: near-kit's view types may have slight field renames. Compile errors will surface these; resolve one by one.

### Phase 6 — Staking + signing modules

**Commit:** `refactor: migrate staking, MPC, and Ledger signing flows`

- `src/commands/staking/delegate/view_balance.rs` → `near.rpc().validators(Some(BlockReference))` for historical, direct call
- `common.rs:1950-2266` staking RPC helpers → near-kit equivalents
- `transaction_signature_options/sign_with_*/mod.rs` (7 modules):
  - **SIMPLE** (2 modules): `sign_with_legacy_keychain`, `sign_with_access_key_file` — pure type swaps, JSON file format unchanged
  - **MODERATE** (3 modules): `sign_with_seed_phrase`, `sign_with_private_key`, `sign_with_keychain` — type swaps + verify `get_hash_and_size()` method exists on near-kit's `Transaction` (it does: line 51 of near-kit's `transaction.rs`)
  - **RISKY** (2 modules): `sign_with_ledger`, `sign_with_mpc` — depends on G1 outcome. `sign_with_ledger` sends full borsh TX bytes to device (lines 320-337 HID, 583-600 BLE). `sign_with_mpc` uses `CryptoHash::hash_borsh(&tx)` (line 414). Both will break silently (signatures invalid on-chain) if borsh output differs.
- Replace all uses of `Signature::from_parts(KeyType::ED25519, ...)` from near-crypto with `near_kit::Signature::ed25519_from_bytes()` / `secp256k1_from_bytes()`

### Phase 7 — Drop deprecated crates + `EXPERIMENTAL_*` escape hatches

**Commit:** `chore: remove near-primitives and friends from workspace`

- Remove `near-primitives`, `near-crypto`, `near-jsonrpc-client`, `near-jsonrpc-primitives`, `near-parameters` from root `Cargo.toml`
- Resolve final compile errors (typically residual imports missed in earlier phases)
- Add `EXPERIMENTAL_view_state` / `EXPERIMENTAL_view_code` / `EXPERIMENTAL_protocol_config` call sites via `near.rpc().call(...)` escape hatch:
  - `src/common.rs:2133` — ViewState
  - `src/commands/contract/verify/mod.rs:302` — ViewCode
  - `src/commands/contract/download_wasm/mod.rs:335, 342, 346` — ViewCode / ViewGlobalContractCode*
  - `src/commands/contract/inspect/mod.rs:84` — ViewCode
  - `src/commands/contract/view_storage/output_format/mod.rs:36` — ViewState
  - `src/types/partial_protocol_config.rs:24` — already uses EXPERIMENTAL_protocol_config string; just route through near-kit's escape hatch
- Paste before/after `cargo tree` output in PR description

---

## Risks & mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| G1 test fails (borsh hash mismatch) | Unknown | Gate — if it fails, open upstream PR to near-kit adding `Transaction::V0` compat enum before continuing |
| Ledger firmware rejects new borsh bytes | Medium (conditional on G1) | Golden-hash test is necessary but not sufficient — also need a manual Ledger smoke test before merge |
| Error-variant remapping misses a variant | Medium | Exhaustive match arms will fail to compile — clippy + `-D warnings` catches this |
| RPC API key header gap blocks Lava/FastNEAR users | Low (users often use default RPC) | If user-impacting, ship prerequisite near-kit PR adding custom headers before this lands |
| `reconstruct_transaction` field-by-field port drift | Low-Medium | Write a reconstruction roundtrip test: parse a known base64 tx, reconstruct, re-serialize, assert bytes equal |
| Meta-transaction delegate signing | Medium | near-kit supports `DelegateAction`; verify in a focused test before Phase 6 |
| Concurrent staking pool queries (10-way async fan-out) | Low | Works identically under near-kit — it uses tokio internally |

---

## Verification checklist

Before PR opens:
- [ ] G1 test passes (borsh byte equality)
- [ ] `cargo build` clean
- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo test` passes
- [ ] `grep -r 'near_primitives\|near_crypto\|near_jsonrpc_client\|near_jsonrpc_primitives\|near_parameters' src/` returns only documented hold-outs
- [ ] `cargo tree` before/after pasted in PR description

Before merge:
- [ ] Manual smoke tests on testnet:
  - [ ] `near account view-account-summary <id> network-config testnet`
  - [ ] `near tokens send-near <from> <to> '1 NEAR' network-config testnet sign-with-seed-phrase` → verify success on explorer
  - [ ] `near contract call-function as-transaction <contract> <method> ...` → verify on explorer
  - [ ] If possible: Ledger signing smoke test with real device
  - [ ] `near validator list` (testnet)

---

## Additional deletions (beyond common.rs)

### src/types/ consolidation (~230 LOC)

- **DELETE outright**:
  - `contract_properties.rs` (36 LOC) — feature-gated, minimal usage, inline into call sites
  - `slip10.rs` (53 LOC) — near-kit's `hd.rs` has `parse_hd_path()` + `derive_ed25519_slip10()`
  - `partial_protocol_config.rs` (32 LOC) — 4 matches, all internal (confirmed zero external callers)
- **REPLACE with direct re-exports** (pure pass-through wrappers):
  - `api_key.rs` (47 LOC)
  - `crypto_hash.rs` (28 LOC)
  - `url.rs` (33 LOC)

### Redundant plumbing (~255 LOC — free with migration)

- Retry loops in `transaction_signature_options/send/mod.rs:127-167` (40 LOC) and `common.rs` `get_account_state` (50 LOC) → near-kit's `RetryConfig`
- Manual `nonce + 1` boilerplate across 4 `sign_with_*` modules (~85 LOC) → near-kit auto-increments
- Manual block hash fetching in signing modules (~15 LOC) → near-kit auto-fetches
- Manual FT balance/metadata queries in `view_ft_balance/`, `send_ft/`, `ft_properties.rs` (~65 LOC) → `near.ft(contract).balance_of()` / `.metadata()`

### Deprecated command shims (~30 LOC) — user-approved delete

- `src/js_command_match/deprecated.rs` — `ValidatorsArgs` and `StakeArgs` redirect stubs for external `near-validator` CLI. Delete + remove the enum entries in `js_command_match/mod.rs`.

### Dead dependencies (verified zero usage)

Candidate drops from `Cargo.toml`:
- `easy-ext` — used only for `JsonRpcClientExt`, deleted with it
- `smart-default` — one use in `OutputFormat` enum; replace with `#[default]` attribute
- `shell-words` — zero grep hits
- `shellexpand` — zero grep hits
- `linked-hash-map` — zero grep hits (comments only)
- `bytesize` — zero grep hits

**Needs verification during execution** (agent flagged as dead but likely used):
- `zstd` — contract ABI decompression
- `wasmparser` — contract inspect wasm parsing
- `toml` — config file serialization

After migration, `bs58` and `hex` become transitive-only via near-kit.

### What we're keeping (with explicit rationale)

- **Legacy keychain support** (~730 LOC across 4 modules) — user-required. near-kit's `FileSigner` is not compatible (flat vs subdirectory layout, single-file vs multi-key-per-account). The signing plumbing *inside* these modules still simplifies via the Tier 2 plumbing delete.
- **CoinGecko USD conversion** (`common.rs:1568-1612`, 45 LOC) — user-useful in `print_transaction_status`.
- **`src/types/signed_delegate_action.rs`** (94 LOC) — legitimate borsh↔base64 interchange
- **`src/types/ft_properties.rs`** (351 LOC) — decimals-normalization business logic
- **Multi-network account probing** (`is_account_exist` / `find_network_where_account_exist`, ~40 LOC) — CLI-unique UX
- **`verify_account_access_key` interactive retry** (85 LOC) — has user prompts

---

## Total deletion tally

| Category | LOC | Status |
|---|---|---|
| common.rs: JsonRpcClientExt trait + impl | 290 | DELETE |
| common.rs: 3 error formatters (replaced by near-kit Display) | 434 | DELETE |
| common.rs: BlockHashAsBase58 wrapper | 23 | DELETE |
| common.rs: `print_transaction_status` slim via near-kit accessors | ~80 | REDUCE |
| src/types/: 3 files fully deletable | 121 | DELETE |
| src/types/: 3 files replace-with-re-export | ~108 | REDUCE |
| Redundant plumbing (retry/nonce/block/FT) | 255 | DELETE |
| Deprecated command shims | 30 | DELETE |
| **Net deletion** | **~1340 LOC** | — |

Plus 5-6 workspace deps dropped outright and 2-3 becoming transitive via near-kit.

---

## Concrete file inventory

### common.rs migration table (57 sections analyzed)

| Verdict | Count | LOC | Notes |
|---|---|---|---|
| DELETE | 6 | ~720 | JsonRpcClientExt trait+impl (290), 3 error formatters (434), BlockHashAsBase58 (23) — all replaced by near-kit built-ins |
| RETARGET | 11 | ~770 | Functions keep shape, internal types swap; `print_transaction_status` slims from 160 → ~80 LOC |
| KEEP | 40 | ~1890 | CLI UX: display functions, keychain, prompts, account history, plugin exec, Coingecko |
| **Total** | **57** | **3458** | Estimated post: ~2400-2500 LOC (net -1000) |

### Per-crate drop inventory

| Crate | Files affected | Last-file to migrate (blocker for drop) |
|---|---|---|
| near-primitives | ~140 | `transaction/reconstruct_transaction/mod.rs` (27 uses) |
| near-crypto | ~5 | `transaction_signature_options/sign_with_ledger/mod.rs` (signature reconstruction) |
| near-jsonrpc-client | ~30 | `common.rs` JsonRpcClientExt (290 LOC) |
| near-jsonrpc-primitives | ~2 | `common.rs` error formatting + `common.rs` RpcQueryResponseExt |
| near-parameters | 0 (suspected unused) | Verify with `cargo tree -i near-parameters` |

---

## Open questions resolved

- **Q1 (view_state)**: ✅ Use `near.rpc().call("EXPERIMENTAL_view_state", params)` escape hatch — no upstream change needed
- **Q2 (validator historical)**: ✅ Use `near.rpc().validators(Some(BlockReference))` directly, no builder
- **Q3 (NetworkConfig)**: ✅ Thin it — keep 10 CLI-specific fields, drop `json_rpc_client()`
- **Q4 (commit granularity)**: ✅ 7 commits in branch, squash on merge

- **Q5 (API key header)**: ✅ **Not load-bearing per user**. Can defer. If Lava/FastNEAR configs need it later, small upstream near-kit PR covers it.
- **Q6 (common.rs display code)**: ✅ Resolved — delete errors + BlockHashAsBase58 outright (~457 LOC), slim `print_transaction_status` via near-kit accessors (~80 LOC), keep the rest as legitimate CLI UX. Upstreaming `Display` for `Transaction`/`Action`/`AccountView` to near-kit rejected: library should stay opinion-less on rendering.
- **Q7 (legacy keychain)**: ✅ **Keep.** near-kit's `FileSigner` is not compatible (flat vs subdirectory layout, single-file vs multi-key-per-account). File I/O and discovery logic stays; signing plumbing inside still simplifies via Tier 2 delete.
- **Q8 (CoinGecko USD)**: ✅ **Keep** per user. Feature is useful in `print_transaction_status`.
- **Q9 (deprecated command shims)**: ✅ **Delete** per user. ~30 LOC from `js_command_match/deprecated.rs`.
