use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocialDb {
    #[serde(flatten)]
    pub accounts: HashMap<near_primitives::types::AccountId, AccountProfile>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccountProfile {
    pub profile: Profile,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ProfileImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_image: Option<ProfileImage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linktree: Option<HashMap<String, Option<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<HashMap<String, Option<String>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProfileImage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipfs_cid: Option<String>,
}
