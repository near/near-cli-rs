use color_eyre::eyre::Context;
use serde_with::{base64::Base64, serde_as};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
pub struct StateInit {
    #[interactive_clap(subcommand)]
    state_init: StateInitModeCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = crate::GlobalContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// Provide state-init details:
pub enum StateInitModeCommand {
    #[strum_discriminants(strum(
        message = "use-global-hash       - Use a global contract code hash"
    ))]
    UseGlobalHash(StateInitWithContractHashRef),
    #[strum_discriminants(strum(
        message = "use-global-account-id - Use a global contract account ID"
    ))]
    UseGlobalAccountId(StateInitWithContractRefByAccount),
    #[strum_discriminants(strum(
        message = "from-borsh-base64     - Provide borsh serialized base64 encoded state init"
    ))]
    FromBorshBase64(StateInitFromBorshBase64),
    #[strum_discriminants(strum(
        message = "from-borsh-file       - Read borsh-serialized state init from a file"
    ))]
    FromBorshFile(StateInitFromBorshFile),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = StateInitWithContractHashRefContext)]
pub struct StateInitWithContractHashRef {
    /// Enter the global contract code hash:
    pub hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(subcommand)]
    data: Data,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = StateInitWithContractRefByAccountContext)]
pub struct StateInitWithContractRefByAccount {
    #[interactive_clap(skip_default_input_arg)]
    pub account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    data: Data,
}

impl StateInitWithContractRefByAccount {
    pub fn input_account_id(
        context: &crate::GlobalContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_non_signer_account_id_from_used_account_list(
            &context.config.credentials_home_dir,
            "Enter the global contract account ID:",
        )
    }
}

#[derive(Debug, Clone)]
pub struct StateInitModeContext {
    pub global_context: crate::GlobalContext,
    pub code: near_primitives::action::GlobalContractIdentifier,
}

#[derive(Debug, Clone)]
pub struct StateInitWithContractHashRefContext(StateInitModeContext);

impl StateInitWithContractHashRefContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<StateInitWithContractHashRef as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(StateInitModeContext {
            global_context: previous_context,
            code: near_primitives::action::GlobalContractIdentifier::CodeHash(scope.hash.into()),
        }))
    }
}

impl From<StateInitWithContractHashRefContext> for StateInitModeContext {
    fn from(item: StateInitWithContractHashRefContext) -> Self {
        item.0
    }
}

impl From<StateInitWithContractRefByAccountContext> for StateInitModeContext {
    fn from(item: StateInitWithContractRefByAccountContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone)]
pub struct StateInitWithContractRefByAccountContext(StateInitModeContext);

impl StateInitWithContractRefByAccountContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<StateInitWithContractRefByAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(StateInitModeContext {
            global_context: previous_context,
            code: near_primitives::action::GlobalContractIdentifier::AccountId(
                scope.account_id.clone().into(),
            ),
        }))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = StateInitFromBorshBase64Context)]
pub struct StateInitFromBorshBase64 {
    /// Enter the borsh-base64 encoded StateInit:
    pub state_init_base64: crate::types::base64_bytes::Base64Bytes,
    #[interactive_clap(subcommand)]
    action: StateInitAction,
}

#[derive(Debug, Clone)]
pub struct StateInitFromBorshBase64Context(StateInitDataContext);

impl StateInitFromBorshBase64Context {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<StateInitFromBorshBase64 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let state_init =
            crate::common::parse_borsh_base64_state_init(scope.state_init_base64.as_bytes())?;
        Ok(Self(StateInitDataContext::new(
            previous_context,
            state_init,
        )))
    }
}

impl From<StateInitFromBorshBase64Context> for StateInitDataContext {
    fn from(item: StateInitFromBorshBase64Context) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = StateInitFromBorshFileContext)]
pub struct StateInitFromBorshFile {
    /// Enter the path to a file containing borsh-serialized StateInit:
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    action: StateInitAction,
}

#[derive(Debug, Clone)]
pub struct StateInitFromBorshFileContext(StateInitDataContext);

