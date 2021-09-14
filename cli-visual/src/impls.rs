use near_primitives::{hash::CryptoHash};
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
