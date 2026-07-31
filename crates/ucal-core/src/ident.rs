//! Canonical binary (§7.1) and UCID (§7.2).
//!
//! # Two encodings, one ordering property
//!
//! Rule S: lexicographic order equals chronological order for the binary form and
//! for UCID, and **not** for the text forms unless they are zero-padded to a
//! fixed tier width. Both encodings here are fixed-width and big-endian, which is
//! the whole reason that property holds: byte order *is* numeric order, so the
//! encoding is directly usable as a database key or a sort key without a
//! comparator.
//!
//! Rule B additionally forbids length-prefixed, minimal, and varint encodings as
//! canonical forms. All three would break the ordering property, and a minimal
//! encoding would also make the wire format depend on the value's magnitude
//! rather than on the profile — which is failure mode F5.
//!
//! # What UCID is not
//!
//! Rule I is emphatic and this module's documentation repeats it: **UCID contains
//! no randomness.** It is a pure function of the instant, so two events at the
//! same tick receive the same UCID. Worse, §2.4 guarantees that an instant read
//! from a nanosecond clock has at least 21 trailing base-5 zeros, so the low
//! digits are not merely non-random but structurally constrained — consecutive
//! nanoseconds share more than twenty leading characters. UCID MUST NOT be used
//! as a unique identifier for concurrent events, and `ucid_has_no_entropy`
//! measures exactly how badly it would fail if it were.

use core::fmt;

use crate::backend::{TickInt, Ticks, CANONICAL_BYTES};
use crate::error::{Code, Result, TimeError};
use crate::profile::Profile;
use crate::value::Instant;

/// Crockford base-32, in ascending value order.
///
/// `I`, `L`, `O` and `U` are absent: the first three because they are confusable
/// with `1` and `0`, and `U` to avoid accidental obscenity. The alphabet is
/// strictly ascending in ASCII, which is what makes lexicographic order equal
/// numeric order (Rule S) — `alphabet_is_ascii_ascending` checks it rather than
/// trusting it.
pub const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// UCID length in characters (§7.2).
///
/// 52 characters of 5 bits is 260 bits, against a 256-bit value, so the leading
/// character encodes only one significant bit and is always `0` or `1`.
pub const UCID_LEN: usize = 52;

/// UCID is defined only for instants below `2^256` (Rule I).
///
/// That ceiling is about 1.978×10²⁶ years — past the end of the stelliferous era,
/// and 30 orders of magnitude beyond the present epoch, but far short of the
/// profile domain's 2.29×10¹⁰³ years. Outside it, `UCAL-E0031`.
pub const UCID_BITS: u32 = 256;

/// The fixed-width sortable text identifier of an instant (§7.2).
///
/// Stored as ASCII bytes rather than a `String`, so the type is `Copy` and needs
/// no allocator — UCID has to work in the `no_std` builds GE-5 targets.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ucid([u8; UCID_LEN]);

impl Ucid {
    /// The identifier as a string slice. Always 52 uppercase Crockford
    /// characters, with no checksum and no separators.
    pub fn as_str(&self) -> &str {
        // The buffer is only ever filled from CROCKFORD, which is ASCII.
        core::str::from_utf8(&self.0).expect("UCID is ASCII by construction")
    }

    /// The raw ASCII bytes.
    pub fn as_bytes(&self) -> &[u8; UCID_LEN] {
        &self.0
    }

    /// Encode a tick count. `UCAL-E0031` at or above `2^256` (Rule I).
    pub fn from_ticks(t: &Ticks) -> Result<Ucid> {
        if t.bit_len() > UCID_BITS {
            return Err(TimeError::with_context(
                Code::E0031,
                "UCID is defined only below 2^256",
            ));
        }
        let be = t.to_canonical_bytes();
        let mut out = [b'0'; UCID_LEN];
        for (c, slot) in out.iter_mut().enumerate() {
            // Character 0 is the most significant, covering bits 255..259.
            let shift = 5 * (UCID_LEN - 1 - c) as u32;
            let mut v = 0u8;
            for j in 0..5u32 {
                let bit = shift + j;
                if bit < UCID_BITS && bit_at(&be, bit) {
                    v |= 1 << j;
                }
            }
            *slot = CROCKFORD[v as usize];
        }
        Ok(Ucid(out))
    }

