#[cfg(feature = "rustcrypto_api")]
pub use cipher::generic_array;

pub use ppv_lite86::Machine;
use ppv_lite86::{vec128_storage, ArithOps, BitOps32, LaneWords4, MultiLane, StoreBytes, Vec4};

pub(crate) const BLOCK: usize = 64;
pub(crate) const BLOCK64: u64 = BLOCK as u64;
const LOG2_BUFBLOCKS: u64 = 2;
const BUFBLOCKS: u64 = 1 << LOG2_BUFBLOCKS;
pub(crate) const BUFSZ64: u64 = BLOCK64 * BUFBLOCKS;
pub(crate) const BUFSZ: usize = BUFSZ64 as usize;

/// Parameters of a ChaCha stream, including fixed parameters and current position.
#[derive(Clone, PartialEq, Eq)]
pub struct ChaCha {
    pub(crate) b: vec128_storage,
    pub(crate) c: vec128_storage,
    pub(crate) d: vec128_storage,
}

/// Working state of a ChaCha stream.
#[derive(Clone, PartialEq, Eq)]
pub struct State<V> {
    pub(crate) a: V,
    pub(crate) b: V,
    pub(crate) c: V,
    pub(crate) d: V,
}

#[inline(always)]
pub(crate) fn round<V: ArithOps + BitOps32>(mut x: State<V>) -> State<V> {
    x.a += x.b;
    x.d = (x.d ^ x.a).rotate_each_word_right16();
    x.c += x.d;
    x.b = (x.b ^ x.c).rotate_each_word_right20();
    x.a += x.b;
    x.d = (x.d ^ x.a).rotate_each_word_right24();
    x.c += x.d;
    x.b = (x.b ^ x.c).rotate_each_word_right25();
    x
}

#[inline(always)]
pub(crate) fn diagonalize<V: LaneWords4>(mut x: State<V>) -> State<V> {
    // Since b has the critical data dependency, avoid rotating b to hide latency.
    //
    // The order of these statements is important for performance on pre-AVX2 Intel machines, which
    // are throughput-bound and operating near their superscalar limits during refill_wide. The
    // permutations here and in undiagonalize have been found in testing on Nehalem to be optimal.
    x.a = x.a.shuffle_lane_words1230();
    x.c = x.c.shuffle_lane_words3012();
    x.d = x.d.shuffle_lane_words2301();
    x
}

#[inline(always)]
pub(crate) fn undiagonalize<V: LaneWords4>(mut x: State<V>) -> State<V> {
    // The order of these statements is magic. See comment in diagonalize.
    x.c = x.c.shuffle_lane_words1230();
    x.d = x.d.shuffle_lane_words2301();
    x.a = x.a.shuffle_lane_words3012();
    x
}

impl ChaCha {
    pub fn new(key: &[u8; 32], nonce: &[u8]) -> Self {
        let ctr_nonce = [
            0,
            if nonce.len() == 12 {
                read_u32le(&nonce[0..4])
            } else {
                0
            },
            read_u32le(&nonce[nonce.len() - 8..nonce.len() - 4]),
            read_u32le(&nonce[nonce.len() - 4..]),
        ];
        let key0 = [
            read_u32le(&key[0..4]),
            read_u32le(&key[4..8]),
            read_u32le(&key[8..12]),
            read_u32le(&key[12..16]),
        ];
        let key1 = [
            read_u32le(&key[16..20]),
            read_u32le(&key[20..24]),
            read_u32le(&key[24..28]),
            read_u32le(&key[28..32]),
        ];

        ChaCha {
            b: key0.into(),
            c: key1.into(),
            d: ctr_nonce.into(),
        }
    }

    #[inline(always)]
    fn pos64<M: Machine>(&self, m: M) -> u64 {
        let d: M::u32x4 = m.unpack(self.d);
        ((d.extract(1) as u64) << 32) | d.extract(0) as u64
    }

    /// Set 64-bit block count, affecting next refill.
    #[inline(always)]
    pub(crate) fn seek64<M: Machine>(&mut self, m: M, blockct: u64) {
        let d: M::u32x4 = m.unpack(self.d);
        self.d = d
            .insert((blockct >> 32) as u32, 1)
            .insert(blockct as u32, 0)
            .into();
    }

