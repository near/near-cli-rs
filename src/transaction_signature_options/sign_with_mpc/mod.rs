use color_eyre::{eyre::Context, owo_colors::OwoColorize};
use inquire::CustomType;
use near_primitives::transaction::{Transaction, TransactionV0};
use strum::{EnumDiscriminants, EnumIter, EnumMessage};

use crate::common::{JsonRpcClientExt, RpcQueryResponseExt};

pub mod mpc_sign_request;
pub mod mpc_sign_result;
mod mpc_sign_with;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignMpcContext)]
pub struct SignMpc {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the Admin account address?
    admin_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(subcommand)]
    /// What is key type for deriving key?
    mpc_key_type: MpcKeyType,
}

#[derive(Clone)]
pub struct SignMpcContext {
    admin_account_id: near_primitives::types::AccountId,
    tx_context: crate::commands::TransactionContext,
}

impl SignMpcContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignMpc as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        if previous_context.global_context.offline {
            eprintln!("\nInternet connection is required to sign with MPC!");
            return Err(color_eyre::eyre::eyre!(
                "Internet connection is required to sign with MPC!"
            ));
        }

        if let Err(err) = previous_context
            .network_config
            .get_mpc_contract_account_id()
        {
            eprintln!(
                "\nCouldn't retrieve MPC contract account id from network config:\n    {err}"
            );
            return Err(color_eyre::eyre::eyre!(
                "Couldn't retrieve MPC contract account id from network config!"
            ));
        }

        Ok(SignMpcContext {
            admin_account_id: scope.admin_account_id.clone().into(),
            tx_context: previous_context,
        })
    }
}

impl SignMpc {
    pub fn input_admin_account_id(
        context: &crate::commands::TransactionContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::account_id::AccountId>> {
        crate::common::input_signer_account_id_from_used_account_list(
            &context.global_context.config.credentials_home_dir,
            "What is the Admin AccountId?",
        )
    }
}

#[derive(Debug, Clone, EnumDiscriminants, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignMpcContext)]
#[strum_discriminants(derive(EnumMessage, EnumIter))]
/// What is the MPC key type for derivation (if unsure choose Secp256K1)?
pub enum MpcKeyType {
    #[strum_discriminants(strum(message = "secp256k1 - use Secp256K1 key derivation"))]
    /// Use Secp256K1 key
    Secp256k1(MpcKeyTypeSecp),
    #[strum_discriminants(strum(message = "ed25519   - use Ed25519 key for derivation"))]
    /// Use Ed25519 key
    Ed25519(MpcKeyTypeEd),
}

#[derive(Clone)]
pub struct MpcKeyTypeContext {
    admin_account_id: near_primitives::types::AccountId,
    key_type: near_crypto::KeyType,
    tx_context: crate::commands::TransactionContext,
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignMpcContext)]
#[interactive_clap(output_context = MpcKeyTypeSecpContext)]
pub struct MpcKeyTypeSecp {
    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDeriveKey,
}

#[derive(Clone)]
pub struct MpcKeyTypeSecpContext(MpcKeyTypeContext);

impl MpcKeyTypeSecpContext {
    fn from_previous_context(
        previous_context: SignMpcContext,
        _scope: &<MpcKeyTypeSecp as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(MpcKeyTypeContext {
            admin_account_id: previous_context.admin_account_id,
            key_type: near_crypto::KeyType::SECP256K1,
            tx_context: previous_context.tx_context,
        }))
    }
}

impl From<MpcKeyTypeSecpContext> for MpcKeyTypeContext {
    fn from(item: MpcKeyTypeSecpContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignMpcContext)]
#[interactive_clap(output_context = MpcKeyTypeEdContext)]
pub struct MpcKeyTypeEd {
    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDeriveKey,
}

#[derive(Clone)]
pub struct MpcKeyTypeEdContext(MpcKeyTypeContext);