    /// Decode to a tick count.
    pub fn to_ticks(&self) -> Result<Ticks> {
        let thirty_two = <Ticks as TickInt>::from_u64(32);
        let mut acc = <Ticks as TickInt>::zero();
        for c in self.0.iter() {
            let d = decode_char(*c)?;
            acc = acc
                .try_mul(&thirty_two)
                .and_then(|v| v.try_add(&<Ticks as TickInt>::from_u64(d as u64)))
                .ok_or(TimeError::new(Code::E0021))?;
        }
        if acc.bit_len() > UCID_BITS {
            return Err(TimeError::with_context(
                Code::E0031,
                "decoded value is at or above 2^256",
            ));
        }
        Ok(acc)
    }

    /// Parse a UCID.
    ///
    /// Accepts the standard Crockford input leniencies — case-insensitive, with
    /// `I`/`L` read as `1` and `O` as `0`, and hyphens ignored — while emitting
    /// only the strict canonical form. Being lenient on input and strict on
    /// output is the right way round: a UCID that has been read aloud, retyped or
    /// line-wrapped should still resolve, but nothing this library writes should
    /// need that forgiveness.
    ///
    /// `UCAL-E0032` on an invalid character or a wrong length.
    pub fn parse(s: &str) -> Result<Ucid> {
        let mut buf = [b'0'; UCID_LEN];
        let mut n = 0usize;
        for c in s.bytes() {
            if c == b'-' {
                continue;
            }
            if n >= UCID_LEN {
                return Err(TimeError::with_context(
                    Code::E0032,
                    "UCID is exactly 52 characters",
                ));
            }
            let d = decode_char(c)?;
            buf[n] = CROCKFORD[d as usize];
            n += 1;
        }
        if n != UCID_LEN {
            return Err(TimeError::with_context(
                Code::E0032,
                "UCID is exactly 52 characters",
            ));
        }
        Ok(Ucid(buf))
    }
}

impl fmt::Display for Ucid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for Ucid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ucid({})", self.as_str())
    }
}

/// Bit `i` of a big-endian byte array, counting `0` as the least significant.
fn bit_at(be: &[u8; CANONICAL_BYTES], i: u32) -> bool {
    let byte = be[CANONICAL_BYTES - 1 - (i / 8) as usize];
    (byte >> (i % 8)) & 1 == 1
}

/// Crockford decoding, with the standard substitutions.
fn decode_char(c: u8) -> Result<u8> {
    let up = c.to_ascii_uppercase();
    match up {
        b'O' => return Ok(0),
        b'I' | b'L' => return Ok(1),
        _ => {}
    }
    CROCKFORD
        .iter()
        .position(|x| *x == up)
        .map(|p| p as u8)
        .ok_or(TimeError::with_context(
            Code::E0032,
            "not a Crockford base-32 character",
        ))
}

impl<P: Profile> Instant<P> {
    /// The instant's UCID (§7.2). `UCAL-E0031` above `2^256` (Rule I).
    ///
    /// Not a unique identifier: see the module documentation and Rule I.
    pub fn to_ucid(&self) -> Result<Ucid> {
        Ucid::from_ticks(self.ticks())
    }

    /// Recover an instant from its UCID.
    pub fn from_ucid(u: &Ucid) -> Result<Self> {
        Self::from_ticks(u.to_ticks()?)
    }