    /// Set 32-bit block count, affecting next refill.
    #[inline(always)]
    pub(crate) fn seek32<M: Machine>(&mut self, m: M, blockct: u32) {
        let d: M::u32x4 = m.unpack(self.d);
        self.d = d.insert(blockct, 0).into();
    }

    /// Produce output from the current state.
    #[inline(always)]
    fn output_narrow<M: Machine>(&mut self, m: M, x: State<M::u32x4>, out: &mut [u8; BLOCK]) {
        let k = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        (x.a + k).write_le(&mut out[0..16]);
        (x.b + m.unpack(self.b)).write_le(&mut out[16..32]);
        (x.c + m.unpack(self.c)).write_le(&mut out[32..48]);
        (x.d + m.unpack(self.d)).write_le(&mut out[48..64]);
    }

    /// Add one to the block counter (no overflow check).
    #[inline(always)]
    fn inc_block_ct<M: Machine>(&mut self, m: M) {
        let mut pos = self.pos64(m);
        let d0: M::u32x4 = m.unpack(self.d);
        pos += 1;
        let d1 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
        self.d = d1.into();
    }

    /// Produce 4 blocks of output, advancing the state
    #[inline(always)]
    pub fn refill4(&mut self, drounds: u32, out: &mut [u8; BUFSZ]) {
        refill_wide(self, drounds, out)
    }

    /// Produce a block of output, advancing the state
    #[inline(always)]
    pub fn refill(&mut self, drounds: u32, out: &mut [u8; BLOCK]) {
        refill_narrow(self, drounds, out)
    }

    #[inline(always)]
    pub(crate) fn refill_rounds(&mut self, drounds: u32) -> State<vec128_storage> {
        refill_narrow_rounds(self, drounds)
    }

    #[inline]
    pub fn set_stream_param(&mut self, param: u32, value: u64) {
        let mut d: [u32; 4] = self.d.into();
        let p0 = ((param << 1) | 1) as usize;
        let p1 = (param << 1) as usize;
        d[p0] = (value >> 32) as u32;
        d[p1] = value as u32;
        self.d = d.into();
    }

    #[inline]
    pub fn get_stream_param(&self, param: u32) -> u64 {
        let d: [u32; 4] = self.d.into();
        let p0 = ((param << 1) | 1) as usize;
        let p1 = (param << 1) as usize;
        ((d[p0] as u64) << 32) | d[p1] as u64
    }

    /// Return whether rhs represents the same stream, irrespective of current 32-bit position.
    #[inline]
    pub fn stream32_eq(&self, rhs: &Self) -> bool {
        let self_d: [u32; 4] = self.d.into();
        let rhs_d: [u32; 4] = rhs.d.into();
        self.b == rhs.b
            && self.c == rhs.c
            && self_d[3] == rhs_d[3]
            && self_d[2] == rhs_d[2]
            && self_d[1] == rhs_d[1]
    }

    /// Return whether rhs represents the same stream, irrespective of current 64-bit position.
    #[inline]
    pub fn stream64_eq(&self, rhs: &Self) -> bool {
        let self_d: [u32; 4] = self.d.into();
        let rhs_d: [u32; 4] = rhs.d.into();
        self.b == rhs.b && self.c == rhs.c && self_d[3] == rhs_d[3] && self_d[2] == rhs_d[2]
    }
}

#[inline(always)]
#[allow(clippy::many_single_char_names)]
fn refill_wide_impl<Mach: Machine>(
    m: Mach,
    state: &mut ChaCha,
    drounds: u32,
    out: &mut [u8; BUFSZ],
) {
    let k = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
    let mut pos = state.pos64(m);
    let d0: Mach::u32x4 = m.unpack(state.d);
    pos += 1;
    let d1 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos += 1;
    let d2 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos += 1;
    let d3 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);

    let b = m.unpack(state.b);
    let c = m.unpack(state.c);
    let mut x = State {
        a: Mach::u32x4x4::from_lanes([k, k, k, k]),
        b: Mach::u32x4x4::from_lanes([b, b, b, b]),
        c: Mach::u32x4x4::from_lanes([c, c, c, c]),
        d: m.unpack(Mach::u32x4x4::from_lanes([d0, d1, d2, d3]).into()),
    };
    for _ in 0..drounds {
        x = round(x);
        x = undiagonalize(round(diagonalize(x)));
    }
    let mut pos = state.pos64(m);
    let d0: Mach::u32x4 = m.unpack(state.d);
    pos += 1;
    let d1 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos += 1;
    let d2 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos += 1;
    let d3 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);
    pos += 1;
    let d4 = d0.insert((pos >> 32) as u32, 1).insert(pos as u32, 0);

    let (a, b, c, d) = (
        x.a.to_lanes(),
        x.b.to_lanes(),
        x.c.to_lanes(),
        x.d.to_lanes(),
    );
    let sb = m.unpack(state.b);
    let sc = m.unpack(state.c);
    let sd = [m.unpack(state.d), d1, d2, d3];
    state.d = d4.into();
    let mut words = out.chunks_exact_mut(16);
    for ((((&a, &b), &c), &d), &sd) in a.iter().zip(&b).zip(&c).zip(&d).zip(&sd) {
        (a + k).write_le(words.next().unwrap());
        (b + sb).write_le(words.next().unwrap());
        (c + sc).write_le(words.next().unwrap());
        (d + sd).write_le(words.next().unwrap());
    }
}