impl MpcKeyTypeEdContext {
    fn from_previous_context(
        previous_context: SignMpcContext,
        _scope: &<MpcKeyTypeEd as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self(MpcKeyTypeContext {
            admin_account_id: previous_context.admin_account_id,
            key_type: near_crypto::KeyType::ED25519,
            tx_context: previous_context.tx_context,
        }))
    }
}

impl From<MpcKeyTypeEdContext> for MpcKeyTypeContext {
    fn from(item: MpcKeyTypeEdContext) -> Self {
        item.0
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = MpcKeyTypeContext)]
#[interactive_clap(output_context = MpcDeriveKeyContext)]
pub struct MpcDeriveKey {
    #[interactive_clap(skip_default_input_arg, always_quote)]
    /// What is the derivation path?
    derivation_path: String,
    #[interactive_clap(named_arg)]
    /// Prepaid Gas for calling MPC contract
    prepaid_gas: PrepaidGas,
}

#[derive(Clone)]
pub struct MpcDeriveKeyContext {
    admin_account_id: near_primitives::types::AccountId,
    derived_public_key: near_crypto::PublicKey,
    derivation_path: String,
    nonce: near_primitives::types::Nonce,
    block_hash: near_primitives::hash::CryptoHash,
    tx_context: crate::commands::TransactionContext,
}

impl MpcDeriveKeyContext {
    pub fn from_previous_context(
        previous_context: MpcKeyTypeContext,
        scope: &<MpcDeriveKey as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let network_config = previous_context.tx_context.network_config.clone();
        let controllable_account = previous_context
            .tx_context
            .prepopulated_transaction
            .signer_id
            .clone();

        let derived_public_key = derive_public_key(
            &network_config.get_mpc_contract_account_id()?,
            &previous_context.admin_account_id,
            &scope.derivation_path,
            &previous_context.key_type,
            &network_config,
        )?;

        let json_rpc_response = network_config
                .json_rpc_client()
                .blocking_call_view_access_key(
                    &controllable_account,
                    &derived_public_key.clone(),
                    near_primitives::types::BlockReference::latest(),
                )
                .inspect_err(|err| {
                    if let near_jsonrpc_client::errors::JsonRpcError::ServerError(
                        near_jsonrpc_client::errors::JsonRpcServerError::HandlerError(
                            near_jsonrpc_primitives::types::query::RpcQueryError::UnknownAccessKey { .. },
                        ),
                    ) = &**err {
                        tracing::error!(
                            "Couldn't find a key on rpc. You can add it to controllable account using following command:"
                        );
                        eprintln!("{}",
                            format!(
                                "    {} account add-key {} grant-full-access use-manually-provided-public-key {}",
                                crate::common::get_near_exec_path(),
                                controllable_account,
                                derived_public_key
                            ).yellow()
                        );
                    }
                })
                .wrap_err_with(||
                    format!("Cannot sign MPC transaction for <{}> due to an error while checking if derived key exists on network <{}>", controllable_account, network_config.network_name)
                )?;

        tracing::info!(
            "Derived public key for <{}>:{}",
            controllable_account,
            crate::common::indent_payload(&format!("\n{derived_public_key}\n"))
        );

        Ok(Self {
            admin_account_id: previous_context.admin_account_id,
            derived_public_key,
            derivation_path: scope.derivation_path.clone(),
            nonce: json_rpc_response
                .access_key_view()
                .wrap_err("Error current_nonce")?
                .nonce
                + 1,
            block_hash: json_rpc_response.block_hash,
            tx_context: previous_context.tx_context,
        })
    }
}

impl MpcDeriveKey {
    pub fn input_derivation_path(
        context: &MpcKeyTypeContext,
    ) -> color_eyre::eyre::Result<Option<String>> {
        let derivation_path = inquire::Text::new("What is the derivation path?")
            .with_initial_value(&format!(
                "{}-{}",
                context.admin_account_id, context.tx_context.prepopulated_transaction.signer_id
            ))
            .prompt()?;
        Ok(Some(derivation_path))
    }
}

