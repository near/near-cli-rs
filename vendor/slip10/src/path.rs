use crate::{Error, HARDEND};

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::convert::From;

/// A path structure defined by BIP 32.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BIP32Path(pub(crate) Vec<u32>);

impl core::str::FromStr for BIP32Path {
    type Err = Error;

    /// Create a BIP32Path form string literals.
    fn from_str(path: &str) -> Result<Self, Self::Err> {
        let mut paths = Vec::new();
        let path = path.replace("/", "\n");

        for p in path.lines() {
            if p != "m" {
                if p.ends_with('H') || p.ends_with('\'') {
                    let index: u32 = p[..p.len() - 1].parse().map_err(|_| Error::InvalidIndex)?;
                    if index < HARDEND {
                        paths.push(index + HARDEND);
                    } else {
                        return Err(Error::InvalidIndex);
                    }
                } else {
                    let index: u32 = p.parse().map_err(|_| Error::InvalidIndex)?;
                    if index < HARDEND {
                        paths.push(index);
                    } else {
                        return Err(Error::InvalidIndex);
                    }
                }
            }
        }

        Ok(BIP32Path(paths))
    }
}

impl core::fmt::Display for BIP32Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "m/{}",
            self.0
                .iter()
                .map(|index| {
                    if let Some(index) = index.checked_sub(HARDEND) {
                        format!("{}'", index)
                    } else {
                        format!("{}", index)
                    }
                })
                .collect::<Vec<String>>()
                .join("/")
        )
    }
}

impl BIP32Path {
    /// Return the depth of the BIP32Path. For example, "m/0'/0'" will have depth of 2.
    pub fn depth(&self) -> u8 {
        self.0.len() as u8
    }

    /// Return the index value of corresponding depth in the BIP32Path.
    pub fn index(&self, depth: u8) -> Option<&u32> {
        self.0.get(depth as usize)
    }

    /// Push one index to the BIP32Path.
    pub fn push(&mut self, index: u32) {
        self.0.push(index);
    }

    /// Pop last index of the BIP32Path.
    pub fn pop(&mut self) -> Option<u32> {
        self.0.pop()
    }
}

impl From<Vec<u32>> for BIP32Path {
    fn from(vector: Vec<u32>) -> Self {
        BIP32Path(vector)
    }
}
