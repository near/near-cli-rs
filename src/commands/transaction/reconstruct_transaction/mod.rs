use color_eyre::eyre::{Context, ContextCompat};
use inquire::CustomType;
use interactive_clap::ToCliArgs;

#[derive(Debug, Clone, interactive_clap::InteractiveClap)]
#[interactive_clap(input_context = crate::GlobalContext)]
#[interactive_clap(output_context = TransactionInfoContext)]
pub struct TransactionInfo {
    /// Enter the hash of the transaction you want to use as a template:
    transaction_hash: crate::types::crypto_hash::CryptoHash,
    #[interactive_clap(named_arg)]
    /// Select network
    network_config: crate::network::Network,
}

#[derive(Clone)]
pub struct TransactionInfoContext(crate::network::NetworkContext);

impl TransactionInfoContext {
    pub fn from_previous_context(
        previous_context: crate::GlobalContext,
        scope: &<TransactionInfo as interactive_clap::ToInteractiveClapContextScope>::InteractiveClapContextScope,
    ) -> color_eyre::eyre::Result<Self> {
        use super::construct_transaction::{
            CliConstructTransaction, CliDirectReceiver, CliReceiverMode, add_action_1, skip_action,
        };
        use super::{CliTransactionActions, CliTransactionCommands};

        let on_after_getting_network_callback: crate::network::OnAfterGettingNetworkCallback =
            std::sync::Arc::new({
                let tx_hash: near_primitives::hash::CryptoHash = scope.transaction_hash.into();

                move |network_config: &crate::config::NetworkConfig| {
                    let query_view_transaction_status = super::view_status::get_transaction_info(network_config, tx_hash)?
                        .final_execution_outcome
                        .wrap_err_with(|| {
                            format!(
                                "Failed to get the final execution outcome for the transaction {tx_hash}"
                            )
                        })?
                        .into_outcome();

                    let mut prepopulated_transaction = crate::commands::PrepopulatedTransaction {
                        signer_id: query_view_transaction_status.transaction.signer_id,
                        receiver_id: query_view_transaction_status.transaction.receiver_id,
                        actions: query_view_transaction_status
                            .transaction
                            .actions
                            .into_iter()
                            .map(near_primitives::transaction::Action::try_from)
                            .collect::<Result<Vec<near_primitives::transaction::Action>, _>>()
                            .map_err(|err| {
                                color_eyre::eyre::eyre!(
                                    "Failed to reconstruct transaction actions: {err}"
                                )
                            })?,
                    };

                    tracing::info!(
                        parent: &tracing::Span::none(),
                        "Transaction {}:{}",
                        query_view_transaction_status.transaction.hash,
                        crate::common::indent_payload(&crate::common::print_unsigned_transaction(
                            &prepopulated_transaction,
                        ))
                    );

                    unwrap_delegate_action(&mut prepopulated_transaction)?;

                    let cmd =
                        crate::commands::CliTopLevelCommand::Transaction(CliTransactionCommands {
                            transaction_actions: Some(CliTransactionActions::ConstructTransaction(
                                CliConstructTransaction {
                                    sender_account_id: Some(
                                        prepopulated_transaction.signer_id.into(),
                                    ),
                                    receiver: Some(CliReceiverMode::ReceiverId(
                                        CliDirectReceiver {
                                            receiver_account_id: Some(
                                                prepopulated_transaction.receiver_id.clone().into(),
                                            ),
                                            next_actions: None,
                                        },
                                    )),
                                },
                            )),
                        });
                    let mut cmd_cli_args = cmd.to_cli_args();

                    for transaction_action in prepopulated_transaction.actions {
                        let next_actions = add_action_1::CliNextAction::AddAction(
                            add_action_1::add_action::CliAddAction {
                                action: action_transformation(
                                    transaction_action,
                                    prepopulated_transaction.receiver_id.clone(),
                                    network_config,
                                    near_primitives::types::BlockReference::BlockId(
                                        near_primitives::types::BlockId::Hash(
                                            query_view_transaction_status
                                                .transaction_outcome
                                                .block_hash,
                                        ),
                                    ),
                                )?,
                            },
                        );
                        cmd_cli_args.extend(next_actions.to_cli_args());
                    }

                    let skip_action = add_action_1::CliNextAction::Skip(skip_action::CliSkipAction {
                        network_config: Some(
                            skip_action::ClapNamedArgNetworkForTransactionArgsForSkipAction::NetworkConfig(
                                crate::network_for_transaction::CliNetworkForTransactionArgs {
                                    network_name: Some(network_config.network_name.clone()),
                                    transaction_signature_options: None,
                                },
                            ),
                        ),
                    });
                    cmd_cli_args.extend(skip_action.to_cli_args());

                    let near_cli_exec_path = crate::common::get_near_exec_path();
                    if let crate::Verbosity::Interactive | crate::Verbosity::TeachMe =
                        previous_context.verbosity
                    {
                        eprintln!(
                            "Here is your console command to run archive transaction. You can to edit it or re-run (printed to stdout):"
                        )
                    }
                    println!(
                        "{}",
                        shell_words::join(std::iter::once(near_cli_exec_path).chain(cmd_cli_args))
                    );
                    Ok(())
                }
            });

        Ok(Self(crate::network::NetworkContext {
            config: previous_context.config,
            interacting_with_account_ids: vec![],
            on_after_getting_network_callback,
        }))
    }
}