impl StateInitFromBorshFileContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<StateInitFromBorshFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = scope.file_path.read_bytes()?;
        let state_init = crate::common::parse_borsh_base64_state_init(&data)?;
        Ok(Self(StateInitDataContext::new(
            previous_context,
            state_init,
        )))
    }
}

impl From<StateInitFromBorshFileContext> for StateInitDataContext {
    fn from(item: StateInitFromBorshFileContext) -> Self {
        item.0
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StateInitModeContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// How would you like to provide the initial state data?
pub enum Data {
    #[strum_discriminants(strum(
        message = "data-from-json - Provide base64-encoded key-value JSON inline"
    ))]
    DataFromJson(DataFromJson),
    #[strum_discriminants(strum(
        message = "data-from-file - Read base64-encoded key-value JSON data from a file"
    ))]
    DataFromFile(DataFromFile),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitModeContext)]
#[interactive_clap(output_context = DataFromFileContext)]
pub struct DataFromFile {
    /// Enter the path to the file with base64-encoded key-value JSON data:
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(subcommand)]
    action: StateInitAction,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitModeContext)]
#[interactive_clap(output_context = DataFromJsonContext)]
pub struct DataFromJson {
    /// Enter the base64-encoded key-value JSON data (e.g. '{"AAEC": "AwQF"}'):
    pub data: String,
    #[interactive_clap(subcommand)]
    action: StateInitAction,
}

#[derive(Debug, Clone)]
pub struct StateInitDataContext {
    pub global_context: crate::GlobalContext,
    pub state_init: near_primitives::deterministic_account_id::DeterministicAccountStateInit,
    pub receiver_account_id: near_primitives::types::AccountId,
}

impl StateInitDataContext {
    fn new(
        global_context: crate::GlobalContext,
        state_init: near_primitives::deterministic_account_id::DeterministicAccountStateInit,
    ) -> Self {
        let receiver_account_id =
            near_primitives::utils::derive_near_deterministic_account_id(&state_init);
        Self {
            global_context,
            state_init,
            receiver_account_id,
        }
    }
}

impl From<DataFromFileContext> for StateInitDataContext {
    fn from(item: DataFromFileContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone)]
pub struct DataFromFileContext(StateInitDataContext);

impl DataFromFileContext {
    pub fn from_previous_context(
        previous_context: StateInitModeContext,
        scope: &<DataFromFile as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let json_str = std::fs::read_to_string(&scope.file_path).wrap_err_with(|| {
            format!(
                "Failed to open or read the file: {}",
                scope.file_path.0.display()
            )
        })?;
        let data = crate::common::parse_base64_kv_map(&json_str)?;
        let state_init =
            near_primitives::deterministic_account_id::DeterministicAccountStateInit::V1(
                near_primitives::deterministic_account_id::DeterministicAccountStateInitV1 {
                    code: previous_context.code,
                    data,
                },
            );
        Ok(Self(StateInitDataContext::new(
            previous_context.global_context,
            state_init,
        )))
    }
}

impl From<DataFromJsonContext> for StateInitDataContext {
    fn from(item: DataFromJsonContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone)]
pub struct DataFromJsonContext(StateInitDataContext);

impl DataFromJsonContext {
    pub fn from_previous_context(
        previous_context: StateInitModeContext,
        scope: &<DataFromJson as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = crate::common::parse_base64_kv_map(&scope.data)?;
        let state_init =
            near_primitives::deterministic_account_id::DeterministicAccountStateInit::V1(
                near_primitives::deterministic_account_id::DeterministicAccountStateInitV1 {
                    code: previous_context.code,
                    data,
                },
            );
        Ok(Self(StateInitDataContext::new(
            previous_context.global_context,
            state_init,
        )))
    }
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StateInitDataContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What would you like to do with the state-init?
pub enum StateInitAction {
    #[strum_discriminants(strum(
        message = "execute - Submit a transaction to initialize the deterministic account"
    ))]
    /// Submit a transaction to initialize the deterministic account
    Execute(Deposit),
    #[strum_discriminants(strum(message = "inspect - Inspect state-init details"))]
    /// Inspect state-init details
    Inspect(Inspect),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StateInitDataContext)]
