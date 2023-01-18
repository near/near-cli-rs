use std::str::FromStr;

#[derive(Eq, Ord, Hash, Clone, Debug, PartialEq, PartialOrd)]
pub struct AccountId(pub near_primitives::types::AccountId);

impl From<AccountId> for near_primitives::types::AccountId {
    fn from(account_id: AccountId) -> Self {
        account_id.0
    }
}

impl From<near_primitives::types::AccountId> for AccountId {
    fn from(account_id: near_primitives::types::AccountId) -> Self {
        Self(account_id)
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for AccountId {
    type Err = <near_primitives::types::AccountId as std::str::FromStr>::Err;

    fn from_str(account_id: &str) -> Result<Self, Self::Err> {
        let account_id = near_primitives::types::AccountId::from_str(account_id)?;
        Ok(Self(account_id))
    }
}

impl AsRef<str> for AccountId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl interactive_clap::ToCli for AccountId {
    type CliVariant = AccountId;
}

impl AccountId {
    pub fn get_parent_account_id_from_sub_account(self) -> Self {
        let owner_account_id = self.to_string();
        let owner_account_id = owner_account_id.split_once('.').map_or("default", |s| s.1);
        Self::from_str(owner_account_id).unwrap()
    }
}