impl From<TransactionInfoContext> for crate::network::NetworkContext {
    fn from(item: TransactionInfoContext) -> Self {
        item.0
    }
}

fn unwrap_delegate_action(
    transaction: &mut crate::commands::PrepopulatedTransaction,
) -> color_eyre::eyre::Result<()> {
    if transaction.actions.len() == 1
        && let near_primitives::transaction::Action::Delegate(signed_delegate_action) =
            &transaction.actions[0]
    {
        // Unlike top-level action views, delegated actions contain actual Wasm.
        // Normalize them to view semantics so both deployment paths carry hashes.
        let actions = signed_delegate_action
            .delegate_action
            .get_actions()
            .into_iter()
            .map(near_primitives::views::ActionView::from)
            .map(near_primitives::transaction::Action::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| {
                color_eyre::eyre::eyre!("Failed to reconstruct delegated actions: {err}")
            })?;
        *transaction = crate::commands::PrepopulatedTransaction {
            signer_id: signed_delegate_action.delegate_action.sender_id.clone(),
            receiver_id: signed_delegate_action.delegate_action.receiver_id.clone(),
            actions,
        };
    }
    Ok(())
}

fn action_transformation(
    archival_action: near_primitives::transaction::Action,
    receiver_id: near_primitives::types::AccountId,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
) -> color_eyre::eyre::Result<
    Option<super::construct_transaction::add_action_1::add_action::CliActionSubcommand>,