pub struct Inspect {
    #[interactive_clap(subcommand)]
    action: InspectAction,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StateInitDataContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
#[non_exhaustive]
/// What would you like to inspect?
pub enum InspectAction {
    #[strum_discriminants(strum(
        message = "account-id - Inspect the derived deterministic account ID"
    ))]
    /// Inspect the derived deterministic account ID
    AccountId(InspectAccountId),
    #[strum_discriminants(strum(message = "state-init - Inspect the state-init"))]
    /// Inspect the state-init
    StateInit(InspectStateInit),
    #[strum_discriminants(strum(message = "kv-map     - Inspect the key-value data map"))]
    /// Inspect the key-value data map
    KvMap(InspectKvMap),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitDataContext)]
#[interactive_clap(output_context = InspectAccountIdContext)]
pub struct InspectAccountId {}

#[derive(Debug, Clone)]
pub struct InspectAccountIdContext;

impl InspectAccountIdContext {
    pub fn from_previous_context(
        previous_context: StateInitDataContext,
        _scope: &<InspectAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        println!("{}", previous_context.receiver_account_id);
        Ok(Self)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StateInitDataContext)]
pub struct InspectStateInit {
    #[interactive_clap(subcommand)]
    format: InspectStateInitFormat,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = StateInitDataContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// In which format would you like to display the state-init?
pub enum InspectStateInitFormat {
    #[strum_discriminants(strum(message = "borsh - Borsh-serialized base64"))]
    /// Borsh-serialized base64
    Borsh(InspectStateInitBorsh),
    #[strum_discriminants(strum(message = "json  - JSON-serialized"))]
    /// JSON-serialized
    Json(InspectStateInitJson),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitDataContext)]
#[interactive_clap(output_context = InspectStateInitBorshContext)]
pub struct InspectStateInitBorsh {}

#[derive(Debug, Clone)]
pub struct InspectStateInitBorshContext;

impl InspectStateInitBorshContext {
    pub fn from_previous_context(
        previous_context: StateInitDataContext,
        _scope: &<InspectStateInitBorsh as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let bytes = borsh::to_vec(&previous_context.state_init)
            .map_err(|e| color_eyre::eyre::eyre!("Failed to borsh-serialize state-init: {e}"))?;
        println!("{}", near_primitives::serialize::to_base64(&bytes));
        Ok(Self)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitDataContext)]
#[interactive_clap(output_context = InspectStateInitJsonContext)]
pub struct InspectStateInitJson {}

#[derive(Debug, Clone)]
pub struct InspectStateInitJsonContext;

impl InspectStateInitJsonContext {
    pub fn from_previous_context(
        previous_context: StateInitDataContext,
        _scope: &<InspectStateInitJson as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        println!(
            "{}",
            serde_json::to_string_pretty(&previous_context.state_init).map_err(
                |e| color_eyre::eyre::eyre!("Failed to serialize state-init to JSON: {e}")
            )?
        );
        Ok(Self)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitDataContext)]
#[interactive_clap(output_context = InspectKvMapContext)]
pub struct InspectKvMap {}

#[derive(Debug, Clone)]
pub struct InspectKvMapContext;

