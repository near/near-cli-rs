// Bitcoin Hashes Library
// Written in 2018 by
//   Andrew Poelstra <apoelstra@wpsoftware.net>
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

//! Macros for serde trait impls, and supporting code

#[cfg(feature = "serde")]
/// Functions used by serde impls of all hashes
pub mod serde_details {
    use Error;

    use core::marker::PhantomData;
    use core::{fmt, ops};
    struct HexVisitor<ValueT>(PhantomData<ValueT>);
    use serde::{de, Serializer, Deserializer};
    use hex::{FromHex, ToHex};

    impl<'de, ValueT> de::Visitor<'de> for HexVisitor<ValueT>
    where
        ValueT: FromHex,
    {
        type Value = ValueT;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("an ASCII hex string")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if let Ok(hex) = ::std::str::from_utf8(v) {
                Self::Value::from_hex(hex).map_err(E::custom)
            } else {
                return Err(E::invalid_value(
                    de::Unexpected::Bytes(v),
                    &self,
                ));
            }
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Self::Value::from_hex(v).map_err(E::custom)
        }
    }

    struct BytesVisitor<ValueT>(PhantomData<ValueT>);

    impl<'de, ValueT> de::Visitor<'de> for BytesVisitor<ValueT>
    where ValueT : SerdeHash {
        type Value = ValueT;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a bytestring")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            SerdeHash::from_slice_delegated(v).map_err(|_| {
                // from_slice only errors on incorrect length
                E::invalid_length(v.len(), &stringify!(N))
            })
        }
    }

    /// Default serialization/deserialization methods
    pub trait SerdeHash
    where
        Self: Sized
            + ToHex
            + FromHex
            + ops::Index<usize, Output = u8>
            + ops::Index<ops::RangeFull, Output = [u8]>
    {
        /// Size, in bits, of the hash
        const N: usize;

        /// helper function to turn a deserialized slice into the correct hash type
        fn from_slice_delegated(sl: &[u8]) -> Result<Self, Error>;

        /// serde serialization
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            if s.is_human_readable() {
                s.serialize_str(&self.to_hex())
            } else {
                s.serialize_bytes(&self[..])
            }
        }

        /// serde deserialization
        fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            if d.is_human_readable() {
                d.deserialize_str(HexVisitor::<Self>(PhantomData))
            } else {
                d.deserialize_bytes(BytesVisitor::<Self>(PhantomData))
            }
        }
    }
}

#[macro_export]
#[cfg(feature = "serde")]
/// Implements `Serialize` and `Deserialize` for a type `$t` which
/// represents a newtype over a byte-slice over length `$len`
macro_rules! serde_impl(
    ($t:ident, $len:expr) => (
        impl $crate::serde_macros::serde_details::SerdeHash for $t {
            const N : usize = $len;
            fn from_slice_delegated(sl: &[u8]) -> Result<Self, $crate::Error> {
                $t::from_slice(sl)
            }
        }

        impl $crate::serde::Serialize for $t {
            fn serialize<S: $crate::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
                $crate::serde_macros::serde_details::SerdeHash::serialize(self, s)
            }
        }

        impl<'de> $crate::serde::Deserialize<'de> for $t {
            fn deserialize<D: $crate::serde::Deserializer<'de>>(d: D) -> Result<$t, D::Error> {
                $crate::serde_macros::serde_details::SerdeHash::deserialize(d)
            }
        }
));

/// Does an "empty" serde implementation for the configuration without serde feature
#[macro_export]
#[cfg(not(feature = "serde"))]
macro_rules! serde_impl(
        ($t:ident, $len:expr) => ()
);
