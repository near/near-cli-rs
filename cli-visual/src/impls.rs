use near_primitives::{hash::CryptoHash, types::BlockHeight};
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
