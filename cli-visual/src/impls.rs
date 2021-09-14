use near_primitives::{hash::CryptoHash, types::BlockHeight};
use crate::prompt::{PromptMessage, Interactive};

impl PromptMessage for CryptoHash {
    fn prompt_msg() -> String {
        "Type the block ID hash".to_string()
    }
}

impl Interactive<Self> for CryptoHash {
    fn interactive(self) -> Self {
        self
    }
}

impl PromptMessage for BlockHeight {
    fn prompt_msg() -> String {
        "Type the block ID height for this account".to_string()
    }
}

impl Interactive<Self> for BlockHeight {
    fn interactive(self) -> Self {
        self
    }
}
