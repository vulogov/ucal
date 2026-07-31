//! Derivation route A: fixed-width `bnum::U512`.
//!
//! Deliberately algorithmically distinct from route B at every step:
//!
//! | step | route A (here) | route B |
//! |---|---|---|
//! | `5^e` | binary exponentiation (`pow`) | repeated multiply-by-5 |
//! | half-even round | compare `2r` against `d` | tie test via `cmp` on doubled remainder in `BigRational` |
//! | base-5 groups | `divmod` by 3125 (Appendix F) | full base-5 digit vector, then chunked |
//! | UCID | 5-bit extraction from the 64-byte BE array | `divmod` by 32 |
//! | day count | Hinnant era algorithm | cumulative month-table loop |
//! | rationals | hand-rolled `(U512, U512)` with Euclid gcd | `num_rational::BigRational` |
//! | v5 valuation | repeated division by 5 | trailing zeros of the base-5 digit vector |

use bnum::types::U512;

use crate::declared as d;
use crate::derivation::{CfTable, Derivation, Fixture, SignedDec};

// ---------------------------------------------------------------- primitives

const fn konst(s: &str) -> U512 {
    match U512::from_str_radix(s, 10) {
        Ok(v) => v,
        Err(_) => panic!("invalid decimal literal in const context"),
    }
}

/// §3.3 requires the profile constants to be `const` on the default backend.
/// These four lines are the proof that they can be.
const BEAT: U512 = konst("867361737988403547205962240695953369140625");
const SECOND: U512 = konst("18548584399861000000000000000000000000000000");
const ORIGIN_OFFSET: U512 =
    konst("8070204002895596515944343085635637180530466139316558837890625");
const BIG_BANG_CLAIM_HALFWIDTH: U512 =
    konst("11706976141141069872000000000000000000000000000000000000000");

const ZERO: U512 = U512::MIN;

fn dec(s: &str) -> U512 {
    U512::from_str_radix(s, 10).expect("invalid decimal")
}

fn small(n: u64) -> U512 {
    dec(&n.to_string())
}

fn one() -> U512 {
    small(1)
}

fn five() -> U512 {
    small(5)
}

fn bits(x: &U512) -> u32 {
    d::appendix_a::DOMAIN_BITS - x.leading_zeros()
}

/// `5^e` by binary exponentiation.
fn pow5(e: u32) -> U512 {
    five().pow(e)
}

fn pow10(e: u32) -> U512 {
    small(10).pow(e)
}

/// Base-5 valuation: how many trailing base-5 digits are zero.
fn v5(x: &U512) -> u32 {
    if *x == ZERO {
        return 0;
    }
    let mut n = *x;
    let mut k = 0u32;
    let f = five();
    while n % f == ZERO {
        n = n / f;
        k += 1;
    }
    k
}

/// Half-even division, the rounding mode §2.2 declares for the datum.
fn div_half_even(n: &U512, den: &U512) -> U512 {
    let q = *n / *den;
    let r = *n % *den;
    let two_r = r * small(2);
    if two_r > *den {
        q + one()
    } else if two_r == *den && (q % small(2)) == one() {
        q + one()
    } else {
        q
    }
}

// ------------------------------------------------------------------ rationals

/// Exact non-negative rational over `U512`, hand-rolled with Euclid gcd.
#[derive(Clone, PartialEq, Eq, Debug)]
struct Rat {
    num: U512,
    den: U512,
}

impl Rat {
    fn new(num: U512, den: U512) -> Self {
        assert!(den != ZERO, "zero denominator");
        let mut r = Rat { num, den };
        r.reduce();
        r
    }

    fn from_u(n: U512) -> Self {
        Rat { num: n, den: one() }
    }

    fn reduce(&mut self) {
        let g = gcd(self.num, self.den);
        if g != ZERO && g != one() {
            self.num = self.num / g;
            self.den = self.den / g;
        }
    }

    fn mul(&self, o: &Rat) -> Rat {
        Rat::new(self.num * o.num, self.den * o.den)
    }

