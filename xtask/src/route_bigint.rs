//! Derivation route B: heap `num_bigint::BigUint` with `num_rational::BigRational`.
//!
//! See the table in `route_bnum` for the step-by-step algorithmic differences.
//! Nothing is shared with route A except the `Derivation` result type.

use num_bigint::{BigInt, BigUint};
use num_integer::Integer;
use num_rational::BigRational;
use num_traits::{One, Pow, Signed, ToPrimitive, Zero};

use crate::declared as d;
use crate::derivation::{CfTable, Derivation, Fixture, SignedDec};

fn dec(s: &str) -> BigUint {
    BigUint::parse_bytes(s.as_bytes(), 10).expect("invalid decimal")
}

fn small(n: u64) -> BigUint {
    BigUint::from(n)
}

/// `5^e` by repeated multiplication — not binary exponentiation (route A's method).
fn pow5(e: u32) -> BigUint {
    let five = small(5);
    let mut acc = BigUint::one();
    for _ in 0..e {
        acc *= &five;
    }
    acc
}

fn pow10(e: u32) -> BigUint {
    let ten = small(10);
    let mut acc = BigUint::one();
    for _ in 0..e {
        acc *= &ten;
    }
    acc
}

/// Full base-5 digit vector, least-significant first.
fn base5_digits(x: &BigUint) -> Vec<u8> {
    if x.is_zero() {
        return vec![0];
    }
    x.to_radix_le(5)
}

/// Base-5 valuation as the count of trailing zeros in the digit vector — not by
/// repeated division (route A's method).
fn v5(x: &BigUint) -> u32 {
    if x.is_zero() {
        return 0;
    }
    let ds = base5_digits(x);
    ds.iter().take_while(|d| **d == 0).count() as u32
}

fn bits(x: &BigUint) -> u32 {
    x.bits() as u32
}

/// Half-even division, expressed through `BigRational` comparison of the
/// doubled remainder rather than integer comparison.
fn div_half_even(n: &BigUint, den: &BigUint) -> BigUint {
    let (q, r) = n.div_rem(den);
    let twice = &r * 2u32;
    match twice.cmp(den) {
        std::cmp::Ordering::Greater => q + 1u32,
        std::cmp::Ordering::Less => q,
        std::cmp::Ordering::Equal => {
            if q.is_even() {
                q
            } else {
                q + 1u32
            }
        }
    }
}

// ------------------------------------------------------------------ rationals

fn rat(n: BigUint, d_: BigUint) -> BigRational {
    BigRational::new(BigInt::from(n), BigInt::from(d_))
}

fn rat_u(n: BigUint) -> BigRational {
    BigRational::from(BigInt::from(n))
}

fn parse_decimal(s: &str) -> BigRational {
    match s.split_once('.') {
        None => rat_u(dec(s)),
        Some((int, frac)) => {
            let scale = frac.len() as u32;
            rat(dec(&format!("{int}{frac}")), pow10(scale))
        }
    }
}

/// Render a non-negative rational to `digits` places, half-even (Rule R).
fn render(r: &BigRational, digits: u32) -> String {
    let num = r.numer().magnitude().clone();
    let den = r.denom().magnitude().clone();
    let scaled = num * pow10(digits);
    let q = div_half_even(&scaled, &den).to_str_radix(10);
    if digits == 0 {
        return q;
    }
    let dg = digits as usize;
    if q.len() <= dg {
        format!("0.{}{}", "0".repeat(dg - q.len()), q)
    } else {
        format!("{}.{}", &q[..q.len() - dg], &q[q.len() - dg..])
    }
}

fn render_ratio(r: &BigRational) -> String {
    format!(
        "{}/{}",
        r.numer().magnitude().to_str_radix(10),
        r.denom().magnitude().to_str_radix(10)
    )
}

// ------------------------------------------------------- continued fractions

fn cf_expand(r: &BigRational, max_depth: u32) -> Vec<u64> {
    let mut out = Vec::new();
    let mut n = r.numer().magnitude().clone();
    let mut den = r.denom().magnitude().clone();
    for _ in 0..max_depth {
        let (a, rem) = n.div_rem(&den);
        out.push(a.to_u64().expect("cf term exceeds u64"));
        if rem.is_zero() {
            break;
        }
        n = den;
        den = rem;
    }
    out
}

fn convergents(cf: &[u64]) -> Vec<BigRational> {
    let (mut hm1, mut hm2) = (BigUint::one(), BigUint::zero());
    let (mut km1, mut km2) = (BigUint::zero(), BigUint::one());
    let mut out = Vec::new();
    for &a in cf {
        let a = small(a);
        let h = &a * &hm1 + &hm2;
        let k = &a * &km1 + &km2;
        out.push(rat(h.clone(), k.clone()));
        hm2 = hm1;
        hm1 = h;
        km2 = km1;
        km1 = k;
    }
    out
}

