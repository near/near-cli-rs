use dialoguer::Input;

#[derive(Debug, clap::Clap)]
pub enum CliSendTo {
    /// Specify an account
    Account(CliSender),
}

#[derive(Debug)]
pub enum SendTo {
    Account(Sender),
}

impl From<CliSendTo> for SendTo {
    fn from(item: CliSendTo) -> Self {
        match item {
            CliSendTo::Account(cli_sender) => {
                let sender = Sender::from(cli_sender);
                Self::Account(sender)
            }
        }
    }
}

impl SendTo {
    pub fn send_to() -> Self {
        Self::from(CliSendTo::Account(Default::default()))
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        match self {
            SendTo::Account(sender) => sender.process(selected_server_url).await,
        }
    }
}

/// Specify the account to be view
#[derive(Debug, Default, clap::Clap)]
pub struct CliSender {
    pub sender_account_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Sender {
    pub sender_account_id: String,
}

impl From<CliSender> for Sender {
    fn from(item: CliSender) -> Self {
        let sender_account_id: String = match item.sender_account_id {
            Some(cli_sender_account_id) => cli_sender_account_id,
            None => Sender::input_sender_account_id(),
        };
        Self { sender_account_id }
    }
}

impl Sender {
    pub fn input_sender_account_id() -> String {
        println!();
        Input::new()
            .with_prompt("What Account ID do you need to view?")
            .interact_text()
            .unwrap()
    }

    fn rpc_client(&self, selected_server_url: &str) -> near_jsonrpc_client::JsonRpcClient {
        near_jsonrpc_client::new_client(&selected_server_url)
    }

    pub async fn process(self, selected_server_url: url::Url) -> crate::CliResult {
        let account_id = self.sender_account_id.clone();
        self.display_account_info(account_id.clone(), selected_server_url.clone())
            .await?;
        self.display_access_key_list(account_id.clone(), selected_server_url.clone())
            .await?;
        Ok(())
    }

    async fn display_account_info(
        &self,
        account_id: String,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(&selected_server_url.as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccount {
                    account_id: account_id.clone(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view account: {:?}",
                    err
                ))
            })?;
        let account_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };

        println!(
            "Account details for '{}' at block #{} ({})\n\
            Native account balance: {}\n\
            Validator stake: {}\n\
            Storage used by the account: {} bytes",
            account_id,
            query_view_method_response.block_height,
            query_view_method_response.block_hash,
            crate::common::NearBalance::from_yoctonear(account_view.amount),
            crate::common::NearBalance::from_yoctonear(account_view.locked),
            account_view.storage_usage
        );
        if account_view.code_hash == near_primitives::hash::CryptoHash::default() {
            println!("Contract code is not deployed to this account.");
        } else {
            println!(
                "Contract code SHA-256 checksum (hex): {}",
                hex::encode(account_view.code_hash.as_ref())
            );
        }
        Ok(())
    }

    async fn display_access_key_list(
        &self,
        account_id: String,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(&selected_server_url.as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                    account_id: account_id.clone(),
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view key list: {:?}",
                    err
                ))
            })?;
        let access_key_view =
            if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(result) =
                query_view_method_response.kind
            {
                result
            } else {
                return Err(color_eyre::Report::msg(format!("Error call result")));
            };

        println!("Number of access keys: {}", access_key_view.keys.len());
        for (index, access_key) in access_key_view.keys.iter().enumerate() {
            let permissions_message = match &access_key.access_key.permission {
                near_primitives::views::AccessKeyPermissionView::FullAccess => {
                    "full access".to_owned()
                }
                near_primitives::views::AccessKeyPermissionView::FunctionCall {
                    allowance,
                    receiver_id,
                    method_names,
                } => {
                    let allowance_message = match allowance {
                        Some(amount) => format!(
                            "with an allowance of {}",
                            crate::common::NearBalance::from_yoctonear(*amount)
                        ),
                        None => format!("with no limit"),
                    };
                    format!(
                        "only do {:?} function calls on {} {}",
                        method_names, receiver_id, allowance_message
                    )
                }
            };
            println!(
                "{: >4}. {} (nonce: {}) is granted to {}",
                index + 1,
                access_key.public_key,
                access_key.access_key.nonce,
                permissions_message
            );
        }
        Ok(())
    }
}