    /// Decode the canonical binary form from a slice, checking the width.
    ///
    /// `UCAL-E0030` if the slice is not exactly 64 bytes. The array-typed
    /// [`Instant::from_bytes`] makes that unrepresentable; this exists for the
    /// boundary where a length arrives from outside the type system — a socket, a
    /// column, a file — which is precisely where Rule B's fixed width needs
    /// enforcing rather than assuming.
    pub fn from_bytes_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != CANONICAL_BYTES {
            return Err(TimeError::with_context(
                Code::E0030,
                "canonical binary is exactly 64 bytes (Rule B)",
            ));
        }
        let mut buf = [0u8; CANONICAL_BYTES];
        buf.copy_from_slice(bytes);
        Self::from_bytes(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::UC1;
    use alloc::string::String;
    use alloc::vec::Vec;

    type I = Instant<UC1>;

    fn at(n: u64) -> I {
        I::from_u64(n).unwrap()
    }

    struct Rng(u64);
    impl Rng {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        /// A value below 2^256, i.e. inside the UCID range.
        fn next_ucid_range(&mut self) -> Ticks {
            let mut bytes = [0u8; CANONICAL_BYTES];
            for chunk in bytes[32..].chunks_mut(8) {
                chunk.copy_from_slice(&self.next_u64().to_be_bytes());
            }
            <Ticks as TickInt>::from_canonical_bytes(&bytes).unwrap()
        }
        /// A value anywhere in the 512-bit domain.
        fn next_domain(&mut self) -> Ticks {
            let mut bytes = [0u8; CANONICAL_BYTES];
            for chunk in bytes.chunks_mut(8) {
                chunk.copy_from_slice(&self.next_u64().to_be_bytes());
            }
            <Ticks as TickInt>::from_canonical_bytes(&bytes).unwrap()
        }
    }

    // ---- the alphabet ----

    #[test]
    fn alphabet_is_ascii_ascending() {
        // This is the load-bearing property behind Rule S for UCID: if the
        // alphabet were not monotonic in ASCII, lexicographic order would not be
        // numeric order and the identifier would silently stop being sortable.
        assert_eq!(CROCKFORD.len(), 32);
        for w in CROCKFORD.windows(2) {
            assert!(w[0] < w[1], "alphabet not ascending at {:?}", w);
        }
        // The confusable and unfortunate letters are absent.
        for c in [b'I', b'L', b'O', b'U'] {
            assert!(!CROCKFORD.contains(&c), "{} must be excluded", c as char);
        }
        assert!(CROCKFORD.iter().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    // ---- Rule I: range ----

    #[test]
    fn ucid_range_boundary_is_two_to_the_256() {
        let max = <Ticks as TickInt>::pow2(UCID_BITS)
            .unwrap()
            .try_sub(&<Ticks as TickInt>::one())
            .unwrap();
        assert_eq!(max.bit_len(), 256);
        let inside = I::from_ticks(max).unwrap();
        assert!(inside.to_ucid().is_ok());

        let boundary = <Ticks as TickInt>::pow2(UCID_BITS).unwrap();
        assert_eq!(boundary.bit_len(), 257);
        let outside = I::from_ticks(boundary).unwrap();
        assert_eq!(outside.to_ucid().unwrap_err().code, Code::E0031);
        assert_eq!(
            outside.to_ucid().unwrap_err().code.exit_code(),
            3,
            "a domain error"
        );

        // ...and well above it.
        let far = I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap();
        assert_eq!(far.to_ucid().unwrap_err().code, Code::E0031);
    }

    #[test]
    fn leading_character_is_always_zero_or_one() {
        // 52 characters of 5 bits is 260 bits against a 256-bit value, so the
        // leading character carries exactly one significant bit.
        let mut rng = Rng(0xA11CE);
        for _ in 0..256 {
            let u = Ucid::from_ticks(&rng.next_ucid_range()).unwrap();
            let c = u.as_bytes()[0];
            assert!(c == b'0' || c == b'1', "leading char was {}", c as char);
        }
    }

    // ---- round trips ----

    #[test]
    fn ucid_round_trips() {
        let mut vals: Vec<Ticks> = alloc::vec![
            <Ticks as TickInt>::zero(),
            <Ticks as TickInt>::one(),
            UC1::origin_offset(),
            UC1::beat(),
        ];
        let mut rng = Rng(0xB0B);
        for _ in 0..512 {
            vals.push(rng.next_ucid_range());
        }
        for v in vals {
            let inst = I::from_ticks(v.clone()).unwrap();
            let u = inst.to_ucid().unwrap();
            assert_eq!(u.as_str().len(), UCID_LEN);
            assert_eq!(u.to_ticks().unwrap(), v);
            assert_eq!(I::from_ucid(&u).unwrap(), inst);
            // Text round trip too.
            assert_eq!(Ucid::parse(u.as_str()).unwrap(), u);
        }
    }

    #[test]
    fn zero_is_all_zeros() {
        let u = I::zero().to_ucid().unwrap();
        assert_eq!(u.as_str(), "0".repeat(UCID_LEN));
    }

    #[test]
    fn reproduces_appendix_c_ucids() {
        // The UC-P0 harness already checks these against the RFC; repeating two
        // here keeps the library honest independently of the harness.
        let cases = [
            (
                "8070204002895596515944343085635637180530466139316558837890625",
                "0000000000050PM5TBHF4BFKRZC1KVN566SZGWG5DZ0SSBM29FJ1",
            ),
            (
                "8070205189123984864657505252035637180530466139316558837890625",
                "0000000000050PM6K45HH4YGQJ6SEDGDDZ1NKFHD32F2XBM29FJ1",
            ),
            (
                "222432546681680327568000000000000000000000000000000000000",
                "000000000000004H4KEWEGEB5M995XKBZHX3425VFFD900000000",
            ),
        ];
        for (ticks, want) in cases {
            let t = <Ticks as TickInt>::from_dec_str(ticks).unwrap();
            assert_eq!(I::from_ticks(t).unwrap().to_ucid().unwrap().as_str(), want);
        }
    }

    // ---- parsing leniency and strictness ----

    #[test]
    fn parse_accepts_crockford_leniencies() {
        let u = I::from_ticks(UC1::origin_offset()).unwrap().to_ucid().unwrap();
        let canonical = String::from(u.as_str());

        // Lowercase.
        assert_eq!(Ucid::parse(&canonical.to_lowercase()).unwrap(), u);
        // Hyphens for readability are ignored.
        let hyphenated = {
            let mut s = String::new();
            for (i, c) in canonical.chars().enumerate() {
                if i > 0 && i % 4 == 0 {
                    s.push('-');
                }
                s.push(c);
            }
            s
        };
        assert_eq!(Ucid::parse(&hyphenated).unwrap(), u);
        // I, L and O substitute for 1, 1 and 0.
        let substituted = canonical.replace('0', "O").replace('1', "L");
        assert_eq!(Ucid::parse(&substituted).unwrap(), u);
    }

    #[test]
    fn parse_rejects_bad_input() {
        let good = I::zero().to_ucid().unwrap();
        let s = String::from(good.as_str());
        // Wrong length, both ways.
        assert_eq!(Ucid::parse(&s[..51]).unwrap_err().code, Code::E0032);
        let mut long = s.clone();
        long.push('0');
        assert_eq!(Ucid::parse(&long).unwrap_err().code, Code::E0032);
        // U is excluded from the alphabet and is not a substitution.
        let bad = alloc::format!("U{}", &s[1..]);
        assert_eq!(Ucid::parse(&bad).unwrap_err().code, Code::E0032);
        // Non-alphabet characters.
        for c in ['!', ' ', '\u{00B7}'] {
            let bad = alloc::format!("{c}{}", &s[1..]);
            assert_eq!(Ucid::parse(&bad).unwrap_err().code, Code::E0032);
        }
        // A syntactically valid 52-char string can still exceed 2^256.
        let too_big = "Z".repeat(UCID_LEN);
        let parsed = Ucid::parse(&too_big).unwrap();
        assert_eq!(parsed.to_ticks().unwrap_err().code, Code::E0031);
    }

    // ---- Rule S: the ordering proofs, fuzzed ----

    #[test]
    fn binary_order_equals_numeric_order_fuzzed() {
        let mut rng = Rng(0xD1CE_0001);
        let mut vals: Vec<I> = (0..2048)
            .map(|_| I::from_ticks(rng.next_domain()).unwrap())
            .collect();
        // Include the extremes and some near-boundary values.
        vals.push(I::zero());
        vals.push(at(1));
        vals.push(I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap());
        vals.push(I::from_ticks(UC1::origin_offset()).unwrap());
        vals.sort();

        let encoded: Vec<[u8; CANONICAL_BYTES]> = vals.iter().map(|v| v.to_bytes()).collect();
        let mut resorted = encoded.clone();
        resorted.sort();
        assert_eq!(resorted, encoded, "byte order diverges from numeric order");

        // And pairwise, which catches an ordering bug the sort could mask.
        for w in vals.windows(2) {
            assert_eq!(
                w[0].cmp(&w[1]),
                w[0].to_bytes().cmp(&w[1].to_bytes()),
                "pairwise order disagrees"
            );
        }
    }

    #[test]
    fn ucid_order_equals_numeric_order_fuzzed() {
        let mut rng = Rng(0xD1CE_0002);
        let mut vals: Vec<I> = (0..2048)
            .map(|_| I::from_ticks(rng.next_ucid_range()).unwrap())
            .collect();
        vals.push(I::zero());
        vals.push(at(1));
        vals.push(I::from_ticks(UC1::origin_offset()).unwrap());
        vals.sort();

        let ids: Vec<Ucid> = vals.iter().map(|v| v.to_ucid().unwrap()).collect();
        let strings: Vec<&str> = ids.iter().map(|u| u.as_str()).collect();
        let mut resorted = strings.clone();
        resorted.sort();
        assert_eq!(resorted, strings, "UCID order diverges from numeric order");

        for w in vals.windows(2) {
            let (a, b) = (w[0].to_ucid().unwrap(), w[1].to_ucid().unwrap());
            assert_eq!(w[0].cmp(&w[1]), a.as_str().cmp(b.as_str()));
        }
    }

    // ---- Rule I: UCID is not an identifier ----

    #[test]
    fn ucid_has_no_entropy() {
        // Rule I says UCID "contains no randomness" and must not be used to
        // identify concurrent events. Two separate claims, both checkable.

        // 1. It is a pure function of the instant, so concurrent events collide.
        let a = at(1_234_567);
        let b = at(1_234_567);
        assert_eq!(a.to_ucid().unwrap(), b.to_ucid().unwrap());

        // 2. The low digits are structurally constrained, not merely non-random.
        //    §2.4 guarantees a nanosecond-clock reading has at least 21 trailing
        //    base-5 zeros, so successive nanoseconds are a tiny step across the
        //    2^256 range and share a long prefix.
        let ns = UC1::bridge()
            .ticks
            .quot_rem(&<Ticks as TickInt>::from_u64(1_000_000_000))
            .0;
        let base = UC1::origin_offset();
        // Named `earlier`/`later` rather than `first`/`second`: in a time library
        // an ordinal `second` is genuinely ambiguous, which is exactly what the
        // foreign-unit lint objects to.
        let earlier = I::from_ticks(base.clone()).unwrap().to_ucid().unwrap();
        let later = I::from_ticks(base.try_add(&ns).unwrap())
            .unwrap()
            .to_ucid()
            .unwrap();
        assert_ne!(earlier, later);

        let shared = earlier
            .as_bytes()
            .iter()
            .zip(later.as_bytes().iter())
            .take_while(|(x, y)| x == y)
            .count();
        assert!(
            shared >= 20,
            "consecutive nanoseconds shared only {shared} of {UCID_LEN} characters; \
             the entropy claim in Rule I depends on this being large"
        );

        // 3. Present-epoch instants all share a long zero prefix, because the
        //    epoch occupies a narrow band of the UCID range.
        assert!(earlier.as_str().starts_with("00000000000"));
    }

    // ---- Rule B: the binary form ----

    #[test]
    fn binary_form_is_fixed_width_and_not_minimal() {
        // Rule B forbids minimal encodings: a small value must still occupy the
        // full 64 bytes, or the width would depend on the value (F5).
        assert_eq!(at(1).to_bytes().len(), CANONICAL_BYTES);
        assert_eq!(at(1).to_bytes()[..63], [0u8; 63]);
        assert_eq!(at(1).to_bytes()[63], 1);
        assert_eq!(I::zero().to_bytes(), [0u8; CANONICAL_BYTES]);
        assert_eq!(
            I::from_ticks(<Ticks as TickInt>::domain_max())
                .unwrap()
                .to_bytes(),
            [0xffu8; CANONICAL_BYTES]
        );
    }

    #[test]
    fn binary_slice_decoding_checks_the_width() {
        let v = at(42);
        let b = v.to_bytes();
        assert_eq!(I::from_bytes_slice(&b).unwrap(), v);
        assert_eq!(
            I::from_bytes_slice(&b[..63]).unwrap_err().code,
            Code::E0030
        );
        let mut too_long = alloc::vec::Vec::from(&b[..]);
        too_long.push(0);
        assert_eq!(
            I::from_bytes_slice(&too_long).unwrap_err().code,
            Code::E0030
        );
        assert_eq!(I::from_bytes_slice(&[]).unwrap_err().code, Code::E0030);
    }

    #[test]
    fn binary_round_trips_over_the_domain() {
        let mut rng = Rng(0xFEED_BEEF);
        for _ in 0..1024 {
            let v = I::from_ticks(rng.next_domain()).unwrap();
            assert_eq!(I::from_bytes(&v.to_bytes()).unwrap(), v);
        }
    }

    #[test]
    fn ucid_and_binary_agree_on_the_low_256_bits() {
        // The two encodings are of the same number, so a UCID-range value's
        // binary form must have 32 zero bytes on the left.
        let mut rng = Rng(0x5A5A);
        for _ in 0..128 {
            let v = I::from_ticks(rng.next_ucid_range()).unwrap();
            assert_eq!(&v.to_bytes()[..32], &[0u8; 32]);
            assert_eq!(v.to_ucid().unwrap().to_ticks().unwrap(), *v.ticks());
        }
    }
}