    fn div(&self, o: &Rat) -> Rat {
        assert!(o.num != ZERO, "division by zero rational");
        Rat::new(self.num * o.den, self.den * o.num)
    }

    /// `|self - o|`
    fn abs_diff(&self, o: &Rat) -> Rat {
        let a = self.num * o.den;
        let b = o.num * self.den;
        let n = if a >= b { a - b } else { b - a };
        Rat::new(n, self.den * o.den)
    }

    fn recip(&self) -> Rat {
        Rat::new(self.den, self.num)
    }

    fn is_zero(&self) -> bool {
        self.num == ZERO
    }

    /// Render to `digits` fractional places, half-even (Rule R: mode stated).
    fn render(&self, digits: u32) -> String {
        let scaled = self.num * pow10(digits);
        let q = div_half_even(&scaled, &self.den);
        let s = q.to_str_radix(10);
        if digits == 0 {
            return s;
        }
        let dg = digits as usize;
        if s.len() <= dg {
            format!("0.{}{}", "0".repeat(dg - s.len()), s)
        } else {
            format!("{}.{}", &s[..s.len() - dg], &s[s.len() - dg..])
        }
    }

    fn render_ratio(&self) -> String {
        format!("{}/{}", self.num.to_str_radix(10), self.den.to_str_radix(10))
    }
}

