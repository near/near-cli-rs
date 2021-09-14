use near_primitives::hash::CryptoHash;
use near_primitives::types::{BlockHeight, AccountId,};
use url::Url;

use crate::prompt::{PromptMessage, Interactive};

impl PromptMessage for CryptoHash {
    const MSG: &'static str = "Type the block ID hash";
}

impl Interactive<Self> for CryptoHash {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for BlockHeight {
    const MSG: &'static str = "Type the block ID height for this account";
}

impl Interactive<Self> for BlockHeight {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for AccountId {
    const MSG: &'static str = "Type account id";
}

impl Interactive<Self> for AccountId {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for Url {
    const MSG: &'static str = "What is the RPC endpoint?";
}

impl Interactive<Self> for Url {
    fn interactive(self) -> Self {
        self
    }
}
