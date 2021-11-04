// Bitcoin Hashes Library
// Written in 2019 by
//   The rust-bitcoin developers.
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # SHA256t (tagged SHA256)

use core::{cmp, str};
use core::marker::PhantomData;

use sha256;
use Hash as HashTrait;
#[allow(unused)]
use Error;

/// Trait representing a tag that can be used as a context for SHA256t hashes.
pub trait Tag {
    /// Returns a hash engine that is pre-tagged and is ready
    /// to be used for the data.
    fn engine() -> sha256::HashEngine;
}

/// Output of the SHA256t hash function.
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Hash<T: Tag>(
    #[cfg_attr(feature = "schemars", schemars(schema_with="crate::util::json_hex_string::len_32"))]
    [u8; 32],
    #[cfg_attr(feature = "schemars", schemars(skip))]
    PhantomData<T>
);

impl<T: Tag> Copy for Hash<T> {}
impl<T: Tag> Clone for Hash<T> {
    fn clone(&self) -> Self {
        Hash(self.0, self.1)
    }
}
impl<T: Tag> PartialEq for Hash<T> {
    fn eq(&self, other: &Hash<T>) -> bool {
        self.0 == other.0
    }
}
impl<T: Tag> Eq for Hash<T> {}
impl<T: Tag> Default for Hash<T> {
    fn default() -> Self {
        Hash([0; 32], PhantomData)
    }
}
impl<T: Tag> PartialOrd for Hash<T> {
    fn partial_cmp(&self, other: &Hash<T>) -> Option<cmp::Ordering> {
        Some(cmp::Ord::cmp(self, other))
    }
}
impl<T: Tag> Ord for Hash<T> {
    fn cmp(&self, other: &Hash<T>) -> cmp::Ordering {
        cmp::Ord::cmp(&self.0, &other.0)
    }
}
impl<T: Tag> ::core::hash::Hash for Hash<T> {
    fn hash<H: ::core::hash::Hasher>(&self, h: &mut H) {
        self.0.hash(h)
    }
}

impl<T: Tag> str::FromStr for Hash<T> {
    type Err = ::hex::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ::hex::FromHex::from_hex(s)
    }
}

hex_fmt_impl!(Debug, Hash, T:Tag);
hex_fmt_impl!(Display, Hash, T:Tag);
hex_fmt_impl!(LowerHex, Hash, T:Tag);
index_impl!(Hash, T:Tag);
borrow_slice_impl!(Hash, T:Tag);

impl<T: Tag> HashTrait for Hash<T> {
    type Engine = sha256::HashEngine;
    type Inner = [u8; 32];

    fn engine() -> sha256::HashEngine {
        T::engine()
    }

    fn from_engine(e: sha256::HashEngine) -> Hash<T> {
        Hash::from_inner(sha256::Hash::from_engine(e).into_inner())
    }

    const LEN: usize = 32;

    fn from_slice(sl: &[u8]) -> Result<Hash<T>, Error> {
        if sl.len() != 32 {
            Err(Error::InvalidLength(Self::LEN, sl.len()))
        } else {
            let mut ret = [0; 32];
            ret.copy_from_slice(sl);
            Ok(Hash::from_inner(ret))
        }
    }

    // NOTE! If this is changed, please make sure the serde serialization is still correct.
    const DISPLAY_BACKWARD: bool = true;

    fn into_inner(self) -> Self::Inner {
        self.0
    }

    fn as_inner(&self) -> &Self::Inner {
        &self.0
    }

    fn from_inner(inner: Self::Inner) -> Self {
        Hash(inner, PhantomData)
    }
}

/// Macro used to define a newtype tagged hash.
/// It creates two public types:
/// - a sha246t::Tag struct,
/// - a sha256t::Hash type alias.
#[macro_export]
macro_rules! sha256t_hash_newtype {
    ($newtype:ident, $tag:ident, $midstate:ident, $midstate_len:expr, $docs:meta, $reverse: expr) => {
        /// The tag used for [$newtype].
        pub struct $tag;

        impl $crate::sha256t::Tag for $tag {
            fn engine() -> $crate::sha256::HashEngine {
                let midstate = $crate::sha256::Midstate::from_inner($midstate);
                $crate::sha256::HashEngine::from_midstate(midstate, $midstate_len)
            }
        }

        $crate::hash_newtype!($newtype, $crate::sha256t::Hash<$tag>, 32, $docs, $reverse);
    };
}

#[cfg(feature="serde")]
impl<T: Tag> ::serde::Serialize for Hash<T> {
    fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use ::hex::ToHex;
        if s.is_human_readable() {
            s.serialize_str(&self.to_hex())
        } else {
            s.serialize_bytes(&self[..])
        }
    }
}