> {
    use near_primitives::transaction::Action;

    use super::construct_transaction::add_action_1::add_action;

    match archival_action {
        Action::CreateAccount(_) => {
            Ok(Some(add_action::CliActionSubcommand::CreateAccount(
                add_action::create_account::CliCreateAccountAction {
                    next_action: None
                }
            )))
        }
        Action::DeleteAccount(delete_account_action) => {
            Ok(Some(add_action::CliActionSubcommand::DeleteAccount(
                add_action::delete_account::CliDeleteAccountAction {
                    beneficiary_id: Some(delete_account_action.beneficiary_id.into()),
                    next_action: None
                }
            )))
        }
        Action::AddKey(add_key_action) => {
            Ok(Some(add_action::CliActionSubcommand::AddKey(
                add_action::add_key::CliAddKeyAction {
                    permission: get_access_key_permission(add_key_action.public_key, add_key_action.access_key.permission)?
                }
            )))
        }
        Action::DeleteKey(delete_key_action) => {
            Ok(Some(add_action::CliActionSubcommand::DeleteKey(
                add_action::delete_key::CliDeleteKeyAction {
                    public_keys: Some(vec![delete_key_action.public_key].into()),
                    next_action: None
                }
            )))
        }
        Action::Transfer(transfer_action) => {
            Ok(Some(add_action::CliActionSubcommand::Transfer(
                add_action::transfer::CliTransferAction {
                    amount_in_near: Some(transfer_action.deposit.into()),
                    next_action: None
                }
            )))
        }
        Action::DeployContract(deploy_contract_action) => {
            let code_hash = deploy_action_code_hash(&deploy_contract_action.code)?;
            let file_path = CustomType::<crate::types::path_buf::PathBuf>::new("Enter the file path where to save the contract:")
                .with_starting_input("reconstruct-transaction-deploy-code.wasm")
                .prompt()?;

            download_code(
                &crate::commands::contract::download_wasm::ContractType::Regular(receiver_id),
                network_config,
                block_reference,
                &file_path,
                &code_hash
            )?;
            Ok(Some(add_action::CliActionSubcommand::DeployContract(
                add_action::deploy_contract::CliDeployContractAction {
                    use_file: Some(add_action::deploy_contract::ClapNamedArgContractFileForDeployContractAction::UseFile(
                        add_action::deploy_contract::CliContractFile {
                            file_path: Some(file_path),
                            initialize: Some(add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                                add_action::deploy_contract::initialize_mode::CliNoInitialize {
                                    next_action: None
                                }
                            ))
                        }
                    )),
                }
            )))
        }
        Action::FunctionCall(function_call_action) => {
            Ok(Some(add_action::CliActionSubcommand::FunctionCall(
                add_action::call_function::CliFunctionCallAction {
                    function_name: Some(function_call_action.method_name),
                    function_args_type: Some(crate::commands::contract::call_function::call_function_args_type::FunctionArgsType::TextArgs),
                    function_args: Some(String::from_utf8(function_call_action.args)?),
                    prepaid_gas: Some(add_action::call_function::ClapNamedArgPrepaidGasForFunctionCallAction::PrepaidGas(
                        add_action::call_function::CliPrepaidGas {
                            gas: Some(near_gas::NearGas::from_gas(function_call_action.gas.as_gas())),
                            attached_deposit: Some(add_action::call_function::ClapNamedArgDepositForPrepaidGas::AttachedDeposit(
                                add_action::call_function::CliDeposit {
                                    deposit: Some(function_call_action.deposit.into()),
                                    next_action: None
                                }
                            ))
                        }
                    ))
                }
            )))
        }
        Action::Stake(stake_action) => {
                Ok(Some(add_action::CliActionSubcommand::Stake(
                add_action::stake::CliStakeAction {
                    stake_amount: Some(stake_action.stake.into()),
                    public_key: Some(stake_action.public_key.into()),
                    next_action: None
                }
            )))
        }
        Action::Delegate(_) => Err(color_eyre::eyre::eyre!(
            "Reconstructing a Delegate action is only supported when it is the only action in the transaction."
        )),
        Action::DelegateV2(_) => Err(color_eyre::eyre::eyre!(
            "Reconstructing DelegateV2 (meta) transactions is not supported."
        )),
        Action::DeployGlobalContract(action) => {
            let code_hash = deploy_action_code_hash(action.code.as_ref())?;
            let file_path = CustomType::<crate::types::path_buf::PathBuf>::new("Enter the file path where to save the contract:")
                .with_starting_input("reconstruct-transaction-deploy-code.wasm")
                .prompt()?;

            let contract_type = match action.deploy_mode {
                near_primitives::action::GlobalContractDeployMode::AccountId => {
                    &crate::commands::contract::download_wasm::ContractType::GlobalContractByAccountId {
                        account_id: receiver_id.clone(),
                        code_hash: Some(code_hash)
                    }
                }
                near_primitives::action::GlobalContractDeployMode::CodeHash => {
                    &crate::commands::contract::download_wasm::ContractType::GlobalContractByCodeHash(code_hash)
                }
            };

            download_code(
                contract_type,
                network_config,
                block_reference,
                &file_path,
                &code_hash
            )?;

            let mode = match action.deploy_mode {
                near_primitives::action::GlobalContractDeployMode::AccountId => add_action::deploy_global_contract::CliDeployGlobalMode::AsGlobalAccountId(
                    add_action::deploy_global_contract::CliNextCommand {
                        next_action: None
                    }
                ),
                near_primitives::action::GlobalContractDeployMode::CodeHash => add_action::deploy_global_contract::CliDeployGlobalMode::AsGlobalHash(
                    add_action::deploy_global_contract::CliNextCommand {
                        next_action: None
                    }
                ),
            };
            Ok(Some(add_action::CliActionSubcommand::DeployGlobalContract(
                add_action::deploy_global_contract::CliDeployGlobalContractAction {
                    file_path: Some(file_path),
                    mode: Some(mode)
                }
            )))
        }
        Action::UseGlobalContract(use_global_contract_action) => {
            let mode = match use_global_contract_action.contract_identifier {
                near_primitives::action::GlobalContractIdentifier::CodeHash(hash) => add_action::use_global_contract::CliUseGlobalActionMode::UseGlobalHash(
                    add_action::use_global_contract::CliUseHashAction {
                        hash: Some(crate::types::crypto_hash::CryptoHash(hash)),
                        initialize: Some(add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                            add_action::deploy_contract::initialize_mode::CliNoInitialize {
                                next_action: None
                            }
                        ))
                    }
                ),
                near_primitives::action::GlobalContractIdentifier::AccountId(account_id) => add_action::use_global_contract::CliUseGlobalActionMode::UseGlobalAccountId(
                    add_action::use_global_contract::CliUseAccountIdAction {
                        account_id: Some(crate::types::account_id::AccountId(account_id)),
                        initialize: Some(add_action::deploy_contract::initialize_mode::CliInitializeMode::WithoutInitCall(
                            add_action::deploy_contract::initialize_mode::CliNoInitialize {
                                next_action: None
                            }
                        ))
                    }
                ),
            };

            Ok(Some(add_action::CliActionSubcommand::UseGlobalContract(
                add_action::use_global_contract::CliUseGlobalContractAction {
                    mode: Some(mode)
                }
            )))
        }
        Action::DeterministicStateInit(_deterministic_state_action) => {
            // TODO: impl
            Err(color_eyre::eyre::eyre!("Deterministic state init is not yet implemented"))
        }
        Action::TransferToGasKey(_) | Action::WithdrawFromGasKey(_) => {
            // TODO: impl
            Err(color_eyre::eyre::eyre!("Gas key actions are not yet supported in transaction reconstruction"))
        }
    }
}

