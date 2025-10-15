use color_eyre::eyre::Context;
use inquire::CustomType;

use crate::common::{JsonRpcClientExt, RpcQueryResponseExt};

mod derivation_logic;
mod mpc_sign_with;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::commands::TransactionContext)]
#[interactive_clap(output_context = SignMpcContext)]
pub struct SignMpc {
    // TODO: maybe include contract mpc check for user
    #[interactive_clap(skip_default_input_arg)]
    /// What is the Admin account addres?
    admin_account_id: crate::types::account_id::AccountId,

    #[interactive_clap(subargs)]
    /// MPC key retrival and SECP256K1 derivation logic
    mpc_derive: MpcDerive,
}

#[derive(Clone)]
pub struct SignMpcContext {
    admin_account_id: near_primitives::types::AccountId,
    mpc_contract_address: near_primitives::types::AccountId,
    tx_context: crate::commands::TransactionContext,
}

impl SignMpcContext {
    pub fn from_previous_context(
        previous_context: crate::commands::TransactionContext,
        scope: &<SignMpc as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        // TODO: check the smart_contract_address exists still...
        let network_name = &previous_context.network_config.network_name;
        let mpc_contract_address = if network_name.contains("mainnet") {
            "v1.signer".parse()?
        } else if network_name.contains("testnet") {
            "v1.signer-prod.testnet".parse()?
        } else {
            return Err(color_eyre::eyre::eyre!(
                "Network name should contain \"mainnet\" or \"testnet\" to get MPC contract address!"
            ));
        };

        // TODO: can also check if MPC and admin account exists using common::is_account_exists
        // function

        Ok(SignMpcContext {
            admin_account_id: scope.admin_account_id.clone().into(),
            mpc_contract_address,
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
#[interactive_clap(output_context = MpcDeriveContext)]
pub struct MpcDerive {
    #[interactive_clap(named_arg)]
    /// Prepaid Gas for calling MPC contract
    prepaid_gas: PrepaidGas,
}

#[derive(Clone)]
pub struct MpcDeriveContext {
    admin_account_id: near_primitives::types::AccountId,
    mpc_contract_address: near_primitives::types::AccountId,
    derived_public_key: near_crypto::Secp256K1PublicKey,
    nounce: near_primitives::types::Nonce,
    block_hash: near_primitives::hash::CryptoHash,
    tx_context: crate::commands::TransactionContext,
}

impl MpcDeriveContext {
    pub fn from_previous_context(
        previous_context: SignMpcContext,
        _scope: &<MpcDerive as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        // NOTE: as of now, default key in v1.signer and v1.signer-prod.testnet MPC contracts is Secp256K1
        // This code works using this key only

        let network_config = previous_context.tx_context.network_config.clone();
        let controlable_account = previous_context
            .tx_context
            .prepopulated_transaction
            .signer_id
            .clone();
        let derive_path = format!(
            "{}-{}",
            previous_context.admin_account_id, controlable_account
        );

        // TODO: Check if this is required and checked before
        if previous_context.tx_context.global_context.offline {
            eprintln!("\nInternet connection is required to retrieve and check derived key!");
            return Err(color_eyre::eyre::eyre!(
                "Internet connection is required to retrieve and check derived key!"
            ));
        }

        // 1. recieve public key for mpc_contract_address
        let mpc_public_key: near_crypto::Secp256K1PublicKey =
            retrieve_mpc_public_key(&previous_context.mpc_contract_address, &network_config)?;

        // 2. derive key for the path
        let derived_public_key = derivation_logic::derive_public_key(
            &mpc_public_key,
            &previous_context.admin_account_id,
            &derive_path,
        )?;

        tracing::info!(
            "Derived public key for {controlable_account}: secp256k1:{}",
            bs58::encode(&derived_public_key).into_string()
        );

        // 3. check if key is published to controllable_account
        let json_rpc_response = network_config
                .json_rpc_client()
                .blocking_call_view_access_key(
                    &controlable_account,
                    &derived_public_key.clone().into(),
                    near_primitives::types::BlockReference::latest(),
                )
                .wrap_err_with(||
                    format!("Cannot sign a transaction due to an error while fetching derived key for most recent nonce value on network <{}>", network_config.network_name)
                )?;

        tracing::info!("Found derived key in controllable account {controlable_account}");

        Ok(Self {
            admin_account_id: previous_context.admin_account_id,
            mpc_contract_address: previous_context.mpc_contract_address,
            derived_public_key,
            nounce: json_rpc_response
                .access_key_view()
                .wrap_err("Error current_nonce")?
                .nonce
                + 1,
            block_hash: json_rpc_response.block_hash,
            tx_context: previous_context.tx_context,
        })
    }
}

#[tracing::instrument(name = "Retrieving public key from MPC contract ...", skip_all)]
pub fn retrieve_mpc_public_key(
    mpc_contract_address: &near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
) -> color_eyre::eyre::Result<near_crypto::Secp256K1PublicKey> {
    let rpc_result = network_config
        .json_rpc_client()
        .blocking_call_view_function(
            mpc_contract_address,
            "public_key",
            serde_json::to_vec(&serde_json::json!({}))?,
            near_primitives::types::BlockReference::latest(),
        )?;

    let public_key: near_crypto::PublicKey = serde_json::from_slice(&rpc_result.result)?;

    match public_key {
        near_crypto::PublicKey::SECP256K1(key) => Ok(key),
        _ => Err(color_eyre::eyre::eyre!(
            "Expected Secp256K1 public key from MPC contract, got different type..."
        )),
    }
}

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = MpcDeriveContext)]
#[interactive_clap(output_context = PrepaidGasContext)]
pub struct PrepaidGas {
    #[interactive_clap(skip_default_input_arg)]
    /// Enter gas amount for contract call:
    gas: crate::common::NearGas,
    #[interactive_clap(named_arg)]
    /// Enter deposit for contract call:
    attached_deposit: Deposit,
}

#[derive(Clone)]
pub struct PrepaidGasContext {
    admin_account_id: near_primitives::types::AccountId,
    mpc_contract_address: near_primitives::types::AccountId,
    derived_public_key: near_crypto::Secp256K1PublicKey,
    nounce: near_primitives::types::Nonce,
    block_hash: near_primitives::hash::CryptoHash,
    tx_context: crate::commands::TransactionContext,
    gas: crate::common::NearGas,
}

impl PrepaidGasContext {
    pub fn from_previous_context(
        previous_context: MpcDeriveContext,
        scope: &<PrepaidGas as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        Ok(PrepaidGasContext {
            admin_account_id: previous_context.admin_account_id,
            mpc_contract_address: previous_context.mpc_contract_address,
            derived_public_key: previous_context.derived_public_key,
            nounce: previous_context.nounce,
            block_hash: previous_context.block_hash,
            tx_context: previous_context.tx_context,
            gas: scope.gas,
        })
    }
}

impl PrepaidGas {
    pub fn input_gas(
        _context: &MpcDeriveContext,
    ) -> color_eyre::eyre::Result<Option<crate::common::NearGas>> {
        Ok(Some(
            CustomType::new("What is the gas limit for signing MPC (if unsure, keep 3 Tgas)?")
                .with_starting_input("3 Tgas")
                .with_validator(move |gas: &crate::common::NearGas| {
                    if gas > &near_gas::NearGas::from_tgas(300) {
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
    derived_public_key: near_crypto::Secp256K1PublicKey,
    nounce: near_primitives::types::Nonce,
    block_hash: near_primitives::hash::CryptoHash,
    tx_context: crate::commands::TransactionContext,
    gas: crate::common::NearGas,
    deposit: crate::types::near_token::NearToken,
}

impl DepositContext {
    pub fn from_previous_context(
        previous_context: PrepaidGasContext,
        scope: &<Deposit as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        // TODO: maybe convert to tx here - it can throw and we can construct it.
        Ok(Self {
            admin_account_id: previous_context.admin_account_id,
            mpc_contract_address: previous_context.mpc_contract_address,
            derived_public_key: previous_context.derived_public_key,
            nounce: previous_context.nounce,
            block_hash: previous_context.block_hash,
            tx_context: previous_context.tx_context,
            gas: previous_context.gas,
            deposit: scope.deposit,
        })
    }
}

impl Deposit {
    pub fn input_deposit(
        _context: &PrepaidGasContext,
    ) -> color_eyre::eyre::Result<Option<crate::types::near_token::NearToken>> {
        Ok(Some(
            CustomType::new("Enter deposit for MPC contract call (if unsure, keep 0.1 NEAR):")
                .with_starting_input("0.1 NEAR")
                .prompt()?,
        ))
    }
}

impl From<DepositContext> for crate::commands::TransactionContext {
    fn from(item: DepositContext) -> Self {
        let admin_account = item.admin_account_id;
        let controllable_account = item.tx_context.prepopulated_transaction.signer_id;
        let contract_address = item.mpc_contract_address;
        let tx_reciever = item.tx_context.prepopulated_transaction.receiver_id;
        let tx_actions = item.tx_context.prepopulated_transaction.actions;

        // Should be almost original account and thingy
        let payload = near_primitives::transaction::TransactionV0 {
            signer_id: controllable_account.clone(),
            receiver_id: tx_reciever,
            public_key: item.derived_public_key.into(),
            nonce: item.nounce,
            block_hash: item.block_hash,
            actions: tx_actions,
        };

        let serialized_near_tx = borsh::to_vec(&payload).unwrap();
        let hashed_transaction = hash_payload(&serialized_near_tx);
        let mpc_tx_args = serde_json::to_vec(&serde_json::json!({
            "payload": hashed_transaction,
            "path": format!("{}-{}", admin_account, controllable_account),
            "key_version": 0u32,
        }))
        .unwrap();

        let mpc_sign_transaction = crate::commands::PrepopulatedTransaction {
            signer_id: admin_account,
            receiver_id: contract_address,
            actions: vec![near_primitives::transaction::Action::FunctionCall(
                Box::new(near_primitives::transaction::FunctionCallAction {
                    method_name: "sign".to_string(),
                    args: mpc_tx_args,
                    gas: item.gas.as_gas(),
                    deposit: item.deposit.as_yoctonear(),
                }),
            )],
        };

        tracing::info!(
            "{}{}",
            "Unsigned MPC transaction",
            crate::common::indent_payload(&crate::common::print_unsigned_transaction(
                &mpc_sign_transaction,
            ))
        );

        todo!();

        // let on_after_sending_transaction_callback: crate::transaction_signature_options::OnAfterSendingTransactionCallback = std::sync::Arc::new({
        //     let signer_account_id = item.signer_account_id.clone();
        //     let nft_contract_account_id = item.nft_contract_account_id.clone();
        //     let receiver_account_id = item.receiver_account_id.clone();
        //     let token_id = item.token_id.clone();
        //
        //     move |outcome_view, network_config| {
        //         if let near_primitives::views::FinalExecutionStatus::SuccessValue(_) = outcome_view.status {
        //             let info_str = format!(
        //                 "<{signer_account_id}> has successfully transferred NFT token_id=\"{token_id}\" to <{receiver_account_id}> on contract <{nft_contract_account_id}>.",
        //             );
        //             tracing::info!(
        //                 parent: &tracing::Span::none(),
        //                 "\n{}",
        //                 crate::common::indent_payload(&info_str)
        //             );
        //         }
        //         Ok(())
        //     }
        // });

        Self {
            global_context: item.tx_context.global_context,
            network_config: item.tx_context.network_config,
            prepopulated_transaction: mpc_sign_transaction,
            on_before_signing_callback: std::sync::Arc::new(
                |_prepolulated_unsinged_transaction, _network_config| Ok(()),
            ),
            on_before_sending_transaction_callback: std::sync::Arc::new(
                |_signed_transaction, _network_config| Ok(String::new()),
            ),
            on_after_sending_transaction_callback: std::sync::Arc::new(
                |_outcome_view, _network_config| Ok(()),
            ),
        }
    }
}

fn hash_payload(payload: &[u8]) -> [u8; 32] {
    let mut hasher = cargo_util::Sha256::new();
    hasher.update(payload);
    hasher.finish()
}