#[tracing::instrument(name = "Retrieving derived public key from MPC contract ...", skip_all)]
pub fn derive_public_key(
    mpc_contract_address: &near_primitives::types::AccountId,
    admin_account_id: &near_primitives::types::AccountId,
    derivation_path: &str,
    key_type: &near_crypto::KeyType,
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<near_crypto::PublicKey> {
    let rpc_result = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            mpc_contract_address,
            "derived_public_key",
            serde_json::to_vec(&serde_json::json!({
                "path": derivation_path,
                "predecessor": admin_account_id,
                "domain_id": near_key_type_to_mpc_domain_id(*key_type)
            }))?,
            near_primitives::types::BlockReference::latest(),
        )?;

    let public_key: near_crypto::PublicKey = serde_json::from_slice(&rpc_result.result)?;

    Ok(public_key)
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = MpcDeriveKeyContext)]
#[interactive_clap(output_context = PrepaidGasContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the gas limit for signing MPC?
    gas: crate::common::NearGas,

    #[interactive_clap(named_arg)]
    /// Deposit for contract call
    attached_deposit: Deposit,
}

#[derive(Clone)]
pub struct PrepaidGasContext {
    admin_account_id: near_primitives::types::AccountId,
    derived_public_key: near_crypto::PublicKey,
    derivation_path: String,
    nonce: near_primitives::types::Nonce,
    block_hash: near_primitives::hash::CryptoHash,
    tx_context: crate::commands::TransactionContext,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: MpcDeriveKeyContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(PrepaidGasContext {
            admin_account_id: previous_context.admin_account_id,
            derived_public_key: previous_context.derived_public_key,
            derivation_path: previous_context.derivation_path,
            nonce: previous_context.nonce,
            block_hash: previous_context.block_hash,
            tx_context: previous_context.tx_context,
            gas: scope.gas,
        })
    }
}

impl PrepaidGas {
    pub fn input_gas(
        _context: &MpcDeriveKeyContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        Ok(Some(
            CustomType::new("What is the gas limit for signing MPC (if unsure, keep 15 Tgas)?")
                .with_starting_input("15 Tgas")
                .with_validator(move |gas: &crate::common::NearGas| {
                    if gas < &near_gas::NearGas::from_tgas(15) {
                        Ok(inquire::validator::Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom(
                                "Sign call to MPC contract requires minimum of 15 TeraGas"
                                    .to_string(),
                            ),
                        ))
                    } else if gas > &near_gas::NearGas::from_tgas(300) {
                        Ok(inquire::validator::Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom(
                                "You need to enter a value of no more than 300 TeraGas".to_string(),
                            ),
                        ))
                    } else {
                        Ok(inquire::validator::Validation::Valid)
                    }
                })
                .prompt()?,
        ))
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = PrepaidGasContext)]
#[interactive_clap(output_context = DepositContext)]
pub struct Deposit {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter deposit for MPC contract call:
    deposit: crate::types::near_token::NearToken,

    #[interactive_clap(subcommand)]
    transaction_signature_options: mpc_sign_with::MpcSignWith,
}