impl InspectKvMapContext {
    pub fn from_previous_context(
        previous_context: StateInitDataContext,
        _scope: &<InspectKvMap as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let data = match &previous_context.state_init {
            near_primitives::deterministic_account_id::DeterministicAccountStateInit::V1(v1) => {
                &v1.data
            }
        };

        #[serde_as]
        #[derive(serde::Serialize)]
        struct Data(
            #[serde_as(as = "std::collections::BTreeMap<Base64, Base64>")]
            std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
        );

        let data = Data(data.clone());
        println!(
            "{}",
            serde_json::to_string(&data).expect("Data should be serializable")
        );
        Ok(Self)
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitDataContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    pub deposit: crate::types::near_token::NearToken,
    #[interactive_clap(named_arg)]
    /// What is the signer account ID?
    sign_as: SignerAccountId,
}

impl Deposit {
    pub fn input_deposit(
        _context: &StateInitDataContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        Ok(Some(
            inquire::CustomType::new("What is the deposit for the state init call?")
                .with_starting_input("0 NEAR")
                .prompt()?,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DepositContext {
    pub global_context: crate::GlobalContext,
    pub state_init: near_primitives::deterministic_account_id::DeterministicAccountStateInit,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub deposit: near_token::NearToken,
}

impl DepositContext {
    pub fn from_previous_context(
        previous_context: StateInitDataContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            state_init: previous_context.state_init,
            receiver_account_id: previous_context.receiver_account_id,
            deposit: scope.deposit.into(),
        })
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = DepositContext)]
#[interactive_clap(output_context = SignerAccountIdContext)]
pub struct SignerAccountId {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the signer account ID?
    signer_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network_for_transaction::NetworkForTransactionArgs,
}

impl SignerAccountId {
    pub fn input_signer_account_id(
        context: &DepositContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the signer account ID?",
        )
    }
}

#[derive(Debug, Clone)]
pub struct SignerAccountIdContext {
    pub global_context: crate::GlobalContext,
    pub state_init: near_primitives::deterministic_account_id::DeterministicAccountStateInit,
    pub receiver_account_id: near_primitives::types::AccountId,
    pub deposit: near_token::NearToken,
    pub signer_account_id: near_primitives::types::AccountId,
}

impl SignerAccountIdContext {
    pub fn from_previous_context(
        previous_context: DepositContext,
        scope: &<SignerAccountId as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            state_init: previous_context.state_init,
            receiver_account_id: previous_context.receiver_account_id,
            deposit: previous_context.deposit,
            signer_account_id: scope.signer_account_id.clone().into(),
        })
    }
}

impl From<SignerAccountIdContext> for crate::commands::ActionContext {
    fn from(item: SignerAccountIdContext) -> Self {
        let signer_id = item.signer_account_id.clone();
        let receiver_id = item.receiver_account_id.clone();

        let get_prepopulated_transaction_after_getting_network_callback: crate::commands::GetPrepopulatedTransactionAfterGettingNetworkCallback =
            std::sync::Arc::new({
                move |network_config| {
                    use crate::common::JsonRpcClientExt as _;
                    let receiver_id = &item.receiver_account_id;
                    let result = network_config
                        .json_rpc_client()
                        .blocking_call_view_account(
                            receiver_id,
                            near_primitives::types::Finality::Final.into(),
                        );
                    // Best-effort check — only cancel if we positively confirm account exists.
                    // All errors (UnknownAccount, network timeout, connection refused, etc.)
                    // are treated as "proceed" to support the sign-later offline signing flow,
                    // where the network may be unreachable at transaction construction time.
                    if result.is_ok() {
                        eprintln!(
                            "\nDeterministic account <{}> already exists on <{}> network. No transaction needed.",
                            receiver_id,
                            network_config.network_name,
                        );
                        return Ok(crate::commands::PrepopulatedTransaction {
                            signer_id: item.signer_account_id.clone(),
                            receiver_id: item.receiver_account_id.clone(),
                            actions: vec![],
                        });
                    }
                    Ok(crate::commands::PrepopulatedTransaction {
                        signer_id: item.signer_account_id.clone(),
                        receiver_id: item.receiver_account_id.clone(),
                        actions: vec![
                            near_primitives::transaction::Action::DeterministicStateInit(Box::new(
                                near_primitives::action::DeterministicStateInitAction {
                                    state_init: item.state_init.clone(),
                                    deposit: item.deposit,
                                },
                            )),
                        ],
                    })
                }
            });

        Self {
            global_context: item.global_context,
            interacting_with_account_ids: vec![signer_id, receiver_id],
            get_prepopulated_transaction_after_getting_network_callback,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
            sign_as_delegate_action: false,
        }
    }
}
