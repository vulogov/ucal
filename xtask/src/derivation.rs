//! The plain-data result of a derivation route.
//!
//! Every field is a decimal string or a small integer. The two routes share
//! *this type* and nothing else — no arithmetic, no codec, no algorithm. All
//! cross-checking is done by comparing these values, so agreement between
//! routes is evidence about the arithmetic and not an artefact of shared code.

/// A signed magnitude, since the tick domain is unsigned (Rule Z) but the
/// provenance residual is not.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SignedDec {
    pub negative: bool,
    /// Decimal magnitude, no sign, no separators.
    pub magnitude: String,
}

impl SignedDec {
    pub fn render(&self) -> String {
        if self.negative && self.magnitude != "0" {
            format!("-{}", self.magnitude)
        } else {
            self.magnitude.clone()
        }
    }
}

/// One fixture, fully rendered.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fixture {
    pub name: String,
    /// Decimal tick count.
    pub ticks: String,
    /// Tick-exact human form (§6.1/§6.4): T5..T0, then `:` and T-1 down to the
    /// lowest non-zero sub-beat tier. The sub-beat part is absent when every
    /// tier below T0 is zero.
    pub human_exact: String,
    /// The lowest non-zero tier index, or `None` for the datum.
    pub lowest_nonzero_tier: Option<i8>,
    /// Human form truncated to T-5, which is what Appendix C prints. Under Rule
    /// T this denotes a window, not a tick — delta D-A4.
    pub human_trunc_t5: String,
    /// The window that `human_trunc_t5` denotes: `[lo, hi]` inclusive.
    pub trunc_window_lo: String,
    pub trunc_window_hi: String,
    /// Base-5 digit form over tiers T9..T-12, the range Appendix C prints (22
    /// groups). Recorded with its range explicit.
    pub digit5_t9_tm12: String,
    /// Base-5 digit form over T5..T-12 (18 groups).
    pub digit5_t5_tm12: String,
    /// Crockford base-32 of the 256-bit big-endian value, 52 chars (§7.2).
    pub ucid: Option<String>,
    /// Canonical 64-byte big-endian encoding, lowercase hex (§7.1, Rule B).
    pub bytes_hex: String,
    /// Base-5 valuation, i.e. count of trailing zero base-5 digits.
    pub v5: u32,
}

/// One continued-fraction table (Appendix I).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CfTable {
    pub label: String,
    /// Exact ratio as `numerator/denominator`.
    pub ratio: String,
    pub whole: String,
    /// Continued fraction of the fractional part.
    pub cf_frac: Vec<u64>,
    /// Continued fraction of the whole ratio.
    pub cf_whole: Vec<u64>,
    /// Convergents of the fractional part as `(num, den)`.
    pub convergents_frac: Vec<(String, String)>,
    /// Convergents of the whole ratio as `(num, den)`.
    pub convergents_whole: Vec<(String, String)>,
    /// Per convergent of the fractional part: exact drift over 1000 periods,
    /// rendered as a decimal string with 9 fractional digits (Rule R: rendering
    /// is where rounding lives, and the mode is stated).
    pub drift_per_1000: Vec<String>,
    /// Per convergent: periods until one unit of drift accumulates, truncated.
    pub slips_in: Vec<String>,
}