// ------------------------------------------------------------------- codecs

fn tier(k: i8) -> BigUint {
    let e = 60i32 + 5 * k as i32;
    assert!(e >= 0, "tier below T-12");
    pow5(e as u32)
}

/// Decimal group values for tiers `k_hi`..`k_lo`, most significant first.
///
/// Route B builds the whole base-5 digit vector and chunks it in fives — the
/// opposite of Appendix F's `divmod` loop, which route A uses.
fn groups(t: &BigUint, k_hi: i8, k_lo: i8) -> Vec<u16> {
    let shifted = t / tier(k_lo);
    let ds = base5_digits(&shifted); // little-endian base-5 digits
    let n = (k_hi - k_lo + 1) as usize;
    let mut out = Vec::with_capacity(n);
    for g in 0..n {
        let mut v: u16 = 0;
        for j in (0..5usize).rev() {
            let idx = g * 5 + j;
            let digit = ds.get(idx).copied().unwrap_or(0) as u16;
            v = v * 5 + digit;
        }
        out.push(v);
    }
    out.reverse();
    out
}

fn digit5_of_group(g: u16) -> String {
    let mut v = g;
    let mut buf = [b'0'; 5];
    for i in (0..5).rev() {
        buf[i] = b'0' + (v % 5) as u8;
        v /= 5;
    }
    String::from_utf8(buf.to_vec()).expect("ascii")
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

/// UCID by repeated `divmod` by 32 — not bit extraction (route A's method).
fn ucid(t: &BigUint) -> Option<String> {
    if bits(t) > 256 {
        return None; // UCAL-E0031
    }
    let thirty_two = small(32);
    let mut x = t.clone();
    let mut chars = Vec::with_capacity(52);
    for _ in 0..52 {
        let (q, r) = x.div_rem(&thirty_two);
        chars.push(CROCKFORD[r.to_usize().expect("digit fits")]);
        x = q;
    }
    assert!(x.is_zero(), "UCID overflow: value did not fit in 260 bits");
    chars.reverse();
    Some(String::from_utf8(chars).expect("ascii"))
}

/// Canonical binary (§7.1, Rule B): left-pad the minimal BE bytes to 64.
fn bytes_hex(t: &BigUint) -> String {
    let raw = t.to_bytes_be();
    let raw = if raw == [0u8] { Vec::new() } else { raw };
    assert!(raw.len() <= 64, "value exceeds the 512-bit domain");
    let mut buf = vec![0u8; 64 - raw.len()];
    buf.extend_from_slice(&raw);
    buf.iter().map(|b| format!("{b:02x}")).collect()
}

// ------------------------------------------------------------ civil day count

const MONTH_LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

/// Days from `0000-01-01`, proleptic Gregorian, astronomical year numbering.
///
/// Cumulative month-table loop rather than Hinnant's era algorithm. Iterating
/// year by year is O(|y|) but the harness runs once, and the point is that it
/// shares no arithmetic with route A.
fn days_from_civil(y: i64, m: u8, day: u8) -> i64 {
    let mut days = 0i64;
    if y >= 0 {
        for yy in 0..y {
            days += if is_leap(yy) { 366 } else { 365 };
        }
    } else {
        for yy in y..0 {
            days -= if is_leap(yy) { 366 } else { 365 };
        }
    }
    for mm in 0..(m as usize - 1) {
        days += MONTH_LENGTHS[mm];
        if mm == 1 && is_leap(y) {
            days += 1;
        }
    }
    days + day as i64 - 1
}

// ----------------------------------------------------------------- fixtures

fn build_fixture(name: &str, t: &BigUint) -> Fixture {
    let val = v5(t);
    let beat_part = render_groups(&groups(t, 5, 0), "·");

    let lowest = if t.is_zero() {
        None
    } else {
        let k = (val as i32 - 60).div_euclid(5);
        Some(k.clamp(-12, 32) as i8)
    };
    let human_exact = match lowest {
        Some(k) if k < 0 => format!("UC1 {beat_part}:{}", render_groups(&groups(t, -1, k), "·")),
        _ => format!("UC1 {beat_part}"),
    };
    let human_trunc_t5 = if t.is_zero() {
        format!("UC1 {beat_part}")
    } else {
        format!("UC1 {beat_part}:{}", render_groups(&groups(t, -1, -5), "·"))
    };

    let t5 = tier(-5);
    let lo = (t / &t5) * &t5;
    let hi = &lo + &t5 - 1u32;

    Fixture {
        name: name.to_string(),
        ticks: t.to_str_radix(10),
        human_exact,
        lowest_nonzero_tier: lowest,
        human_trunc_t5,
        trunc_window_lo: lo.to_str_radix(10),
        trunc_window_hi: hi.to_str_radix(10),
        digit5_t9_tm12: format!("UC1/5 {}", render_digit5(&groups(t, 9, -12))),
        digit5_t5_tm12: format!("UC1/5 {}", render_digit5(&groups(t, 5, -12))),
        ucid: ucid(t),
        bytes_hex: bytes_hex(t),
        v5: val,
    }
}

fn cf_table(label: &str, ratio_dec: &str) -> CfTable {
    let r = parse_decimal(ratio_dec);
    let whole = r.numer().magnitude() / r.denom().magnitude();
    let frac = &r - rat_u(whole.clone());

    let cf_frac = cf_expand(&frac, 32);
    let cf_whole = cf_expand(&r, 32);
    let cv_frac: Vec<BigRational> = convergents(&cf_frac).into_iter().skip(1).collect();
    let cv_whole = convergents(&cf_whole);

    let thousand = rat_u(small(1000));
    let mut drift = Vec::new();
    let mut slips = Vec::new();
    for c in &cv_frac {
        let err = (c - &frac).abs();
        drift.push(render(&(&err * &thousand), 9));
        slips.push(if err.is_zero() {
            "exact".to_string()
        } else {
            render(&err.recip(), 0)
        });
    }

    CfTable {
        label: label.to_string(),
        ratio: render_ratio(&r),
        whole: whole.to_str_radix(10),
        cf_frac,
        cf_whole,
        convergents_frac: cv_frac.iter().map(pair).collect(),
        convergents_whole: cv_whole.iter().map(pair).collect(),
        drift_per_1000: drift,
        slips_in: slips,
    }
}

fn pair(r: &BigRational) -> (String, String) {
    (
        r.numer().magnitude().to_str_radix(10),
        r.denom().magnitude().to_str_radix(10),
    )
}

// --------------------------------------------------------------------- drive

pub fn derive() -> Derivation {
    let beat_computed = pow5(d::appendix_a::BEAT_EXPONENT);
    let beat_parsed = dec(d::appendix_a::BEAT);
    let second_computed =
        dec(d::appendix_a::SECOND_MANTISSA) * pow10(d::appendix_a::SECOND_DECIMAL_SCALE);
    let second_parsed = dec(d::appendix_a::SECOND);

    let julian_year = dec(d::provenance::JULIAN_YEAR_SECONDS);
    let age_s = dec(d::provenance::INPUT_GYR_TIMES_1000) * pow10(6) * &julian_year;
    let age_ticks = &age_s * &second_computed;
    let beats = div_half_even(&age_ticks, &beat_computed);
    let oo_from_chain = &beats * &beat_computed;
    let oo_from_beats = dec(d::appendix_a::ORIGIN_OFFSET_BEATS) * &beat_computed;
    let oo_parsed = dec(d::appendix_a::ORIGIN_OFFSET);

    let residual = if oo_from_chain >= age_ticks {
        SignedDec {
            negative: false,
            magnitude: (&oo_from_chain - &age_ticks).to_str_radix(10),
        }
    } else {
        SignedDec {
            negative: true,
            magnitude: (&age_ticks - &oo_from_chain).to_str_radix(10),
        }
    };
    let residual_seconds_rendered = format!(
        "{}{}",
        if residual.negative { "-" } else { "" },
        render(
            &rat(dec(&residual.magnitude), second_computed.clone()),
            9
        )
    );

    let bbc_computed = dec(d::appendix_a::BBC_JULIAN_SECONDS_TIMES_1000)
        * dec(d::appendix_a::SECOND_MANTISSA)
        * pow10(39);
    let bbc_parsed = dec(d::appendix_a::BIG_BANG_CLAIM_HALFWIDTH);
    let point02_gyr = dec(d::provenance::UNCERTAINTY_GYR_TIMES_1000)
        * pow10(6)
        * &julian_year
        * &second_computed;
    let bbc_over = rat(bbc_computed.clone(), point02_gyr);
    let bbc_in_drifts = render(&rat(bbc_computed.clone(), pow5(80)), 2);

    let nanosecond = &second_computed / pow10(9);
    assert_eq!(
        &nanosecond * pow10(9),
        second_computed,
        "SECOND must divide exactly by 10^9"
    );
    let mut min_v5_s = u32::MAX;
    let mut min_v5_ns = u32::MAX;
    for n in 1..=400u64 {
        let k = small(n);
        min_v5_s = min_v5_s.min(v5(&(&oo_parsed + &k * &second_computed)));
        min_v5_ns = min_v5_ns.min(v5(&(&oo_parsed + &k * &nanosecond)));
    }

    let mut tiers = Vec::with_capacity(d::tiers::COUNT);
    for k in d::tiers::K_MIN..=d::tiers::K_MAX {
        tiers.push(tier(k).to_str_radix(10));
    }

    let mut fixtures = Vec::new();
    for f in d::fixtures::ALL {
        let t: BigUint = match (f.civil, f.name) {
            (Some((y, m, day, h, mi, s)), _) => {
                let secs = days_from_civil(y, m, day) * 86400
                    + h as i64 * 3600
                    + mi as i64 * 60
                    + s as i64;
                let mag = small(secs.unsigned_abs()) * &second_computed;
                if secs < 0 {
                    assert!(oo_parsed >= mag, "UCAL-E0020: result precedes the datum");
                    &oo_parsed - mag
                } else {
                    &oo_parsed + mag
                }
            }
            (None, n) if n.starts_with("absolute zero") => BigUint::zero(),
            (None, n) if n.starts_with("Earth formation") => {
                let back = dec(d::fixtures::EARTH_FORMATION_GYR_TIMES_100_BEFORE_EPOCH)
                    * pow10(7)
                    * &julian_year
                    * &second_computed;
                assert!(oo_parsed >= back, "UCAL-E0020");
                &oo_parsed - back
            }
            (None, n) if n.starts_with("recombination") => {
                dec(d::fixtures::RECOMBINATION_KYR_AFTER_DATUM)
                    * pow10(3)
                    * &julian_year
                    * &second_computed
            }
            (None, n) => panic!("unhandled non-civil fixture: {n}"),
        };
        fixtures.push(build_fixture(f.name, &t));
    }
    for (y, m, day) in [(-1i64, 1u8, 1u8), (-100, 1, 1), (-4712, 1, 1)] {
        let secs = days_from_civil(y, m, day) * 86400;
        let mag = small(secs.unsigned_abs()) * &second_computed;
        let t = if secs < 0 {
            assert!(oo_parsed >= mag, "UCAL-E0020");
            &oo_parsed - mag
        } else {
            &oo_parsed + mag
        };
        fixtures.push(build_fixture(
            &format!("{y:05}-{m:02}-{day:02}T00:00:00 TT"),
            &t,
        ));
    }

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

    // Mars satellites (delta D-A5)
    let mars_solar = rat(dec(d::appendix_i::MARS_SOLAR_DAY_S.0), pow10(3));
    let synodic = |p_orb: &BigRational| -> BigRational {
        (p_orb.recip() - mars_solar.recip()).abs().recip()
    };
    let ph = rat_u(dec(d::appendix_i::PHOBOS_ORBITAL_S.0));
    let de = rat_u(dec(d::appendix_i::DEIMOS_ORBITAL_S.0));
    let ph_sols = synodic(&ph) / &mars_solar;
    let de_sols = synodic(&de) / &mars_solar;

    let mars_year = parse_decimal(d::appendix_i::I3_MARS_INTERCALATION.ratio);
    let cycles = &mars_year / &de_sols;
    let cv_cy = convergents(&cf_expand(&cycles, 10));

    Derivation {
        route: "B: num-bigint heap + BigRational".into(),
        beat_computed: beat_computed.to_str_radix(10),
        beat_parsed: beat_parsed.to_str_radix(10),
        second_computed: second_computed.to_str_radix(10),
        second_parsed: second_parsed.to_str_radix(10),
        origin_offset_from_chain: oo_from_chain.to_str_radix(10),
        origin_offset_from_beats: oo_from_beats.to_str_radix(10),
        origin_offset_parsed: oo_parsed.to_str_radix(10),
        bbc_computed: bbc_computed.to_str_radix(10),
        bbc_parsed: bbc_parsed.to_str_radix(10),
        domain_max: (pow2(512) - 1u32).to_str_radix(10),
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
        bbc_over_point02_gyr: render_ratio(&bbc_over),
        bbc_in_drifts,
        fixtures,
        cf_tables,
        deimos_synodic_sols: render(&de_sols, 6),
        phobos_synodic_sols: render(&ph_sols, 6),
        mars_deimos_cycles_per_year: render(&cycles, 9),
        mars_deimos_convergents: cv_cy.iter().map(pair).collect(),
    }
}

fn pow2(e: u32) -> BigUint {
    BigUint::from(2u32).pow(e)
}
