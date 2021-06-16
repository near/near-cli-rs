use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliTransactionsSigning {
    /// Enter an public key
    TransactionsSigningPublicKey(CliTransactionsSigningAction),
}

#[derive(Debug)]
pub enum TransactionsSigning {
    TransactionsSigningPublicKey(TransactionsSigningAction),
}

impl From<CliTransactionsSigning> for TransactionsSigning {
    fn from(item: CliTransactionsSigning) -> Self {
        match item {
            CliTransactionsSigning::TransactionsSigningPublicKey(
                cli_transactions_signing_action,
            ) => Self::TransactionsSigningPublicKey(cli_transactions_signing_action.into()),
        }
    }
}

impl TransactionsSigning {
    pub fn choose_sign_transactions() -> Self {
        Self::from(CliTransactionsSigning::TransactionsSigningPublicKey(
            Default::default(),
        ))
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        stake: u128,
    ) -> crate::CliResult {
        match self {
            TransactionsSigning::TransactionsSigningPublicKey(transactions_sign_action) => {
                transactions_sign_action
                    .process(
                        prepopulated_unsigned_transaction,
                        network_connection_config,
                        stake,
                    )
                    .await
            }
        }
    }
}

/// данные о получателе транзакции
#[derive(Debug, Default, clap::Clap)]
pub struct CliTransactionsSigningAction {
    transactions_signing_public_key: Option<near_crypto::PublicKey>,
    #[clap(subcommand)]
    sign_option: Option<
        crate::commands::construct_transaction_command::sign_transaction::CliSignTransaction,
    >,
}

#[derive(Debug)]
pub struct TransactionsSigningAction {
    pub transactions_signing_public_key: near_crypto::PublicKey,
    pub sign_option:
        crate::commands::construct_transaction_command::sign_transaction::SignTransaction,
}

impl From<CliTransactionsSigningAction> for TransactionsSigningAction {
    fn from(item: CliTransactionsSigningAction) -> Self {
        let transactions_signing_public_key: near_crypto::PublicKey =
            match item.transactions_signing_public_key {
                Some(cli_transactions_signing_public_key) => cli_transactions_signing_public_key,
                None => TransactionsSigningAction::input_public_key(),
            };
        let sign_option = match item.sign_option {
            Some(cli_sign_transaction) => cli_sign_transaction.into(),
            None => crate::commands::construct_transaction_command::sign_transaction::SignTransaction::choose_sign_option(),
        };
        Self {
            transactions_signing_public_key,
            sign_option,
        }
    }
}

impl TransactionsSigningAction {
    pub fn input_public_key() -> near_crypto::PublicKey {
        Input::new()
            .with_prompt("Enter a public key for this server")
            .interact_text()
            .unwrap()
    }