dispatch!(m, Mach, {
    fn refill_wide(state: &mut ChaCha, drounds: u32, out: &mut [u8; BUFSZ]) {
        refill_wide_impl(m, state, drounds, out);
    }
});

// Refill the buffer from a single-block round, updating the block count.
dispatch_light128!(m, Mach, {
    fn refill_narrow(state: &mut ChaCha, drounds: u32, out: &mut [u8; BLOCK]) {
        let x = refill_narrow_rounds(state, drounds);
        let x = State {
            a: m.unpack(x.a),
            b: m.unpack(x.b),
            c: m.unpack(x.c),
            d: m.unpack(x.d),
        };
        state.output_narrow(m, x, out);
        state.inc_block_ct(m);
    }
});

// Single-block, rounds-only; shared by try_apply_keystream for tails shorter than BUFSZ
// and XChaCha's setup step.
dispatch!(m, Mach, {
    fn refill_narrow_rounds(state: &mut ChaCha, drounds: u32) -> State<vec128_storage> {
        let k: Mach::u32x4 = m.vec([0x6170_7865, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]);
        let mut x = State {
            a: k,
            b: m.unpack(state.b),
            c: m.unpack(state.c),
            d: m.unpack(state.d),
        };
        for _ in 0..drounds {
            x = round(x);
            x = undiagonalize(round(diagonalize(x)));
        }
        State {
            a: x.a.into(),
            b: x.b.into(),
            c: x.c.into(),
            d: x.d.into(),
        }
    }
});

fn read_u32le(xs: &[u8]) -> u32 {
    assert_eq!(xs.len(), 4);
    u32::from(xs[0]) | (u32::from(xs[1]) << 8) | (u32::from(xs[2]) << 16) | (u32::from(xs[3]) << 24)
}

dispatch_light128!(m, Mach, {
    fn init_chacha_x(key: &[u8; 32], nonce: &[u8; 24], rounds: u32) -> ChaCha {
        let key0: Mach::u32x4 = m.read_le(&key[..16]);
        let key1: Mach::u32x4 = m.read_le(&key[16..]);
        let nonce0: Mach::u32x4 = m.read_le(&nonce[..16]);
        let mut state = ChaCha {
            b: key0.into(),
            c: key1.into(),
            d: nonce0.into(),
        };
        let x = refill_narrow_rounds(&mut state, rounds);
        let ctr_nonce1 = [0, 0, read_u32le(&nonce[16..20]), read_u32le(&nonce[20..24])];
        state.b = x.a;
        state.c = x.d;
        state.d = ctr_nonce1.into();
        state
    }
});

#[cfg(test)]
mod tests {
    use super::*;

    /// Basic check that streamXX_eq is block-count invariant
    #[test]
    fn test_stream_eq() {
        let key = hex!("fa44478c59ca70538e3549096ce8b523232c50d9e8e8d10c203ef6c8d07098a5");
        let nonce = hex!("8d3a0d6d7827c00701020304");
        let mut a = ChaCha::new(&key, &nonce);
        let b = a.clone();
        let mut out = [0u8; BLOCK];
        assert!(a == b);
        assert!(a.stream32_eq(&b));
        assert!(a.stream64_eq(&b));
        a.refill(0, &mut out);
        assert!(a != b);
        assert!(a.stream32_eq(&b));
        assert!(a.stream64_eq(&b));
    }
}
