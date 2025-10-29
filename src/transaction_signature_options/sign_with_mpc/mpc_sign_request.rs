#[derive(serde::Serialize, Debug, Clone)]
/// Payload for Sign request to MPC
pub enum SignPayload {
    #[serde(with = "hex::serde")]
    Ecdsa([u8; 32]),
    #[serde(with = "hex::serde")]
    Eddsa(Vec<u8>),
}

#[derive(serde::Serialize, Debug, Clone)]
pub struct SignRequest {
    #[serde(rename = "payload_v2")]
    pub payload: SignPayload,
    pub path: String,
    pub domain_id: u64,
}
