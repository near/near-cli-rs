//! Commonly used Base58 alphabets.

/// Bitcoin's alphabet as defined in their Base58Check encoding.
///
/// See https://en.bitcoin.it/wiki/Base58Check_encoding#Base58_symbol_chart.
pub const BITCOIN: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Monero's alphabet as defined in this forum post.
///
/// See https://forum.getmonero.org/4/academic-and-technical/221/creating-a-standard-for-physical-coins
pub const MONERO: &[u8; 58] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Ripple's alphabet as defined in their wiki.
///
/// See https://wiki.ripple.com/Encodings
pub const RIPPLE: &[u8; 58] = b"rpshnaf39wBUDNEGHJKLM4PQRST7VWXYZ2bcdeCg65jkm8oFqi1tuvAxyz";

/// Flickr's alphabet for creating short urls from photo ids.
///
/// See https://www.flickr.com/groups/api/discuss/72157616713786392/
pub const FLICKR: &[u8; 58] = b"123456789abcdefghijkmnopqrstuvwxyzABCDEFGHJKLMNPQRSTUVWXYZ";

/// The default alphabet used if none is given. Currently is the
/// [`BITCOIN`](constant.BITCOIN.html) alphabet.
pub const DEFAULT: &[u8; 58] = BITCOIN;

/// Prepared Alpabet for [`EncodeBuilder`](crate::encode::EncodeBuilder) and [`DecodeBuilder`](crate::decode::DecodeBuilder).
#[derive(Clone, Copy)]
#[allow(missing_debug_implementations)]
pub struct Alphabet {
    pub(crate) encode: [u8; 58],
    pub(crate) decode: [u8; 128],
}

impl Alphabet {
    /// Bitcoin's prepared alphabet.
    pub const BITCOIN: &'static Self = &Self::new(BITCOIN);
    /// Monero's prepared alphabet.
    pub const MONERO: &'static Self = &Self::new(MONERO);
    /// Ripple's prepared alphabet.
    pub const RIPPLE: &'static Self = &Self::new(RIPPLE);
    /// Flickr's prepared alphabet.
    pub const FLICKR: &'static Self = &Self::new(FLICKR);
    /// The default prepared alphabet used if none is given. Currently is the
    /// [`Alphabet::Bitcoin`](Alphabet::BITCOIN) alphabet.
    pub const DEFAULT: &'static Self = Self::BITCOIN;

    /// Create prepared alphabet.
    pub const fn new(base: &[u8; 58]) -> Alphabet {
        let encode = [
            base[0], base[1], base[2], base[3], base[4], base[5], base[6], base[7], base[8],
            base[9], base[10], base[11], base[12], base[13], base[14], base[15], base[16],
            base[17], base[18], base[19], base[20], base[21], base[22], base[23], base[24],
            base[25], base[26], base[27], base[28], base[29], base[30], base[31], base[32],
            base[33], base[34], base[35], base[36], base[37], base[38], base[39], base[40],
            base[41], base[42], base[43], base[44], base[45], base[46], base[47], base[48],
            base[49], base[50], base[51], base[52], base[53], base[54], base[55], base[56],
            base[57],
        ];

        let mut decode = [0xFF; 128];
        decode[base[0] as usize] = 0;
        decode[base[1] as usize] = 1;
        decode[base[2] as usize] = 2;
        decode[base[3] as usize] = 3;
        decode[base[4] as usize] = 4;
        decode[base[5] as usize] = 5;
        decode[base[6] as usize] = 6;
        decode[base[7] as usize] = 7;
        decode[base[8] as usize] = 8;
        decode[base[9] as usize] = 9;
        decode[base[10] as usize] = 10;
        decode[base[11] as usize] = 11;
        decode[base[12] as usize] = 12;
        decode[base[13] as usize] = 13;
        decode[base[14] as usize] = 14;
        decode[base[15] as usize] = 15;
        decode[base[16] as usize] = 16;
        decode[base[17] as usize] = 17;
        decode[base[18] as usize] = 18;
        decode[base[19] as usize] = 19;
        decode[base[20] as usize] = 20;
        decode[base[21] as usize] = 21;
        decode[base[22] as usize] = 22;
        decode[base[23] as usize] = 23;
        decode[base[24] as usize] = 24;
        decode[base[25] as usize] = 25;
        decode[base[26] as usize] = 26;
        decode[base[27] as usize] = 27;
        decode[base[28] as usize] = 28;
        decode[base[29] as usize] = 29;
        decode[base[30] as usize] = 30;
        decode[base[31] as usize] = 31;
        decode[base[32] as usize] = 32;
        decode[base[33] as usize] = 33;
        decode[base[34] as usize] = 34;
        decode[base[35] as usize] = 35;
        decode[base[36] as usize] = 36;
        decode[base[37] as usize] = 37;
        decode[base[38] as usize] = 38;
        decode[base[39] as usize] = 39;
        decode[base[40] as usize] = 40;
        decode[base[41] as usize] = 41;
        decode[base[42] as usize] = 42;
        decode[base[43] as usize] = 43;
        decode[base[44] as usize] = 44;
        decode[base[45] as usize] = 45;
        decode[base[46] as usize] = 46;
        decode[base[47] as usize] = 47;
        decode[base[48] as usize] = 48;
        decode[base[49] as usize] = 49;
        decode[base[50] as usize] = 50;
        decode[base[51] as usize] = 51;
        decode[base[52] as usize] = 52;
        decode[base[53] as usize] = 53;
        decode[base[54] as usize] = 54;
        decode[base[55] as usize] = 55;
        decode[base[56] as usize] = 56;
        decode[base[57] as usize] = 57;

        Alphabet { encode, decode }
    }
}

/// `std::borrow::Cow` alternative.
#[allow(variant_size_differences)]
pub(crate) enum AlphabetCow<'a> {
    Borrowed(&'a Alphabet),
    Owned(Alphabet),
}