/// The complete output of one derivation route.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Derivation {
    pub route: String,

    // ---- Appendix A primitives, derived two ways within the route ----
    /// `5^60` computed by the route's own exponentiation.
    pub beat_computed: String,
    /// `BEAT` obtained by parsing the RFC's decimal literal.
    pub beat_parsed: String,
    /// `SECOND` from mantissa x 10^30 (D-3), not from the literal.
    pub second_computed: String,
    pub second_parsed: String,
    /// `ORIGIN_OFFSET` from the full provenance chain (§2.2).
    pub origin_offset_from_chain: String,
    /// `ORIGIN_OFFSET` from `beats x BEAT` with `beats` parsed.
    pub origin_offset_from_beats: String,
    pub origin_offset_parsed: String,
    pub bbc_computed: String,
    pub bbc_parsed: String,
    pub domain_max: String,

    // ---- provenance chain (§2.2) ----
    pub age_s: String,
    pub age_ticks: String,
    pub beats: String,
    pub residual: SignedDec,
    pub residual_seconds_rendered: String,

    // ---- structural facts about ORIGIN_OFFSET ----
    pub oo_bits: u32,
    pub oo_base5_digits: usize,
    pub oo_trailing_base5_zeros: usize,

    // ---- §2.4 alignment ----
    pub v5_second: u32,
    pub v5_nanosecond: u32,
    pub v5_origin_offset: u32,
    /// Minimum v5 over `ORIGIN_OFFSET + n x SECOND` for n in 1..=SAMPLES.
    pub min_v5_whole_seconds: u32,
    /// Minimum v5 over `ORIGIN_OFFSET + n x NANOSECOND`.
    pub min_v5_whole_nanoseconds: u32,

    // ---- tier grid (§4, Appendix B) ----
    /// `5^(60+5k)` for k = -12..=32, in ascending k. 45 entries.
    pub tiers: Vec<String>,

    // ---- BIG_BANG_CLAIM relations ----
    /// `BBC / (0.020 Gyr in ticks)` as an exact ratio; must be `1/1`.
    pub bbc_over_point02_gyr: String,
    /// BBC as a count of drifts (5^80), truncated to 2 decimals.
    pub bbc_in_drifts: String,

    pub fixtures: Vec<Fixture>,
    pub cf_tables: Vec<CfTable>,

    /// Deimos synodic period in sols, rendered to 6 decimals, and the
    /// Mars-year/Deimos-synodic ratio (delta D-A5).
    pub deimos_synodic_sols: String,
    pub phobos_synodic_sols: String,
    pub mars_deimos_cycles_per_year: String,
    pub mars_deimos_convergents: Vec<(String, String)>,
}

/// Field-by-field comparison of two routes. Returns the names of fields that
/// differ, with both values.
pub fn diff(a: &Derivation, b: &Derivation) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    macro_rules! cmp {
        ($f:ident) => {
            if a.$f != b.$f {
                out.push((
                    stringify!($f).to_string(),
                    format!("{:?}", a.$f),
                    format!("{:?}", b.$f),
                ));
            }
        };
    }
    cmp!(beat_computed);
    cmp!(beat_parsed);
    cmp!(second_computed);
    cmp!(second_parsed);
    cmp!(origin_offset_from_chain);
    cmp!(origin_offset_from_beats);
    cmp!(origin_offset_parsed);
    cmp!(bbc_computed);
    cmp!(bbc_parsed);
    cmp!(domain_max);
    cmp!(age_s);
    cmp!(age_ticks);
    cmp!(beats);
    cmp!(residual);
    cmp!(residual_seconds_rendered);
    cmp!(oo_bits);
    cmp!(oo_base5_digits);
    cmp!(oo_trailing_base5_zeros);
    cmp!(v5_second);
    cmp!(v5_nanosecond);
    cmp!(v5_origin_offset);
    cmp!(min_v5_whole_seconds);
    cmp!(min_v5_whole_nanoseconds);
    cmp!(tiers);
    cmp!(bbc_over_point02_gyr);
    cmp!(bbc_in_drifts);
    cmp!(deimos_synodic_sols);
    cmp!(phobos_synodic_sols);
    cmp!(mars_deimos_cycles_per_year);
    cmp!(mars_deimos_convergents);

    if a.fixtures.len() != b.fixtures.len() {
        out.push((
            "fixtures.len".into(),
            a.fixtures.len().to_string(),
            b.fixtures.len().to_string(),
        ));
    } else {
        for (x, y) in a.fixtures.iter().zip(b.fixtures.iter()) {
            if x != y {
                out.push((
                    format!("fixture[{}]", x.name),
                    format!("{x:?}"),
                    format!("{y:?}"),
                ));
            }
        }
    }
    if a.cf_tables.len() != b.cf_tables.len() {
        out.push((
            "cf_tables.len".into(),
            a.cf_tables.len().to_string(),
            b.cf_tables.len().to_string(),
        ));
    } else {
        for (x, y) in a.cf_tables.iter().zip(b.cf_tables.iter()) {
            if x != y {
                out.push((
                    format!("cf_table[{}]", x.label),
                    format!("{x:?}"),
                    format!("{y:?}"),
                ));
            }
        }
    }
    out
}