fn get_access_key_permission(
    public_key: near_crypto::PublicKey,
    access_key_permission: near_primitives::account::AccessKeyPermission,
) -> color_eyre::eyre::Result<
    Option<super::construct_transaction::add_action_1::add_action::add_key::CliAccessKeyPermission>,
> {
    use super::construct_transaction::add_action_1::add_action::add_key;

    match access_key_permission {
        near_primitives::account::AccessKeyPermission::FullAccess => {
            Ok(Some(add_key::CliAccessKeyPermission::GrantFullAccess(
                add_key::access_key_type::CliFullAccessType {
                    access_key_mode: Some(add_key::CliAccessKeyMode::UseManuallyProvidedPublicKey(
                        add_key::use_public_key::CliAddAccessKeyAction {
                            public_key: Some(public_key.into()),
                            next_action: None,
                        },
                    )),
                },
            )))
        }
        near_primitives::account::AccessKeyPermission::FunctionCall(
            near_primitives::account::FunctionCallPermission {
                allowance,
                receiver_id,
                method_names,
            },
        ) => Ok(Some(
            add_key::CliAccessKeyPermission::GrantFunctionCallAccess(
                add_key::access_key_type::CliFunctionCallType {
                    allowance: {
                        match allowance {
                            Some(allowance) => {
                                Some(crate::types::near_allowance::NearAllowance::from_near(
                                    allowance.into(),
                                ))
                            }
                            None => Some(crate::types::near_allowance::NearAllowance::unlimited()),
                        }
                    },
                    contract_account_id: Some(receiver_id.parse()?),
                    function_names: Some(crate::types::vec_string::VecString(method_names)),
                    access_key_mode: Some(add_key::CliAccessKeyMode::UseManuallyProvidedPublicKey(
                        add_key::use_public_key::CliAddAccessKeyAction {
                            public_key: Some(public_key.into()),
                            next_action: None,
                        },
                    )),
                },
            ),
        )),
        near_primitives::account::AccessKeyPermission::GasKeyFunctionCall(_, _)
        | near_primitives::account::AccessKeyPermission::GasKeyFullAccess(_) => {
            // TODO: impl
            Err(color_eyre::eyre::eyre!(
                "Gas key permissions are not yet supported in transaction reconstruction"
            ))
        }
    }
}

