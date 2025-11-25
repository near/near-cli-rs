#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
/// Payload for Sign request to MPC
pub enum MpcSignPayload {
    #[serde(with = "hex::serde")]
    Ecdsa([u8; 32]),
    #[serde(with = "hex::serde")]
    Eddsa(Vec<u8>),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct MpcSignRequestArgs {
    #[serde(rename = "payload_v2")]
    pub payload: MpcSignPayload,
    pub path: String,
    pub domain_id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct MpcSignRequest {
    pub request: MpcSignRequestArgs,
}
