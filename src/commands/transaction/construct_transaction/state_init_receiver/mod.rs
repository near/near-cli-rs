use color_eyre::eyre::Context;
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionSenderContext)]
pub struct StateInitReceiver {
    #[interactive_clap(subcommand)]
    state_init: StateInitModeCommand,
}

#[derive(Debug, EnumDiscriminants, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(context = super::ConstructTransactionSenderContext)]
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
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ConstructTransactionSenderContext)]
#[interactive_clap(output_context = StateInitWithContractHashRefContext)]
pub struct StateInitWithContractHashRef {
    /// Enter the global contract code hash:
    pub hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(subcommand)]
    data: Data,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ConstructTransactionSenderContext)]
#[interactive_clap(output_context = StateInitWithContractRefByAccountContext)]
pub struct StateInitWithContractRefByAccount {
    #[interactive_clap(skip_default_input_arg)]
    pub account_id: crate::types::account_id::AccountId,
    #[interactive_clap(subcommand)]
    data: Data,
}

impl StateInitWithContractRefByAccount {
    pub fn input_account_id(
        context: &super::ConstructTransactionSenderContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "Enter the global contract account ID: ",
        )
    }
}

#[derive(Debug, Clone)]
pub struct StateInitModeContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub code: near_primitives::action::GlobalContractIdentifier,
}

#[derive(Debug, Clone)]
pub struct StateInitWithContractHashRefContext(StateInitModeContext);

impl StateInitWithContractHashRefContext {
    pub fn from_previous_context(
        previous_context: super::ConstructTransactionSenderContext,
        scope: &<StateInitWithContractHashRef as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(StateInitModeContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
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
        previous_context: super::ConstructTransactionSenderContext,
        scope: &<StateInitWithContractRefByAccount as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(StateInitModeContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            code: near_primitives::action::GlobalContractIdentifier::AccountId(
                scope.account_id.clone().into(),
            ),
        }))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::ConstructTransactionSenderContext)]
#[interactive_clap(output_context = StateInitFromBorshBase64Context)]
pub struct StateInitFromBorshBase64 {
    /// Enter the borsh-base64 encoded StateInit:
    pub state_init_base64: crate::types::base64_bytes::Base64Bytes,
    #[interactive_clap(named_arg)]
    deposit: Deposit,
}

#[derive(Debug, Clone)]
pub struct StateInitFromBorshBase64Context(StateInitDataContext);

impl StateInitFromBorshBase64Context {
    pub fn from_previous_context(
        previous_context: super::ConstructTransactionSenderContext,
        scope: &<StateInitFromBorshBase64 as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let state_init =
            crate::common::parse_borsh_base64_state_init(scope.state_init_base64.as_bytes())?;
        let receiver_account_id =
            near_primitives::utils::derive_near_deterministic_account_id(&state_init);
        Ok(Self(StateInitDataContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            state_init,
            receiver_account_id,
        }))
    }
}

impl From<StateInitFromBorshBase64Context> for StateInitDataContext {
    fn from(item: StateInitFromBorshBase64Context) -> Self {
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
        message = "data-from-file - Read base64-encoded key-value JSON data from a file"
    ))]
    DataFromFile(DataFromFile),
    #[strum_discriminants(strum(
        message = "data-from-json - Provide base64-encoded key-value JSON data inline (e.g. '{\"AAEC\": \"AwQF\"})')"
    ))]
    DataFromJson(DataFromJson),
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitModeContext)]
#[interactive_clap(output_context = DataFromFileContext)]
pub struct DataFromFile {
    #[interactive_clap(skip_default_input_arg)]
    pub file_path: crate::types::path_buf::PathBuf,
    #[interactive_clap(named_arg)]
    deposit: Deposit,
}

impl DataFromFile {
    pub fn input_file_path(
        _context: &StateInitModeContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::path_buf::PathBuf>> {
        Ok(Some(
            inquire::CustomType::new(
                "Enter the path to the file with base64-encoded key-value JSON data:",
            )
            .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitModeContext)]
#[interactive_clap(output_context = DataFromJsonContext)]
pub struct DataFromJson {
    #[interactive_clap(skip_default_input_arg)]
    pub data: String,
    #[interactive_clap(named_arg)]
    deposit: Deposit,
}

impl DataFromJson {
    pub fn input_data(_context: &StateInitModeContext) -> color_eyre::eyre::Result<Option<String>> {
        Ok(Some(
            inquire::Text::new(
                "Enter the base64-encoded key-value JSON data (e.g. '{\"AAEC\": \"AwQF\"}'):",
            )
            .prompt()?,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct StateInitDataContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub state_init: near_primitives::deterministic_account_id::DeterministicAccountStateInit,
    pub receiver_account_id: near_primitives::types::AccountId,
}

impl StateInitDataContext {
    fn build(
        code: near_primitives::action::GlobalContractIdentifier,
        global_context: crate::GlobalContext,
        signer_account_id: near_primitives::types::AccountId,
        data: std::collections::BTreeMap<Vec<u8>, Vec<u8>>,
    ) -> color_eyre::eyre::Result<Self> {
        let state_init =
            near_primitives::deterministic_account_id::DeterministicAccountStateInit::V1(
                near_primitives::deterministic_account_id::DeterministicAccountStateInitV1 {
                    code,
                    data,
                },
            );
        let receiver_account_id =
            near_primitives::utils::derive_near_deterministic_account_id(&state_init);
        Ok(Self {
            global_context,
            signer_account_id,
            state_init,
            receiver_account_id,
        })
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
        Ok(Self(StateInitDataContext::build(
            previous_context.code,
            previous_context.global_context,
            previous_context.signer_account_id,
            data,
        )?))
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
        Ok(Self(StateInitDataContext::build(
            previous_context.code,
            previous_context.global_context,
            previous_context.signer_account_id,
            data,
        )?))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = StateInitDataContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    pub deposit: crate::types::near_token::NearToken,
    #[interactive_clap(subcommand)]
    pub next_actions: super::add_action_1::NextAction,
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
pub struct DepositContext(super::ConstructTransactionContext);

impl DepositContext {
    pub fn from_previous_context(
        previous_context: StateInitDataContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let deposit: near_token::NearToken = scope.deposit.into();
        Ok(Self(super::ConstructTransactionContext {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.signer_account_id,
            receiver_account_id: previous_context.receiver_account_id,
            actions: vec![
                near_primitives::transaction::Action::DeterministicStateInit(Box::new(
                    near_primitives::action::DeterministicStateInitAction {
                        state_init: previous_context.state_init,
                        deposit,
                    },
                )),
            ],
            sign_as_delegate_action: false,
        }))
    }
}

impl From<DepositContext> for super::ConstructTransactionContext {
    fn from(item: DepositContext) -> Self {
        item.0
    }
}