#[derive(Clone)]
pub struct DepositContext {
    admin_account_id: near_primitives::types::AccountId,
    mpc_contract_address: near_primitives::types::AccountId,
    gas: crate::common::NearGas,
    deposit: crate::types::near_token::NearToken,
    original_payload_transaction: Transaction,
    mpc_sign_request: mpc_sign_request::MpcSignRequest,
    mpc_sign_request_serialized: Vec<u8>,
    global_context: crate::GlobalContext,
    network_config: crate::config::NetworkConfig,
}

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        let controllable_account = previous_context
            .tx_context
            .prepopulated_transaction
            .signer_id;

        let mut payload = TransactionV0 {
            signer_id: controllable_account.clone(),
            receiver_id: previous_context
                .tx_context
                .prepopulated_transaction
                .receiver_id,
            public_key: previous_context.derived_public_key.clone(),
            nonce: previous_context.nonce,
            block_hash: previous_context.block_hash,
            actions: previous_context.tx_context.prepopulated_transaction.actions,
        };

        (previous_context.tx_context.on_before_signing_callback)(
            &mut payload,
            &previous_context.tx_context.network_config,
        )?;

        let mpc_tx_payload = Transaction::V0(payload);
        let hashed_payload = near_primitives::hash::CryptoHash::hash_borsh(&mpc_tx_payload).0;

        let payload: mpc_sign_request::MpcSignPayload =
            match previous_context.derived_public_key.key_type() {
                near_crypto::KeyType::ED25519 => {
                    mpc_sign_request::MpcSignPayload::Eddsa(hashed_payload.to_vec())
                }
                near_crypto::KeyType::SECP256K1 => {
                    mpc_sign_request::MpcSignPayload::Ecdsa(hashed_payload)
                }
            };

        let mpc_sign_request = mpc_sign_request::MpcSignRequest {
            request: mpc_sign_request::MpcSignRequestArgs {
                payload,
                path: previous_context.derivation_path,
                domain_id: near_key_type_to_mpc_domain_id(
                    previous_context.derived_public_key.key_type(),
                ),
            },
        };

        let mpc_sign_request_serialized = serde_json::to_vec(&mpc_sign_request)?;

        Ok(Self {
            admin_account_id: previous_context.admin_account_id,
            mpc_contract_address: previous_context
                .tx_context
                .network_config
                .get_mpc_contract_account_id()?,
            gas: previous_context.gas,
            deposit: scope.deposit,
            original_payload_transaction: mpc_tx_payload,
            mpc_sign_request,
            mpc_sign_request_serialized,
            global_context: previous_context.tx_context.global_context,
            network_config: previous_context.tx_context.network_config,
        })
    }
}

impl Deposit {
    pub fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        Ok(Some(
            CustomType::new(
                "What is the deposit for MPC contract call (if unsure, keep 1 yoctoNEAR)?",
            )
            .with_starting_input("1 yoctoNEAR")
            .with_validator(move |deposit: &crate::types::near_token::NearToken| {
                if deposit < &crate::types::near_token::NearToken::from_yoctonear(1) {
                    Ok(inquire::validator::Validation::Invalid(
                        inquire::validator::ErrorMessage::Custom(
                            "Sign call to MPC contract requires deposit no lower than 1 yoctoNEAR"
                                .to_string(),
                        ),
                    ))
                } else {
                    Ok(inquire::validator::Validation::Valid)
                }
            })
            .prompt()?,
        ))
    }
}