fn download_code(
    contract_type: &crate::commands::contract::download_wasm::ContractType,
    network_config: &crate::config::NetworkConfig,
    block_reference: near_primitives::types::BlockReference,
    file_path: &crate::types::path_buf::PathBuf,
    hash_to_match: &near_primitives::hash::CryptoHash,
) -> color_eyre::eyre::Result<()> {
    // Unfortunately, RPC doesn't return the code for the deployed contract. Only the hash.
    // So we need to fetch it from archive node.

    let code = crate::commands::contract::download_wasm::get_code_with_hash(
        contract_type,
        network_config,
        block_reference,
        Some(*hash_to_match),
    ).map_err(|e| {
        color_eyre::Report::msg(format!("Couldn't fetch the code. Please verify that you are using the archival node in the `network_connection.*.rpc_url` field of the `config.toml` file. You can see the list of RPC providers at https://docs.near.org/api/rpc/providers.\nError: {e}"))
    })?;

    let code_hash = verify_downloaded_code(&code, hash_to_match)?;
    tracing::info!(
        parent: &tracing::Span::none(),
        "The code for <{}> was downloaded successfully with hash <{}>",
        contract_type,
        code_hash,
    );

    std::fs::write(file_path, code).wrap_err(format!(
        "Failed to write the deploy command code to file: '{file_path}' in the current folder"
    ))?;

    tracing::info!(
        parent: &tracing::Span::none(),
        "The file `{}` with contract code of `{}` was downloaded successfully",
        file_path,
        contract_type,
    );

    Ok(())
}

// Action views contain the code hash, not the deployed Wasm. Converting a view to
// an Action preserves those bytes, so hashing them again would hash the hash.
fn deploy_action_code_hash(
    code: &[u8],
) -> color_eyre::eyre::Result<near_primitives::hash::CryptoHash> {
    near_primitives::hash::CryptoHash::try_from(code).map_err(|err| {
        color_eyre::eyre::eyre!("Invalid code hash in the contract deploy action view: {err}")
    })
}

