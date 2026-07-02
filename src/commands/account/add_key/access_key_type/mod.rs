use std::str::FromStr;

use inquire::{CustomType, Select, Text};

#[derive(Debug, Clone)]
pub struct AccessTypeContext {
    pub global_context: crate::GlobalContext,
    pub signer_account_id: near_primitives::types::AccountId,
    pub permission: near_primitives::account::AccessKeyPermission,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = FullAccessTypeContext)]
pub struct FullAccessType {
    #[interactive_clap(subcommand)]
    pub access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct FullAccessTypeContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
}

impl FullAccessTypeContext {
    pub fn from_previous_context(
        previous_context: super::AddKeyCommandContext,
        _scope: &<FullAccessType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id.into(),
            permission: near_primitives::account::AccessKeyPermission::FullAccess,
        })
    }
}

impl From<FullAccessTypeContext> for AccessTypeContext {
    fn from(item: FullAccessTypeContext) -> Self {
        Self {
            global_context: item.global_context,
            signer_account_id: item.signer_account_id,
            permission: item.permission,
        }
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = FunctionCallTypeContext)]
pub struct FunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: crate::types::near_allowance::NearAllowance,
    #[interactive_clap(long)]
    /// Enter the contract account ID that this access key can be used to sign call function transactions for:
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    function_names: crate::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct FunctionCallTypeContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    allowance: Option<crate::types::near_token::NearToken>,
    contract_account_id: crate::types::account_id::AccountId,
    function_names: crate::types::vec_string::VecString,
}

impl FunctionCallTypeContext {
    pub fn from_previous_context(
        previous_context: super::AddKeyCommandContext,
        scope: &<FunctionCallType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id.into(),
            allowance: scope.allowance.optional_near_token(),
            contract_account_id: scope.contract_account_id.clone(),
            function_names: scope.function_names.clone(),
        })
    }
}

impl From<FunctionCallTypeContext> for AccessTypeContext {
    fn from(item: FunctionCallTypeContext) -> Self {
        Self {
            global_context: item.global_context,
            signer_account_id: item.signer_account_id,
            permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                near_primitives::account::FunctionCallPermission {
                    allowance: item.allowance.map(Into::into),
                    receiver_id: item.contract_account_id.to_string(),
                    method_names: item.function_names.into(),
                },
            ),
        }
    }
}

impl FunctionCallType {
    pub fn input_function_names(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        #[derive(strum_macros::Display)]
        enum ConfirmOptions {
            #[strum(
                to_string = "Yes, I want to input a list of function names that can be called when transaction is signed by this access key"
            )]
            Yes,
            #[strum(to_string = "No, I allow it to call any functions on the specified contract")]
            No,
        }

        let select_choose_input = Select::new(
            "Would you like the access key to be valid exclusively for calling specific functions on the contract?",
            vec![ConfirmOptions::Yes, ConfirmOptions::No],
        )
        .prompt()?;
        if let ConfirmOptions::Yes = select_choose_input {
            let mut input_function_names = Text::new("Enter a comma-separated list of function names that will be allowed to be called in a transaction signed by this access key:")
                    .prompt()?;
            if input_function_names.contains('\"') {
                input_function_names.clear()
            };
            if input_function_names.is_empty() {
                Ok(Some(crate::types::vec_string::VecString(vec![])))
            } else {
                Ok(Some(crate::types::vec_string::VecString::from_str(
                    &input_function_names,
                )?))
            }
        } else {
            Ok(Some(crate::types::vec_string::VecString(vec![])))
        }
    }

    pub fn input_allowance(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_allowance::NearAllowance>> {
        let allowance_near_balance: crate::types::near_allowance::NearAllowance =
            CustomType::new("Enter the allowance, a budget this access key can use to pay for transaction fees (example: 10 NEAR or 0.5 NEAR or 10000 yoctonear):")
                .with_starting_input("unlimited")
                .prompt()?;
        Ok(Some(allowance_near_balance))
    }
}

/// Prompt for the number of parallel nonces of a gas key.
///
/// `interactive_clap` only implements `ToCli` for `u64`/`u128` (not `u16`), so
/// the CLI field is a `u64` and is narrowed to `NonceIndex` (`u16`) in the
/// context builder, where it is also bounded by the protocol limit
/// `AccessKeyPermission::MAX_NONCES_FOR_GAS_KEY`.
fn input_num_nonces() -> color_eyre::eyre::Result<u64> {
    let max = near_primitives::account::AccessKeyPermission::MAX_NONCES_FOR_GAS_KEY;
    let num_nonces: u64 = CustomType::new(&format!(
        "How many parallel nonces should this gas key have (1..={max})?"
    ))
    .with_starting_input("1")
    .prompt()?;
    Ok(num_nonces)
}