impl From<DepositContext> for crate::commands::TransactionContext {
    fn from(item: DepositContext) -> Self {
        let mpc_sign_transaction = crate::commands::PrepopulatedTransaction {
            signer_id: item.admin_account_id.clone(),
            receiver_id: item.mpc_contract_address.clone(),
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                Box::new(near_primitives::transaction::FunctionCallAction {
                    method_name: "sign".to_string(),
                    args: item.mpc_sign_request_serialized,
                    gas: item.gas.as_gas(),
                    deposit: item.deposit.as_yoctonear(),
                }),
            )],
        };

        tracing::info!(
            "{}{}",
            "Unsigned transaction for signing with MPC contract",
            crate::common::indent_payload(&crate::common::print_unsigned_transaction(
                &mpc_sign_transaction,
            ))
        );

        let original_transaction_for_signing = item.original_payload_transaction.clone();
        let original_transaction_for_after_send = item.original_payload_transaction.clone();
        let global_context_for_after_send = item.global_context.clone();

        let on_after_signing_callback: crate::commands::OnAfterSigningCallback =
            std::sync::Arc::new({
                move |signed_transaction_to_replace, network_config| {
                    if let Some(near_primitives::action::Action::FunctionCall(fc)) =
                        signed_transaction_to_replace.transaction.actions().first()
                    {
                        // NOTE: Early exit if it is a DAO, it will be required to enter different
                        // flow for it
                        if fc.method_name == "add_proposal" {
                            return Ok(());
                        }
                    }

                    let unsigned_transaction = original_transaction_for_signing.clone();

                    let sender_id = unsigned_transaction.signer_id().clone();
                    let receiver_id = unsigned_transaction.receiver_id().clone();
                    let contract_id = item.mpc_contract_address.clone();
                    let sign_request_tx = signed_transaction_to_replace.clone();

                    let sign_outcome_view =
                        match crate::transaction_signature_options::send::sending_signed_transaction(
                            network_config,
                            &sign_request_tx,
                        ) {
                            Ok(outcome_view) => outcome_view,
                            Err(error) => return Err(error),
                        };

                    let signed_transaction = match sign_outcome_view.status {
                        near_primitives::views::FinalExecutionStatus::SuccessValue(result) => {
                            let sign_result: mpc_sign_result::SignResult =
                                serde_json::from_slice(&result)?;
                            let signature: near_crypto::Signature = sign_result.into();

                            near_primitives::transaction::SignedTransaction::new(
                                signature,
                                unsigned_transaction,
                            )
                        }
                        _ => {
                            let error_msg = format!("Failed to sign MPC transaction for <{sender_id}>\nUnexpected outcome view after sending to \"sign\" to <{contract_id}> contract.");
                            eprintln!("{error_msg}");
                            return Err(color_eyre::eyre::eyre!(error_msg));
                        }
                    };

                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "Successfully signed original transaction from <{}> to <{}> via MPC contract <{}>:{}",
                        sender_id,
                        receiver_id,
                        contract_id,
                        crate::common::indent_payload(&format!(
                            "\nSignature:  {}\n",
                            signed_transaction.signature,
                        ))
                    );

                    *signed_transaction_to_replace = signed_transaction;

                    Ok(())
                }
            });

        let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
            move |outcome_view, network_config| {
                let global_context = global_context_for_after_send.clone();
                let unsigned_transaction = original_transaction_for_after_send.clone();
                let mpc_sign_request = item.mpc_sign_request.clone();

                // NOTE: checking if outcome view status is not failure is not neccessary, as this
                // callback will be called only after `crate::common::print_transaction_status`,
                // which will check for failure of transaction already

                if let Some(near_primitives::views::ActionView::FunctionCall { method_name, args, .. }) =
      outcome_view.transaction.actions.first()
                {
                    if method_name == "add_proposal" {
                        if let Ok(Some(proposal)) = serde_json::from_slice::<serde_json::Value>(args).map(|parsed_args| parsed_args.get("proposal").cloned()) {
                            if let Some(kind) = proposal.get("kind") {
                                if serde_json::from_value::<super::submit_dao_proposal::dao_kind_arguments::ProposalKind>(kind.clone()).is_ok() {
                                    dao_sign_with_mpc_after_send_flow(
                                        &global_context,
                                        network_config,
                                        outcome_view,
                                        &unsigned_transaction,
                                        &mpc_sign_request
                                    )?;
                                }
                            }
                        }
                    }
                }

                Ok(())
            }
        });

        Self {
            global_context: item.global_context,
            network_config: item.network_config,
            prepopulated_transaction: mpc_sign_transaction,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepopulated_unsigned_transaction, _network_config| Ok(()),
            ),
            on_after_signing_callback,
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback,
        }
    }
}

pub fn near_key_type_to_mpc_domain_id(key_type: near_crypto::KeyType) -> u64 {
    match key_type {
        near_crypto::KeyType::SECP256K1 => 0u64,
        near_crypto::KeyType::ED25519 => 1u64,
    }
}