#[cfg(feature="serde")]
struct HexVisitor<T: Tag>(PhantomData<T>);

#[cfg(feature="serde")]
impl<T: Tag> Default for HexVisitor<T> {
    fn default() -> Self { HexVisitor(PhantomData) }
}

#[cfg(feature="serde")]
impl<'de, T: Tag> ::serde::de::Visitor<'de> for HexVisitor<T> {
    type Value = Hash<T>;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("an ASCII hex string")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: ::serde::de::Error,
    {
        use ::hex::FromHex;
        if let Ok(hex) = ::std::str::from_utf8(v) {
            Hash::<T>::from_hex(hex).map_err(E::custom)
        } else {
            return Err(E::invalid_value(::serde::de::Unexpected::Bytes(v), &self));
        }
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: ::serde::de::Error,
    {
        use ::hex::FromHex;
        Hash::<T>::from_hex(v).map_err(E::custom)
    }
}

#[cfg(feature="serde")]
struct BytesVisitor<T: Tag>(PhantomData<T>);

#[cfg(feature="serde")]
impl<T: Tag> Default for BytesVisitor<T> {
    fn default() -> Self { BytesVisitor(PhantomData) }
}

#[cfg(feature="serde")]
impl<'de, T: Tag> ::serde::de::Visitor<'de> for BytesVisitor<T> {
    type Value = Hash<T>;

    fn expecting(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        formatter.write_str("a bytestring")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: ::serde::de::Error,
    {
        Hash::<T>::from_slice(v).map_err(|_| {
            // from_slice only errors on incorrect length
            E::invalid_length(v.len(), &"32")
        })
    }
}

#[cfg(feature="serde")]
impl<'de, T: Tag> ::serde::Deserialize<'de> for Hash<T> {
    fn deserialize<D: ::serde::Deserializer<'de>>(d: D) -> Result<Hash<T>, D::Error> {
        if d.is_human_readable() {
            d.deserialize_str(HexVisitor::<T>::default())
        } else {
            d.deserialize_bytes(BytesVisitor::<T>::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use ::{Hash, sha256, sha256t};
    use ::hex::ToHex;

    const TEST_MIDSTATE: [u8; 32] = [
       156, 224, 228, 230, 124, 17, 108, 57, 56, 179, 202, 242, 195, 15, 80, 137, 211, 243,
       147, 108, 71, 99, 110, 96, 125, 179, 62, 234, 221, 198, 240, 201,
    ];

    #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
    #[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
    pub struct TestHashTag;

    impl sha256t::Tag for TestHashTag {
        fn engine() -> sha256::HashEngine {
            // The TapRoot TapLeaf midstate.
            let midstate = sha256::Midstate::from_inner(TEST_MIDSTATE);
            sha256::HashEngine::from_midstate(midstate, 64)
        }
    }

    /// A hash tagged with `$name`.
    pub type TestHash = sha256t::Hash<TestHashTag>;

    sha256t_hash_newtype!(NewTypeHash, NewTypeTag, TEST_MIDSTATE, 64, doc="test hash", true);

    #[test]
    fn test_sha256t() {
       assert_eq!(
           TestHash::hash(&[0]).to_hex(),
           "29589d5122ec666ab5b4695070b6debc63881a4f85d88d93ddc90078038213ed"
       );
       assert_eq!(
           NewTypeHash::hash(&[0]).to_hex(),
           "29589d5122ec666ab5b4695070b6debc63881a4f85d88d93ddc90078038213ed"
       );
    }

    #[cfg(all(feature = "schemars",feature = "serde"))]
    #[test]
    fn jsonschema_accurate() {
        static HASH_BYTES: [u8; 32] = [
            0xef, 0x53, 0x7f, 0x25, 0xc8, 0x95, 0xbf, 0xa7,
            0x82, 0x52, 0x65, 0x29, 0xa9, 0xb6, 0x3d, 0x97,
            0xaa, 0x63, 0x15, 0x64, 0xd5, 0xd7, 0x89, 0xc2,
            0xb7, 0x65, 0x44, 0x8c, 0x86, 0x35, 0xfb, 0x6c,
        ];

        let hash = TestHash::from_slice(&HASH_BYTES).expect("right number of bytes");
        let js = serde_json::from_str(&serde_json::to_string(&hash).unwrap()).unwrap();
        let s  = schemars::schema_for! (TestHash);
        let schema = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(jsonschema_valid::Config::from_schema(&schema, None).unwrap().validate(&js).is_ok());
    }
}
