use color_eyre::{eyre::Context, owo_colors::OwoColorize};
use inquire::CustomType;
use near_primitives::transaction::{Transaction, TransactionV0};

use crate::common::{JsonRpcClientExt, RpcQueryResponseExt};

mod mpc_sign_request;
mod mpc_sign_result;
mod mpc_sign_with;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignMpcContext)]
pub struct SignMpc {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the Admin account address?
    admin_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(subargs)]
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
        let _ = previous_context
            .network_config
            .get_mpc_contract_account_id()?;

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

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = SignMpcContext)]
#[interactive_clap(output_context = MpcKeyTypeContext)]
pub struct MpcKeyType {
    #[interactive_clap(skip_default_input_arg)]
    /// What is the MPC key type for derivation?
    key_type: crate::types::key_type::KeyType,

    #[interactive_clap(named_arg)]
    /// What is the derivation path?
    derivation_path: MpcDeriveKey,
}

#[derive(Clone)]
pub struct MpcKeyTypeContext {
    admin_account_id: near_primitives::types::AccountId,
    key_type: crate::types::key_type::KeyType,
    tx_context: crate::commands::TransactionContext,
}

impl MpcKeyTypeContext {
    pub fn from_previous_context(
        previous_context: SignMpcContext,
        scope: &<MpcKeyType as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(Self {
            admin_account_id: previous_context.admin_account_id,
            key_type: scope.key_type.clone(),
            tx_context: previous_context.tx_context,
        })
    }
}

impl MpcKeyType {
    pub fn input_key_type(
        _context: &SignMpcContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::key_type::KeyType>> {
        let options = vec![
            crate::types::key_type::KeyType(near_crypto::KeyType::SECP256K1),
            crate::types::key_type::KeyType(near_crypto::KeyType::ED25519),
        ];

        let selection = inquire::Select::new(
            "What is the MPC key type for derivation (if unsure choose Secp256K1)?",
            options,
        )
        .prompt()?;

        Ok(Some(selection))
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

        if previous_context.tx_context.global_context.offline {
            eprintln!("\nInternet connection is required to retrieve and check derived key!");
            return Err(color_eyre::eyre::eyre!(
                "Internet connection is required to retrieve and check derived key!"
            ));
        }

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
    key_type: &crate::types::key_type::KeyType,
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
                "domain_id": key_type.to_mpc_domain_id(),
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
    mpc_tx_args: Vec<u8>,
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

        let payload: mpc_sign_request::SignPayload = match previous_context
            .derived_public_key
            .key_type()
        {
            near_crypto::KeyType::ED25519 => {
                mpc_sign_request::SignPayload::Eddsa(hashed_payload.to_vec())
            }
            near_crypto::KeyType::SECP256K1 => mpc_sign_request::SignPayload::Ecdsa(hashed_payload),
        };

        let mpc_tx_args = serde_json::to_vec(&serde_json::json!({
            "request": mpc_sign_request::SignRequest {
                payload,
                path: previous_context.derivation_path,
                domain_id: crate::types::key_type::near_key_type_to_mpc_domain_id(
                    previous_context.derived_public_key.key_type(),
                ),
            }
        }))?;

        Ok(Self {
            admin_account_id: previous_context.admin_account_id,
            mpc_contract_address: previous_context
                .tx_context
                .network_config
                .get_mpc_contract_account_id()?,
            gas: previous_context.gas,
            deposit: scope.deposit,
            original_payload_transaction: mpc_tx_payload,
            mpc_tx_args,
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
                    args: item.mpc_tx_args,
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

        let on_after_signing_callback: crate::commands::OnAfterSigningCallback =
            std::sync::Arc::new({
                move |signed_transaction_to_replace, network_config| {
                    let unsigned_transaction = item.original_payload_transaction.clone();
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
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}