pub fn dao_sign_with_mpc_after_send_flow(
    global_context: &crate::GlobalContext,
    network_config: &crate::config::NetworkConfig,
    outcome_view: &near_primitives::views::FinalExecutionOutcomeView,
    unsigned_mpc_transaction: &near_primitives::transaction::Transaction,
    original_sign_request: &mpc_sign_request::MpcSignRequest,
) -> color_eyre::eyre::Result<()> {
    use tracing_indicatif::suspend_tracing_indicatif;

    let signed_transaction = loop {
        let transaction_hash = match suspend_tracing_indicatif(|| {
            inquire::CustomType::new("Enter the transaction hash of the executed DAO proposal:")
                .prompt()
        }) {
            Ok(tx_hash) => tx_hash,
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => {
                return Ok(());
            }
            Err(err) => {
                eprintln!("{}", format!("{err}").red());
                continue;
            }
        };

        let sender_id = match suspend_tracing_indicatif(|| {
            inquire::CustomType::new("Enter the person who initiated execution of DAO proposal:")
                .prompt()
        }) {
            Ok(tx_hash) => tx_hash,
            Err(
                inquire::error::InquireError::OperationCanceled
                | inquire::error::InquireError::OperationInterrupted,
            ) => {
                return Ok(());
            }
            Err(err) => {
                eprintln!("{}", format!("{err}").red());
                continue;
            }
        };

        let signed_transaction = match fetch_mpc_contract_response_from_dao_tx(
            network_config,
            original_sign_request,
            transaction_hash,
            sender_id,
            outcome_view.transaction.receiver_id.clone(),
        ) {
            Ok(sign_result) => {
                let signature: near_crypto::Signature = sign_result.into();

                near_primitives::transaction::SignedTransaction::new(
                    signature,
                    unsigned_mpc_transaction.clone(),
                )
            }
            Err(err) => {
                eprintln!(
                    "{}",
                    format!("Failed to get signature from MPC contract:\n   {err}").red()
                );
                continue;
            }
        };

        break signed_transaction;
    };

    let submit_context = super::SubmitContext {
        network_config: network_config.clone(),
        global_context: global_context.clone(),
        signed_transaction_or_signed_delegate_action:
            crate::transaction_signature_options::SignedTransactionOrSignedDelegateAction::SignedTransaction(
                signed_transaction
            ),
        on_before_sending_transaction_callback:
            std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
        ),
        on_after_sending_transaction_callback:
            std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
        ),
    };

    prompt_and_submit(submit_context)
}

