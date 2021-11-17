use near_primitives::hash::CryptoHash;
use near_primitives::types::{AccountId, BlockHeight};
use url::Url;

use crate::prompt::{Interactive, PromptMessage};

impl PromptMessage for CryptoHash {
    const MSG: &'static str = "Type the block ID hash";
}

impl Interactive for CryptoHash {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for BlockHeight {
    const MSG: &'static str = "Type the block ID height for this account";
}

impl Interactive for BlockHeight {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for AccountId {
    const MSG: &'static str = "Type account id";
}

impl Interactive for AccountId {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for Url {
    const MSG: &'static str = "What is the RPC endpoint?";
}

impl Interactive for Url {
    fn interactive(self) -> Self {
        self
    }
}