fn verify_downloaded_code(
    code: &[u8],
    hash_to_match: &near_primitives::hash::CryptoHash,
) -> color_eyre::eyre::Result<near_primitives::hash::CryptoHash> {
    let code_hash = near_primitives::hash::CryptoHash::hash_bytes(code);
    if code_hash != *hash_to_match {
        color_eyre::eyre::bail!(
            "The code hash of the contract deploy action does not match the code that we retrieved from the archive node."
        );
    }
    Ok(code_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_primitives::{
        action::GlobalContractDeployMode, transaction::Action, views::ActionView,
    };

    #[test]
    fn deploy_views_preserve_the_expected_archive_code_hash() {
        let code = b"\0asm\x01\0\0\0";
        let expected_hash = near_primitives::hash::CryptoHash::hash_bytes(code);
        for original_action in [
            Action::DeployContract(near_primitives::transaction::DeployContractAction {
                code: code.to_vec(),
            }),
            Action::DeployGlobalContract(near_primitives::action::DeployGlobalContractAction {
                code: code.to_vec().into(),
                deploy_mode: GlobalContractDeployMode::CodeHash,
            }),
            Action::DeployGlobalContract(near_primitives::action::DeployGlobalContractAction {
                code: code.to_vec().into(),
                deploy_mode: GlobalContractDeployMode::AccountId,
            }),
        ] {
            let archival_action = Action::try_from(ActionView::from(original_action)).unwrap();
            let hash_bytes = match archival_action {
                Action::DeployContract(action) => action.code,
                Action::DeployGlobalContract(action) => action.code.to_vec(),
                _ => unreachable!(),
            };
            let action_hash = deploy_action_code_hash(&hash_bytes).unwrap();
            assert_eq!(action_hash, expected_hash);
            verify_downloaded_code(code, &action_hash).unwrap();
            let err = verify_downloaded_code(b"different contract", &action_hash).unwrap_err();
            assert!(err.to_string().contains("does not match"));
        }
    }

    #[test]
    fn malformed_deploy_hash_is_rejected() {
        for len in [0, 31, 33] {
            let err = deploy_action_code_hash(&vec![0; len]).unwrap_err();
            assert!(err.to_string().contains("Invalid code hash"));
        }
    }

    #[test]
    fn delegated_deployments_are_normalized_to_action_view_hashes() {
        for code in [b"\0asm\x01\0\0\0".to_vec(), vec![42; 32]] {
            let expected_hash = near_primitives::hash::CryptoHash::hash_bytes(&code);
            let deployments = vec![
                Action::DeployContract(near_primitives::transaction::DeployContractAction {
                    code: code.clone(),
                }),
                Action::DeployGlobalContract(near_primitives::action::DeployGlobalContractAction {
                    code: code.clone().into(),
                    deploy_mode: GlobalContractDeployMode::CodeHash,
                }),
                Action::DeployGlobalContract(near_primitives::action::DeployGlobalContractAction {
                    code: code.clone().into(),
                    deploy_mode: GlobalContractDeployMode::AccountId,
                }),
            ];
            let delegate = Action::Delegate(Box::new(
                near_primitives::action::delegate::SignedDelegateAction {
                    delegate_action: near_primitives::action::delegate::DelegateAction {
                        sender_id: "sender.near".parse().unwrap(),
                        receiver_id: "receiver.near".parse().unwrap(),
                        actions: deployments
                            .into_iter()
                            .map(|action| action.try_into().unwrap())
                            .collect(),
                        nonce: 1,
                        max_block_height: 100,
                        public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
                    },
                    signature: near_crypto::Signature::empty(near_crypto::KeyType::ED25519),
                },
            ));
            // Exercise the same outer view conversion and unwrap as RPC reconstruction.
            let mut transaction = crate::commands::PrepopulatedTransaction {
                signer_id: "relayer.near".parse().unwrap(),
                receiver_id: "sender.near".parse().unwrap(),
                actions: vec![Action::try_from(ActionView::from(delegate)).unwrap()],
            };
            unwrap_delegate_action(&mut transaction).unwrap();
            assert_eq!(transaction.signer_id.as_str(), "sender.near");
            assert_eq!(transaction.receiver_id.as_str(), "receiver.near");
            assert_eq!(transaction.actions.len(), 3);
            for action in transaction.actions {
                let hash_bytes = match action {
                    Action::DeployContract(action) => action.code,
                    Action::DeployGlobalContract(action) => action.code.to_vec(),
                    _ => unreachable!(),
                };
                let action_hash = deploy_action_code_hash(&hash_bytes).unwrap();
                assert_eq!(action_hash, expected_hash);
                verify_downloaded_code(&code, &action_hash).unwrap();
                assert!(verify_downloaded_code(b"different contract", &action_hash).is_err());
            }
        }
    }

    fn reconstruct_action(action: Action) -> color_eyre::eyre::Result<Vec<String>> {
        let config = crate::config::Config::default();
        let network_config = config.network_connection.values().next().unwrap();
        Ok(action_transformation(
            action,
            "receiver.near".parse().unwrap(),
            network_config,
            near_primitives::types::BlockReference::Finality(
                near_primitives::types::Finality::Final,
            ),
        )?
        .unwrap()
        .to_cli_args()
        .into_iter()
        .collect())
    }

    #[test]
    fn unsupported_gas_key_actions_return_errors() {
        let public_key = near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519);
        let function_call = near_primitives::account::FunctionCallPermission {
            allowance: None,
            receiver_id: "receiver.near".to_owned(),
            method_names: vec!["method".to_owned()],
        };
        for action in [
            Action::TransferToGasKey(Box::new(near_primitives::action::TransferToGasKeyAction {
                public_key: public_key.clone(),
                deposit: near_token::NearToken::from_yoctonear(1),
            })),
            Action::WithdrawFromGasKey(Box::new(
                near_primitives::action::WithdrawFromGasKeyAction {
                    public_key: public_key.clone(),
                    amount: near_token::NearToken::from_yoctonear(1),
                },
            )),
            Action::AddKey(Box::new(near_primitives::transaction::AddKeyAction {
                public_key: public_key.clone(),
                access_key: near_primitives::account::AccessKey::gas_key_full_access(1),
            })),
            Action::AddKey(Box::new(near_primitives::transaction::AddKeyAction {
                public_key,
                access_key: near_primitives::account::AccessKey::gas_key_function_call(
                    1,
                    function_call,
                ),
            })),
        ] {
            let err = reconstruct_action(action).unwrap_err();
            assert!(err.to_string().contains("Gas key"));
            assert!(err.to_string().contains("not yet supported"));
        }
    }

    #[test]
    fn unhandled_delegate_returns_an_error_instead_of_panicking() {
        let action = Action::Delegate(Box::new(
            near_primitives::action::delegate::SignedDelegateAction {
                delegate_action: near_primitives::action::delegate::DelegateAction {
                    sender_id: "sender.near".parse().unwrap(),
                    receiver_id: "receiver.near".parse().unwrap(),
                    actions: vec![],
                    nonce: 1,
                    max_block_height: 100,
                    public_key: near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519),
                },
                signature: near_crypto::Signature::empty(near_crypto::KeyType::ED25519),
            },
        ));
        let err = reconstruct_action(action).unwrap_err();
        assert!(err.to_string().contains("only action"));
    }

    #[test]
    fn supported_actions_keep_their_command_arguments() {
        let args = reconstruct_action(Action::Transfer(
            near_primitives::transaction::TransferAction {
                deposit: near_token::NearToken::from_near(1),
            },
        ))
        .unwrap();
        assert_eq!(args, ["transfer", "1 NEAR"]);

        let public_key = near_crypto::PublicKey::empty(near_crypto::KeyType::ED25519);
        let args = reconstruct_action(Action::AddKey(Box::new(
            near_primitives::transaction::AddKeyAction {
                public_key: public_key.clone(),
                access_key: near_primitives::account::AccessKey::full_access(),
            },
        )))
        .unwrap();
        assert_eq!(
            args,
            [
                "add-key",
                "grant-full-access",
                "use-manually-provided-public-key",
                &public_key.to_string()
            ]
        );

        let args = reconstruct_action(Action::AddKey(Box::new(
            near_primitives::transaction::AddKeyAction {
                public_key: public_key.clone(),
                access_key: near_primitives::account::AccessKey {
                    nonce: 0,
                    permission: near_primitives::account::AccessKeyPermission::FunctionCall(
                        near_primitives::account::FunctionCallPermission {
                            allowance: None,
                            receiver_id: "receiver.near".to_owned(),
                            method_names: vec!["method".to_owned()],
                        },
                    ),
                },
            },
        )))
        .unwrap();
        assert!(args.contains(&"grant-function-call-access".to_owned()));
        assert!(args.contains(&"receiver.near".to_owned()));
        assert!(args.contains(&"method".to_owned()));
        assert!(args.contains(&public_key.to_string()));
    }
}

#[cfg(test)]
mod archive_rpc_tests;