    pub async fn process(
        self,
        prepopulated_unsigned_transaction: near_primitives::transaction::Transaction,
        network_connection_config: Option<crate::common::ConnectionConfig>,
        stake: u128,
    ) -> crate::CliResult {
        let action = near_primitives::transaction::Action::Stake(
            near_primitives::transaction::StakeAction {
                stake,
                public_key: self.transactions_signing_public_key,
            },
        );
        let mut actions = prepopulated_unsigned_transaction.actions.clone();
        actions.push(action);
        let unsigned_transaction = near_primitives::transaction::Transaction {
            actions,
            ..prepopulated_unsigned_transaction
        };
        match self
            .sign_option
            .process(unsigned_transaction, network_connection_config)
            .await?
        {
            Some(transaction_info) => {
                match transaction_info.status {
                    near_primitives::views::FinalExecutionStatus::NotStarted => {
                        println!("NotStarted")
                    }
                    near_primitives::views::FinalExecutionStatus::Started => println!("Started"),
                    near_primitives::views::FinalExecutionStatus::Failure(tx_execution_error) => {
                        match tx_execution_error {
                            near_primitives::errors::TxExecutionError::ActionError(action_error) => {
                                match action_error.kind {
                                    near_primitives::errors::ActionErrorKind::AccountAlreadyExists{account_id} => {
                                        println!("Error: Create Account action tries to create an account with account ID <{}> which is already exists in the storage.", account_id)
                                    },
                                    near_primitives::errors::ActionErrorKind::AccountDoesNotExist{account_id} => {
                                        println!("Error: TX receiver ID <{}> doesn't exist (but action is not \"Create Account\").", account_id)
                                    },
                                    near_primitives::errors::ActionErrorKind::CreateAccountOnlyByRegistrar{account_id:_, registrar_account_id:_, predecessor_id:_} => {
                                        println!("Error: A top-level account ID can only be created by registrar.")
                                    },
                                    near_primitives::errors::ActionErrorKind::CreateAccountNotAllowed{account_id, predecessor_id} => {
                                        println!("Error: A newly created account <{}> must be under a namespace of the creator account <{}>.", account_id, predecessor_id)
                                    },
                                    near_primitives::errors::ActionErrorKind::ActorNoPermission{account_id:_, actor_id:_} => {
                                        println!("Error: Administrative actions can be proceed only if sender=receiver or the first TX action is a \"Create Account\" action.")
                                    },
                                    near_primitives::errors::ActionErrorKind::DeleteKeyDoesNotExist{account_id, public_key} => {
                                        println!("Error: Account <{}>  tries to remove an access key <{}> that doesn't exist.", account_id, public_key)
                                    },
                                    near_primitives::errors::ActionErrorKind::AddKeyAlreadyExists{account_id, public_key} => {
                                        println!("Error: Public key <{}> is already used for an existing account ID <{}>.", public_key, account_id)
                                    },
                                    near_primitives::errors::ActionErrorKind::DeleteAccountStaking{account_id} => {
                                        println!("Error: Account <{}> is staking and can not be deleted", account_id)
                                    },
                                    near_primitives::errors::ActionErrorKind::LackBalanceForState{account_id, amount} => {
                                        println!("Error: Receipt action can't be completed, because the remaining balance will not be enough to cover storage.\nAn account which needs balance: <{}>\nBalance required to complete an action: <{}>",
                                            account_id,
                                            crate::common::NearBalance::from_yoctonear(amount)
                                        )
                                    },
                                    near_primitives::errors::ActionErrorKind::TriesToUnstake{account_id} => {
                                        println!("Error: Account <{}> is not yet staked, but tries to unstake.", account_id)
                                    },
                                    near_primitives::errors::ActionErrorKind::TriesToStake{account_id, stake, locked:_, balance} => {
                                        println!("Error: Account <{}> doesn't have enough balance ({}) to increase the stake ({}).",
                                        account_id,
                                        crate::common::NearBalance::from_yoctonear(balance),
                                        crate::common::NearBalance::from_yoctonear(stake)
                                        )
                                    },
                                    near_primitives::errors::ActionErrorKind::InsufficientStake{account_id:_, stake:_, minimum_stake} => {
                                        println!("Error: Insufficient stake.\nThe minimum rate must be {}.",
                                            crate::common::NearBalance::from_yoctonear(minimum_stake)
                                        )
                                    },
                                    near_primitives::errors::ActionErrorKind::FunctionCallError(function_call_error_ser) => {
                                        println!("Error: An error occurred during a `FunctionCall` Action, parameter is debug message.\n{:?}", function_call_error_ser)
                                    },
                                    near_primitives::errors::ActionErrorKind::NewReceiptValidationError(receipt_validation_error) => {
                                        println!("Error: Error occurs when a new `ActionReceipt` created by the `FunctionCall` action fails.\n{:?}", receipt_validation_error)
                                    },
                                    near_primitives::errors::ActionErrorKind::OnlyImplicitAccountCreationAllowed{account_id:_} => {
                                        println!("Error: Error occurs when a `CreateAccount` action is called on hex-characters account of length 64.\nSee implicit account creation NEP: https://github.com/nearprotocol/NEPs/pull/71")
                                    },
                                    near_primitives::errors::ActionErrorKind::DeleteAccountWithLargeState{account_id} => {
                                        println!("Error: Delete account <{}> whose state is large is temporarily banned.", account_id)
                                    },
                                }
                            },
                            near_primitives::errors::TxExecutionError::InvalidTxError(invalid_tx_error) => {
                                match invalid_tx_error {
                                    near_primitives::errors::InvalidTxError::InvalidAccessKeyError(invalid_access_key_error) => {
                                        match invalid_access_key_error {
                                            near_primitives::errors::InvalidAccessKeyError::AccessKeyNotFound{account_id, public_key} => {
                                                println!("Error: Public key {} doesn't exist for the account <{}>.", public_key, account_id)
                                            },
                                            near_primitives::errors::InvalidAccessKeyError::ReceiverMismatch{tx_receiver, ak_receiver} => {
                                                println!("Error: Transaction for <{}> doesn't match the access key for <{}>.", tx_receiver, ak_receiver)
                                            },
                                            near_primitives::errors::InvalidAccessKeyError::MethodNameMismatch{method_name} => {
                                                println!("Error: Transaction method name <{}> isn't allowed by the access key.", method_name)
                                            },
                                            near_primitives::errors::InvalidAccessKeyError::RequiresFullAccess => {
                                                println!("Error: Transaction requires a full permission access key.")
                                            },
                                            near_primitives::errors::InvalidAccessKeyError::NotEnoughAllowance{account_id, public_key, allowance, cost} => {
                                                println!("Error: Access Key <{}> for account <{}> does not have enough allowance ({}) to cover transaction cost ({}).",
                                                    public_key,
                                                    account_id,
                                                    crate::common::NearBalance::from_yoctonear(allowance),
                                                    crate::common::NearBalance::from_yoctonear(cost)
                                                )
                                            },
                                            near_primitives::errors::InvalidAccessKeyError::DepositWithFunctionCall => {
                                                println!("Error: Having a deposit with a function call action is not allowed with a function call access key.")
                                            }
                                        }
                                    },
                                    near_primitives::errors::InvalidTxError::InvalidSignerId { signer_id } => {
                                        println!("Error: TX signer ID <{}> is not in a valid format or not satisfy requirements see \"near_runtime_utils::utils::is_valid_account_id\".", signer_id)
                                    },
                                    near_primitives::errors::InvalidTxError::SignerDoesNotExist { signer_id } => {
                                        println!("Error: TX signer ID <{}> is not found in a storage.", signer_id)
                                    },
                                    near_primitives::errors::InvalidTxError::InvalidNonce { tx_nonce, ak_nonce } => {
                                        println!("Error: Transaction nonce ({}) must be account[access_key].nonce ({}) + 1.", tx_nonce, ak_nonce)
                                    },
                                    near_primitives::errors::InvalidTxError::NonceTooLarge { tx_nonce, upper_bound } => {
                                        println!("Error: Transaction nonce ({}) is larger than the upper bound ({}) given by the block height.", tx_nonce, upper_bound)
                                    },
                                    near_primitives::errors::InvalidTxError::InvalidReceiverId { receiver_id } => {
                                        println!("Error: TX receiver ID ({}) is not in a valid format or not satisfy requirements see \"near_runtime_utils::is_valid_account_id\".", receiver_id)
                                    },
                                    near_primitives::errors::InvalidTxError::InvalidSignature => {
                                        println!("Error: TX signature is not valid")
                                    },
                                    near_primitives::errors::InvalidTxError::NotEnoughBalance {signer_id, balance, cost} => {
                                        println!("Error: Account <{}> does not have enough balance ({}) to cover TX cost ({}).",
                                            signer_id,
                                            crate::common::NearBalance::from_yoctonear(balance),
                                            crate::common::NearBalance::from_yoctonear(cost)
                                        )
                                    },
                                    near_primitives::errors::InvalidTxError::LackBalanceForState {signer_id, amount} => {
                                        println!("Error: Signer account <{}> doesn't have enough balance ({}) after transaction.",
                                            signer_id,
                                            crate::common::NearBalance::from_yoctonear(amount)
                                        )
                                    },
                                    near_primitives::errors::InvalidTxError::CostOverflow => {
                                        println!("Error: An integer overflow occurred during transaction cost estimation.")
                                    },
                                    near_primitives::errors::InvalidTxError::InvalidChain => {
                                        println!("Error: Transaction parent block hash doesn't belong to the current chain.")
                                    },
                                    near_primitives::errors::InvalidTxError::Expired => {
                                        println!("Error: Transaction has expired.")
                                    },
                                    near_primitives::errors::InvalidTxError::ActionsValidation(actions_validation_error) => {
                                        match actions_validation_error {
                                            near_primitives::errors::ActionsValidationError::DeleteActionMustBeFinal => {
                                                println!("Error: The delete action must be a final action in transaction.")
                                            },
                                            near_primitives::errors::ActionsValidationError::TotalPrepaidGasExceeded {total_prepaid_gas, limit} => {
                                                println!("Error: The total prepaid gas ({}) for all given actions exceeded the limit ({}).",
                                                total_prepaid_gas,
                                                limit
                                                )
                                            },
                                            near_primitives::errors::ActionsValidationError::TotalNumberOfActionsExceeded {total_number_of_actions, limit} => {
                                                println!("Error: The number of actions ({}) exceeded the given limit ({}).", total_number_of_actions, limit)
                                            },
                                            near_primitives::errors::ActionsValidationError::AddKeyMethodNamesNumberOfBytesExceeded {total_number_of_bytes, limit} => {
                                                println!("Error: The total number of bytes ({}) of the method names exceeded the limit ({}) in a Add Key action.", total_number_of_bytes, limit)
                                            },
                                            near_primitives::errors::ActionsValidationError::AddKeyMethodNameLengthExceeded {length, limit} => {
                                                println!("Error: The length ({}) of some method name exceeded the limit ({}) in a Add Key action.", length, limit)
                                            },
                                            near_primitives::errors::ActionsValidationError::IntegerOverflow => {
                                                println!("Error: Integer overflow during a compute.")
                                            },
                                            near_primitives::errors::ActionsValidationError::InvalidAccountId {account_id} => {
                                                println!("Error: Invalid account ID <{}>.", account_id)
                                            },
                                            near_primitives::errors::ActionsValidationError::ContractSizeExceeded {size, limit} => {
                                                println!("Error: The size ({}) of the contract code exceeded the limit ({}) in a DeployContract action.", size, limit)
                                            },
                                            near_primitives::errors::ActionsValidationError::FunctionCallMethodNameLengthExceeded {length, limit} => {
                                                println!("Error: The length ({}) of the method name exceeded the limit ({}) in a Function Call action.", length, limit)
                                            },
                                            near_primitives::errors::ActionsValidationError::FunctionCallArgumentsLengthExceeded {length, limit} => {
                                                println!("Error: The length ({}) of the arguments exceeded the limit ({}) in a Function Call action.", length, limit)
                                            },
                                            near_primitives::errors::ActionsValidationError::UnsuitableStakingKey {public_key} => {
                                                println!("Error: An attempt to stake with a public key <{}> that is not convertible to ristretto.", public_key)
                                            },
                                            near_primitives::errors::ActionsValidationError::FunctionCallZeroAttachedGas => {
                                                println!("Error: The attached amount of gas in a FunctionCall action has to be a positive number.")
                                            }
                                        }
                                    },
                                }
                            },
                        }
                    }
                    near_primitives::views::FinalExecutionStatus::SuccessValue(_) => {
                        let stake: u128 = if let near_primitives::views::ActionView::Stake {
                            stake,
                            public_key: _,
                        } = transaction_info.transaction.actions[0]
                        {
                            stake
                        } else {
                            0
                        };
                        println!(
                            "\nValidator <{}> has successfully staked {}.",
                            transaction_info.transaction.signer_id,
                            crate::common::NearBalance::from_yoctonear(stake),
                        );
                    }
                }
                println!("\nTransaction Id {id}.\n\nTo see the transaction in the transaction explorer, please open this url in your browser:
                    \nhttps://explorer.testnet.near.org/transactions/{id}\n", id=transaction_info.transaction_outcome.id);
            }
            None => {}
        };
        Ok(())
    }
}
