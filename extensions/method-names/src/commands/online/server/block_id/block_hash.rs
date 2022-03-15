#[derive(clap::Args)]
pub struct BlockIdHash {
    block_id_hash: near_primitives::hash::CryptoHash,
}