/// Narrow a CLI-provided `u64` nonce count to a protocol-valid `NonceIndex`.
fn validate_num_nonces(
    num_nonces: u64,
) -> color_eyre::eyre::Result<near_primitives::types::NonceIndex> {
    let max = near_primitives::account::AccessKeyPermission::MAX_NONCES_FOR_GAS_KEY;
    if num_nonces == 0 || num_nonces > u64::from(max) {
        color_eyre::eyre::bail!(
            "A gas key must have between 1 and {max} parallel nonces, got {num_nonces}"
        );
    }
    Ok(num_nonces as near_primitives::types::NonceIndex)
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = GasKeyFullAccessTypeContext)]
pub struct GasKeyFullAccessType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    num_nonces: u64,
    #[interactive_clap(subcommand)]
    pub access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct GasKeyFullAccessTypeContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
}

impl GasKeyFullAccessTypeContext {
    pub fn from_previous_context(
        previous_context: super::AddKeyCommandContext,
        scope: &<GasKeyFullAccessType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id.into(),
            // Gas keys must be created empty: the protocol rejects an AddKey that
            // sets a non-zero gas-key balance. Fund it afterwards with `fund-gas-key`.
            permission: near_primitives::account::AccessKeyPermission::GasKeyFullAccess(
                near_primitives::account::GasKeyInfo {
                    balance: near_token::NearToken::from_yoctonear(0),
                    num_nonces: validate_num_nonces(scope.num_nonces)?,
                },
            ),
        })
    }
}

impl From<GasKeyFullAccessTypeContext> for AccessTypeContext {
    fn from(item: GasKeyFullAccessTypeContext) -> Self {
        Self {
            global_context: item.global_context,
            signer_account_id: item.signer_account_id,
            permission: item.permission,
        }
    }
}

impl GasKeyFullAccessType {
    pub fn input_num_nonces(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(Some(input_num_nonces()?))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = super::AddKeyCommandContext)]
#[interactive_clap(output_context = GasKeyFunctionCallTypeContext)]
pub struct GasKeyFunctionCallType {
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    num_nonces: u64,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    allowance: crate::types::near_allowance::NearAllowance,
    #[interactive_clap(long)]
    /// Enter the contract account ID that this gas key can be used to sign call function transactions for:
    contract_account_id: crate::types::account_id::AccountId,
    #[interactive_clap(long)]
    #[interactive_clap(skip_default_input_arg)]
    function_names: crate::types::vec_string::VecString,
    #[interactive_clap(subcommand)]
    access_key_mode: super::AccessKeyMode,
}

#[derive(Debug, Clone)]
pub struct GasKeyFunctionCallTypeContext {
    global_context: crate::GlobalContext,
    signer_account_id: near_primitives::types::AccountId,
    permission: near_primitives::account::AccessKeyPermission,
}

impl GasKeyFunctionCallTypeContext {
    pub fn from_previous_context(
        previous_context: super::AddKeyCommandContext,
        scope: &<GasKeyFunctionCallType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            global_context: previous_context.global_context,
            signer_account_id: previous_context.owner_account_id.into(),
            // Gas keys must be created empty: the protocol rejects an AddKey that
            // sets a non-zero gas-key balance. Fund it afterwards with `fund-gas-key`.
            permission: near_primitives::account::AccessKeyPermission::GasKeyFunctionCall(
                near_primitives::account::GasKeyInfo {
                    balance: near_token::NearToken::from_yoctonear(0),
                    num_nonces: validate_num_nonces(scope.num_nonces)?,
                },
                near_primitives::account::FunctionCallPermission {
                    allowance: scope.allowance.optional_near_token().map(Into::into),
                    receiver_id: scope.contract_account_id.to_string(),
                    method_names: scope.function_names.clone().into(),
                },
            ),
        })
    }
}

impl From<GasKeyFunctionCallTypeContext> for AccessTypeContext {
    fn from(item: GasKeyFunctionCallTypeContext) -> Self {
        Self {
            global_context: item.global_context,
            signer_account_id: item.signer_account_id,
            permission: item.permission,
        }
    }
}

impl GasKeyFunctionCallType {
    pub fn input_num_nonces(
        _context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<u64>> {
        Ok(Some(input_num_nonces()?))
    }

    pub fn input_allowance(
        context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_allowance::NearAllowance>> {
        FunctionCallType::input_allowance(context)
    }

    pub fn input_function_names(
        context: &super::AddKeyCommandContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::vec_string::VecString>> {
        FunctionCallType::input_function_names(context)
    }
}