fn gcd(mut a: U512, mut b: U512) -> U512 {
    while b != ZERO {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Parse an exact decimal such as `"365.242190"` into a rational.
fn parse_decimal(s: &str) -> Rat {
    match s.split_once('.') {
        None => Rat::from_u(dec(s)),
        Some((int, frac)) => {
            let scale = frac.len() as u32;
            let digits = format!("{int}{frac}");
            Rat::new(dec(&digits), pow10(scale))
        }
    }
}

// ------------------------------------------------------- continued fractions

/// Euclid on `num/den`. Exact; returns the full sequence (Appendix H.5).
fn cf_expand(r: &Rat, max_depth: u32) -> Vec<u64> {
    let mut out = Vec::new();
    let mut n = r.num;
    let mut den = r.den;
    for _ in 0..max_depth {
        let a = n / den;
        let rem = n % den;
        out.push(
            a.to_str_radix(10)
                .parse::<u64>()
                .expect("cf term exceeds u64"),
        );
        if rem == ZERO {
            break;
        }
        n = den;
        den = rem;
    }
    out
}

fn convergents(cf: &[u64]) -> Vec<Rat> {
    let (mut hm1, mut hm2) = (one(), ZERO);
    let (mut km1, mut km2) = (ZERO, one());
    let mut out = Vec::new();
    for &a in cf {
        let a = small(a);
        let h = a * hm1 + hm2;
        let k = a * km1 + km2;
        out.push(Rat::new(h, k));
        hm2 = hm1;
        hm1 = h;
        km2 = km1;
        km1 = k;
    }
    out
}

// ------------------------------------------------------------------- codecs

const GROUP_BASE: u64 = 3125;

fn tier(k: i8) -> U512 {
    let e = 60i32 + 5 * k as i32;
    assert!(e >= 0, "tier below T-12");
    pow5(e as u32)
}

/// Decimal group values for tiers `k_hi` down to `k_lo`, most significant first.
///
/// Appendix F: repeated `divmod` by `5^5`, one group per step.
fn groups(t: &U512, k_hi: i8, k_lo: i8) -> Vec<u16> {
    let gb = small(GROUP_BASE);
    let mut x = *t / tier(k_lo);
    let n = (k_hi - k_lo + 1) as usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let r = x % gb;
        out.push(
            r.to_str_radix(10)
                .parse::<u16>()
                .expect("group out of range"),
        );
        x = x / gb;
    }
    out.reverse();
    out
}

fn digit5_of_group(g: u16) -> String {
    let mut s = String::with_capacity(5);
    for i in (0..5).rev() {
        let p = 5u16.pow(i);
        s.push(char::from(b'0' + ((g / p) % 5) as u8));
    }
    s
}

fn render_groups(gs: &[u16], sep: &str) -> String {
    gs.iter()
        .map(|g| format!("{g:04}"))
        .collect::<Vec<_>>()
        .join(sep)
}

fn render_digit5(gs: &[u16]) -> String {
    gs.iter()
        .map(|g| digit5_of_group(*g))
        .collect::<Vec<_>>()
        .join(".")
}

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// UCID (§7.2): 52 Crockford base-32 chars over the 256-bit big-endian value.
/// Extracted bitwise from the canonical 64-byte array — no big-integer division.
fn ucid(t: &U512) -> Option<String> {
    if bits(t) > 256 {
        return None; // UCAL-E0031
    }
    let be = t.to_be_bytes();
    // bit i, counting 0 = least significant of the whole 512-bit value
    let bit = |i: u32| -> u32 {
        let byte = be[be.len() - 1 - (i / 8) as usize];
        ((byte >> (i % 8)) & 1) as u32
    };
    let mut s = String::with_capacity(52);
    for ch in 0..52u32 {
        let shift = 5 * (51 - ch);
        let mut v = 0u32;
        for j in 0..5u32 {
            if shift + j < 256 {
                v |= bit(shift + j) << j;
            }
        }
        s.push(char::from(CROCKFORD[v as usize]));
    }
    Some(s)
}

/// Canonical binary (§7.1, Rule B): 64 bytes, big-endian, zero-padded.
fn bytes_hex(t: &U512) -> String {
    let be = t.to_be_bytes();
    assert_eq!(be.len(), 64, "Rule B requires exactly 64 bytes");
    be.iter().map(|b| format!("{b:02x}")).collect()
}

// ------------------------------------------------------------ civil day count

/// Days from `0000-01-01` in the proleptic Gregorian calendar with
/// astronomical year numbering (§2.5). Hinnant's era algorithm.
fn days_from_civil(y: i64, m: u8, day: u8) -> i64 {
    let y2 = y - if m <= 2 { 1 } else { 0 };
    let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
    let yoe = y2 - era * 400;
    let mp = (m as i64 + if m > 2 { -3 } else { 9 }) as i64;
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468 + 719528
}

/// `ORIGIN_OFFSET + signed_seconds x SECOND`, failing rather than wrapping.
fn instant_from_si_offset(signed_seconds: i64) -> U512 {
    let mag = small(signed_seconds.unsigned_abs()) * SECOND;
    if signed_seconds < 0 {
        ORIGIN_OFFSET
            .checked_sub(mag)
            .expect("UCAL-E0020: result precedes the datum")
    } else {
        ORIGIN_OFFSET
            .checked_add(mag)
            .expect("UCAL-E0021: result exceeds DOMAIN")
    }
}

// ----------------------------------------------------------------- fixtures

fn build_fixture(name: &str, t: U512) -> Fixture {
    let val = v5(&t);
    let beat_part = render_groups(&groups(&t, 5, 0), "·");

    // Tick-exact form: descend to the lowest tier holding a non-zero digit.
    // v5 = 30 puts the lowest non-zero digit at index 30, i.e. tier -6.
    let lowest = if t == ZERO {
        None
    } else {
        let k = (val as i32 - 60).div_euclid(5);
        Some(k.clamp(-12, 32) as i8)
    };
    let human_exact = match lowest {
        Some(k) if k < 0 => {
            format!(
                "UC1 {beat_part}:{}",
                render_groups(&groups(&t, -1, k), "·")
            )
        }
        _ => format!("UC1 {beat_part}"),
    };

    // Appendix C's truncation to T-5, and the window it denotes (Rule T).
    let human_trunc_t5 = if t == ZERO {
        format!("UC1 {beat_part}")
    } else {
        format!(
            "UC1 {beat_part}:{}",
            render_groups(&groups(&t, -1, -5), "·")
        )
    };
    let t5 = tier(-5);
    let lo = (t / t5) * t5;
    let hi = lo + t5 - one();

    Fixture {
        name: name.to_string(),
        ticks: t.to_str_radix(10),
        human_exact,
        lowest_nonzero_tier: lowest,
        human_trunc_t5,
        trunc_window_lo: lo.to_str_radix(10),
        trunc_window_hi: hi.to_str_radix(10),
        digit5_t9_tm12: format!("UC1/5 {}", render_digit5(&groups(&t, 9, -12))),
        digit5_t5_tm12: format!("UC1/5 {}", render_digit5(&groups(&t, 5, -12))),
        ucid: ucid(&t),
        bytes_hex: bytes_hex(&t),
        v5: val,
    }
}

fn cf_table(label: &str, ratio_dec: &str) -> CfTable {
    let r = parse_decimal(ratio_dec);
    let whole = r.num / r.den;
    let frac = Rat::new(r.num - whole * r.den, r.den);

    let cf_frac = cf_expand(&frac, 32);
    let cf_whole = cf_expand(&r, 32);
    let cv_frac = convergents(&cf_frac);
    let cv_whole = convergents(&cf_whole);

    // Convergents of the fractional part, skipping the leading integer term 0/1.
    let cv_frac_reported: Vec<Rat> = cv_frac.iter().skip(1).cloned().collect();

    let thousand = Rat::from_u(small(1000));
    let mut drift = Vec::new();
    let mut slips = Vec::new();
    for c in &cv_frac_reported {
        let err = c.abs_diff(&frac);
        drift.push(err.mul(&thousand).render(9));
        slips.push(if err.is_zero() {
            "exact".to_string()
        } else {
            err.recip().render(0)
        });
    }

    CfTable {
        label: label.to_string(),
        ratio: r.render_ratio(),
        whole: whole.to_str_radix(10),
        cf_frac,
        cf_whole,
        convergents_frac: cv_frac_reported
            .iter()
            .map(|c| (c.num.to_str_radix(10), c.den.to_str_radix(10)))
            .collect(),
        convergents_whole: cv_whole
            .iter()
            .map(|c| (c.num.to_str_radix(10), c.den.to_str_radix(10)))
            .collect(),
        drift_per_1000: drift,
        slips_in: slips,
    }
}

// --------------------------------------------------------------------- drive

pub fn derive() -> Derivation {
    // --- primitives, two ways each ---
    let beat_computed = pow5(d::appendix_a::BEAT_EXPONENT);
    let beat_parsed = dec(d::appendix_a::BEAT);
    let second_computed =
        dec(d::appendix_a::SECOND_MANTISSA) * pow10(d::appendix_a::SECOND_DECIMAL_SCALE);
    let second_parsed = dec(d::appendix_a::SECOND);

    // --- provenance chain (§2.2), re-executed ---
    let julian_year = dec(d::provenance::JULIAN_YEAR_SECONDS);
    // 13.787 Gyr = 13787 x 10^6 Julian years
    let age_s = dec(d::provenance::INPUT_GYR_TIMES_1000) * pow10(6) * julian_year;
    let age_ticks = age_s * second_computed;
    let beats = div_half_even(&age_ticks, &beat_computed);
    let oo_from_chain = beats * beat_computed;
    let oo_from_beats = dec(d::appendix_a::ORIGIN_OFFSET_BEATS) * beat_computed;
    let oo_parsed = dec(d::appendix_a::ORIGIN_OFFSET);

    let residual = if oo_from_chain >= age_ticks {
        SignedDec {
            negative: false,
            magnitude: (oo_from_chain - age_ticks).to_str_radix(10),
        }
    } else {
        SignedDec {
            negative: true,
            magnitude: (age_ticks - oo_from_chain).to_str_radix(10),
        }
    };
    let residual_rat = Rat::new(dec(&residual.magnitude), second_computed);
    let residual_seconds_rendered = format!(
        "{}{}",
        if residual.negative { "-" } else { "" },
        residual_rat.render(9)
    );

    // --- BIG_BANG_CLAIM ---
    // 0.020 Gyr = 2 x 10^7 Julian years = 631 152 x 10^9 s, so the half-width
    // is 631 152 x SECOND_MANTISSA x 10^39 ticks (Appendix A).
    let bbc_computed = dec(d::appendix_a::BBC_JULIAN_SECONDS_TIMES_1000)
        * dec(d::appendix_a::SECOND_MANTISSA)
        * pow10(39);
    let bbc_parsed = dec(d::appendix_a::BIG_BANG_CLAIM_HALFWIDTH);
    // 0.020 Gyr in ticks = 20 x 10^6 Julian years x SECOND
    let point02_gyr = dec(d::provenance::UNCERTAINTY_GYR_TIMES_1000) * pow10(6) * julian_year
        * second_computed;
    let bbc_over = Rat::new(bbc_computed, point02_gyr);
    let bbc_in_drifts = Rat::new(bbc_computed, pow5(80)).render(2);

    // --- §2.4 alignment ---
    let nanosecond = second_computed / pow10(9);
    assert_eq!(
        nanosecond * pow10(9),
        second_computed,
        "SECOND must divide exactly by 10^9"
    );
    let samples = 400u64;
    let mut min_v5_s = u32::MAX;
    let mut min_v5_ns = u32::MAX;
    for n in 1..=samples {
        let k = small(n);
        min_v5_s = min_v5_s.min(v5(&(oo_parsed + k * second_computed)));
        min_v5_ns = min_v5_ns.min(v5(&(oo_parsed + k * nanosecond)));
    }

    // --- tier grid ---
    let mut tiers = Vec::with_capacity(d::tiers::COUNT);
    for k in d::tiers::K_MIN..=d::tiers::K_MAX {
        tiers.push(tier(k).to_str_radix(10));
    }

    // --- fixtures ---
    let mut fixtures = Vec::new();
    for f in d::fixtures::ALL {
        let t = match (f.civil, f.name) {
            (Some((y, m, day, h, mi, s)), _) => {
                let secs = days_from_civil(y, m, day) * 86400
                    + h as i64 * 3600
                    + mi as i64 * 60
                    + s as i64;
                instant_from_si_offset(secs)
            }
            (None, n) if n.starts_with("absolute zero") => ZERO,
            (None, n) if n.starts_with("Earth formation") => {
                // SI_EPOCH - 4.54 Gyr = OO - 454 x 10^7 Julian years x SECOND
                let back = dec(d::fixtures::EARTH_FORMATION_GYR_TIMES_100_BEFORE_EPOCH)
                    * pow10(7)
                    * julian_year
                    * second_computed;
                oo_parsed
                    .checked_sub(back)
                    .expect("UCAL-E0020: Earth formation precedes the datum")
            }
            (None, n) if n.starts_with("recombination") => {
                dec(d::fixtures::RECOMBINATION_KYR_AFTER_DATUM)
                    * pow10(3)
                    * julian_year
                    * second_computed
            }
            (None, n) => panic!("unhandled non-civil fixture: {n}"),
        };
        fixtures.push(build_fixture(f.name, t));
    }
    // Extra negative-year vectors. Astronomical year numbering with negative
    // eras is where day-count algorithms go wrong (see the note on the 44 BC
    // fixture in `declared`), so the vector file pins a few more of them.
    for (y, m, day) in [(-1i64, 1u8, 1u8), (-100, 1, 1), (-4712, 1, 1)] {
        fixtures.push(build_fixture(
            &format!("{y:05}-{m:02}-{day:02}T00:00:00 TT"),
            instant_from_si_offset(days_from_civil(y, m, day) * 86400),
        ));
    }

    // --- Appendix I ---
    let cf_tables = vec![
        cf_table(
            d::appendix_i::I1_EARTH_INTERCALATION.label,
            d::appendix_i::I1_EARTH_INTERCALATION.ratio,
        ),
        cf_table(
            d::appendix_i::I2_EARTH_GROUPING.label,
            d::appendix_i::I2_EARTH_GROUPING.ratio,
        ),
        cf_table(
            d::appendix_i::I3_MARS_INTERCALATION.label,
            d::appendix_i::I3_MARS_INTERCALATION.ratio,
        ),
        cf_table(
            d::appendix_i::I5_TITAN_INTERCALATION.label,
            d::appendix_i::I5_TITAN_INTERCALATION.ratio,
        ),
    ];

    // --- Mars satellites (delta D-A5) ---
    let mars_solar = Rat::new(dec(d::appendix_i::MARS_SOLAR_DAY_S.0), pow10(3));
    let synodic = |p_orb: &Rat| -> Rat {
        // 1 / |1/P_orb - 1/P_solar|
        p_orb.recip().abs_diff(&mars_solar.recip()).recip()
    };
    let phobos = Rat::from_u(dec(d::appendix_i::PHOBOS_ORBITAL_S.0));
    let deimos = Rat::from_u(dec(d::appendix_i::DEIMOS_ORBITAL_S.0));
    let ph_sols = synodic(&phobos).div(&mars_solar);
    let de_sols = synodic(&deimos).div(&mars_solar);

    let mars_year_sols = parse_decimal(d::appendix_i::I3_MARS_INTERCALATION.ratio);
    let cycles_per_year = mars_year_sols.div(&de_sols);
    let cf_cy = cf_expand(&cycles_per_year, 10);
    let cv_cy = convergents(&cf_cy);

    Derivation {
        route: "A: bnum U512, fixed width".into(),
        beat_computed: beat_computed.to_str_radix(10),
        beat_parsed: beat_parsed.to_str_radix(10),
        second_computed: second_computed.to_str_radix(10),
        second_parsed: second_parsed.to_str_radix(10),
        origin_offset_from_chain: oo_from_chain.to_str_radix(10),
        origin_offset_from_beats: oo_from_beats.to_str_radix(10),
        origin_offset_parsed: oo_parsed.to_str_radix(10),
        bbc_computed: bbc_computed.to_str_radix(10),
        bbc_parsed: bbc_parsed.to_str_radix(10),
        domain_max: U512::MAX.to_str_radix(10),
        age_s: age_s.to_str_radix(10),
        age_ticks: age_ticks.to_str_radix(10),
        beats: beats.to_str_radix(10),
        residual,
        residual_seconds_rendered,
        oo_bits: bits(&oo_parsed),
        oo_base5_digits: oo_parsed.to_str_radix(5).len(),
        oo_trailing_base5_zeros: v5(&oo_parsed) as usize,
        v5_second: v5(&second_computed),
        v5_nanosecond: v5(&nanosecond),
        v5_origin_offset: v5(&oo_parsed),
        min_v5_whole_seconds: min_v5_s,
        min_v5_whole_nanoseconds: min_v5_ns,
        tiers,
        bbc_over_point02_gyr: bbc_over.render_ratio(),
        bbc_in_drifts,
        fixtures,
        cf_tables,
        deimos_synodic_sols: de_sols.render(6),
        phobos_synodic_sols: ph_sols.render(6),
        mars_deimos_cycles_per_year: cycles_per_year.render(9),
        mars_deimos_convergents: cv_cy
            .iter()
            .map(|c| (c.num.to_str_radix(10), c.den.to_str_radix(10)))
            .collect(),
    }
}

/// Assert the four §3.3 `const` constants agree with the parsed literals. This
/// exists so that a `const`-evaluation regression is a test failure, not a
/// silent divergence.
pub fn const_selfcheck() -> Vec<String> {
    let mut bad = Vec::new();
    if BEAT != dec(d::appendix_a::BEAT) {
        bad.push("const BEAT".into());
    }
    if SECOND != dec(d::appendix_a::SECOND) {
        bad.push("const SECOND".into());
    }
    if ORIGIN_OFFSET != dec(d::appendix_a::ORIGIN_OFFSET) {
        bad.push("const ORIGIN_OFFSET".into());
    }
    if BIG_BANG_CLAIM_HALFWIDTH != dec(d::appendix_a::BIG_BANG_CLAIM_HALFWIDTH) {
        bad.push("const BIG_BANG_CLAIM_HALFWIDTH".into());
    }
    if BEAT != pow5(60) {
        bad.push("const BEAT != 5^60".into());
    }
    bad
}
