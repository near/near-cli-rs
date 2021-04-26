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

    pub async fn process(
        self,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        match self {
            SendTo::Account(sender) => {
                sender
                    .process(selected_server_url)
                    .await
            }
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
        Self {
            sender_account_id,
        }
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

    pub async fn process(
        self,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        let account_id = self.sender_account_id.clone();
        self.get_account_info(account_id.clone(), selected_server_url.clone()).await?;
        self.get_access_key_list(account_id.clone(), selected_server_url.clone()).await?;
        Ok(())
    }

    async fn get_account_info(
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
        let call_access_view = if let near_jsonrpc_primitives::types::query::QueryResponseKind::ViewAccount(
            result,
        ) = query_view_method_response.kind
        {
            result
        } else {
            return Err(color_eyre::Report::msg(format!(
                "Error call result"
            )));
        };
        println!("{:#?}\n", &call_access_view);
        Ok(())
    }

    async fn get_access_key_list(
        &self,
        account_id: String,
        selected_server_url: url::Url,
    ) -> crate::CliResult {
        let query_view_method_response = self
            .rpc_client(&selected_server_url.as_str())
            .query(near_jsonrpc_primitives::types::query::RpcQueryRequest {
                block_reference: near_primitives::types::Finality::Final.into(),
                request: near_primitives::views::QueryRequest::ViewAccessKeyList {
                    account_id,
                },
            })
            .await
            .map_err(|err| {
                color_eyre::Report::msg(format!(
                    "Failed to fetch query for view key list: {:?}",
                    err
                ))
            })?;
        let call_access_key_view = if let near_jsonrpc_primitives::types::query::QueryResponseKind::AccessKeyList(
            result,
        ) = query_view_method_response.kind
        {
            result
        } else {
            return Err(color_eyre::Report::msg(format!(
                "Error call result"
            )));
        };
        println!("{:#?}", &call_access_key_view);
        Ok(())
    }
}