#[tracing::instrument(name = "Fetching executed DAO proposal ...", skip_all)]
fn fetch_mpc_contract_response_from_dao_tx(
    network_config: &crate::config::NetworkConfig,
    original_sign_request: &mpc_sign_request::MpcSignRequest,
    tx_hash: near_primitives::hash::CryptoHash,
    sender_account_id: near_primitives::types::AccountId,
    dao_address: near_primitives::types::AccountId,
) -> color_eyre::eyre::Result<mpc_sign_result::SignResult> {
    let request = near_jsonrpc_client::methods::tx::RpcTransactionStatusRequest {
        transaction_info: near_jsonrpc_client::methods::tx::TransactionInfo::TransactionId {
            tx_hash,
            sender_account_id,
        },
        wait_until: near_primitives::views::TxExecutionStatus::Final,
    };

    let exec_outcome_view = network_config
        .json_rpc_client()
        .blocking_call(request)
        .wrap_err("Couldn't fetch DAO transaction")?
        .final_execution_outcome
        .ok_or(color_eyre::eyre::eyre!("No final execution outcome"))?
        .into_outcome();

    if exec_outcome_view.transaction.receiver_id != *dao_address {
        return Err(color_eyre::eyre::eyre!(
            "Transaction receiver is not dao account!"
        ));
    }

    if !matches!(
        exec_outcome_view.status,
        near_primitives::views::FinalExecutionStatus::SuccessValue(_)
    ) {
        return Err(color_eyre::eyre::eyre!("Transaction did not succeed"));
    }

    let act_proposal_args = exec_outcome_view
        .transaction
        .actions
        .iter()
        .find_map(|action| {
            if let near_primitives::views::ActionView::FunctionCall {
                method_name, args, ..
            } = action
            {
                if method_name == "act_proposal" {
                    return Some(args);
                }
            }
            None
        })
        .ok_or(color_eyre::eyre::eyre!("No act_proposal action found"))?;

    let act_proposal_args: serde_json::Value = serde_json::from_slice(act_proposal_args)?;

    let proposal = act_proposal_args
        .get("proposal")
        .ok_or(color_eyre::eyre::eyre!(
            "Couldn't find proposal in \"act_proposal\""
        ))?;

    let proposal_kind: super::submit_dao_proposal::dao_kind_arguments::ProposalKind =
        serde_json::from_value(proposal.clone())?;

    let mpc_sign_request = proposal_kind.try_to_mpc_sign_request(network_config)?;

    if mpc_sign_request != *original_sign_request {
        return Err(color_eyre::eyre::eyre!("Fetched sign request from DAO proposal doesn't match original that was made in this session"));
    };

    let mut sign_response_opt = None;
    let mpc_contract_address = network_config
        .get_mpc_contract_account_id()
        .expect("Already checked it before calling");

    for receipt in exec_outcome_view.receipts_outcome {
        if receipt.outcome.executor_id == mpc_contract_address {
            if let near_primitives::views::ExecutionStatusView::SuccessValue(success_response) =
                receipt.outcome.status
            {
                sign_response_opt = Some(success_response);
                break;
            }
        }
    }

    let Some(sign_response_vec) = sign_response_opt else {
        return Err(color_eyre::eyre::eyre!(
            "Couldn't find response from MPC contract"
        ));
    };

    let mpc_contract_sign_result: mpc_sign_result::SignResult =
        serde_json::from_slice(&sign_response_vec).map_err(|_| {
            color_eyre::eyre::eyre!("Couldn't parse sign response from MPC contract")
        })?;

    Ok(mpc_contract_sign_result)
}

fn prompt_and_submit(submit_context: super::SubmitContext) -> color_eyre::eyre::Result<()> {
    use strum::IntoEnumIterator;

    let choices: Vec<_> = super::SubmitDiscriminants::iter()
        .map(|variant| {
            let message = variant.get_message().unwrap_or("Unknown");
            (message.to_string(), variant)
        })
        .collect();

    let display_options: Vec<&str> = choices.iter().map(|(msg, _)| msg.as_str()).collect();

    let selected_msg = tracing_indicatif::suspend_tracing_indicatif(|| {
        inquire::Select::new("How would you like to proceed?", display_options).prompt()
    })?;

    let selected = choices
        .iter()
        .find(|(msg, _)| msg.as_str() == selected_msg)
        .map(|(_, variant)| variant)
        .unwrap();

    match selected {
        super::SubmitDiscriminants::Send => {
            super::send::SendContext::from_previous_context(
                submit_context,
                &super::send::InteractiveClapContextScopeForSend {},
            )?;
        }
        super::SubmitDiscriminants::Display => {
            super::display::DisplayContext::from_previous_context(
                submit_context,
                &super::display::InteractiveClapContextScopeForDisplay {},
            )?;
        }
        super::SubmitDiscriminants::SaveToFile => {
            let file_path: crate::types::path_buf::PathBuf = loop {
                match CustomType::new(
                    "What is the location of the file to save the transaction information?",
                )
                .with_starting_input("signed-transaction-info.json")
                .prompt()
                {
                    Ok(file_path) => break file_path,
                    Err(
                        inquire::error::InquireError::OperationCanceled
                        | inquire::error::InquireError::OperationInterrupted,
                    ) => {
                        return Ok(());
                    }
                    Err(err) => {
                        eprintln!("{err}");
                        continue;
                    }
                }
            };
            super::save_to_file::SaveToFileContext::from_previous_context(
                submit_context,
                &super::save_to_file::InteractiveClapContextScopeForSaveToFile { file_path },
            )?;
        }
    }

    Ok(())
}
