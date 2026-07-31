//! Profiles (§2, Rule P) and the datum (Rule Q).
//!
//! A profile is an immutable, named constant set fixing the datum and the tick.
//! `UC-1` is normative.
//!
//! Three things about this module are load-bearing:
//!
//! - **Rule P.** Instants are parameterised by profile at the type level, so
//!   cross-profile arithmetic does not compile. Every serialised form carries
//!   [`Profile::TAG`].
//! - **Rule Q.1.** Tick 0 is a **stipulated** datum: exact by declaration,
//!   unrevisable within a profile. Nothing here describes it as measured,
//!   derived, observed, or as the creation of anything. The permitted phrasing is
//!   *the datum, conventionally identified with the FLRW t→0 limit*.
//! - **Rule Q.3 / Q.4.** [`Profile::big_bang_claim`] returns a [`SignedWindow`],
//!   which has no arithmetic. [`Profile::datum_provenance`] returns
//!   machine-readable data, not prose, so the chain can be re-executed and
//!   audited — the P0 harness does exactly that.
//!
//! ### Deviation from §13's literal signatures
//!
//! §13 writes the profile constants as `const BEAT: Ticks` and
//! `fn bridge() -> &'static Bridge`. Associated `const`s and `static`s cannot hold
//! a heap value, so on the `bigint` backend those forms are impossible. The trait
//! therefore uses by-value functions throughout. On the default backend the
//! underlying literals remain `const` — see [`uc1::consts`] — which is what §3.3
//! actually requires.

use crate::backend::{TickInt, Ticks};
use crate::error::Result;
use crate::tier::TierTable;
use crate::value::{Delta, Instant, SignedWindow};

/// The reference frame a profile declares (Rule F).
///
/// Implementations MUST NOT convert between frames and MUST NOT claim
/// observer-independence (N2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Frame {
    /// Proper time along a comoving worldline in an FLRW frame — cosmological
    /// time in the CMB rest frame. The frame in which "the universe is 13.787 Gyr
    /// old" is a meaningful statement (§1.1).
    FlrwComoving,
}

impl Frame {
    /// Human description, for `ucal doctor` and `ucal datum`.
    pub const fn describe(self) -> &'static str {
        match self {
            Frame::FlrwComoving => "FLRW comoving (cosmological time, CMB rest frame)",
        }
    }
}

/// A literature citation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Citation {
    /// Full source reference.
    pub source: &'static str,
    /// DOI, bibcode or URL, where one exists.
    pub locator: Option<&'static str>,
}

/// A foreign-unit value recorded verbatim with its unit (Rule Y.1).
///
/// Rule Y concedes metrology and nothing else: empirical inputs arrive in foreign
/// units because that is how measurement works, but the *declared constant* — the
/// value the specification and the code use — is always the tick value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeasuredValue {
    /// The value exactly as published, as a string. Never parsed into a float.
    pub verbatim: &'static str,
    /// The unit the published value is in.
    pub unit: &'static str,
    /// What quantity this is.
    pub quantity: &'static str,
    /// Published uncertainty, verbatim, where one is quoted.
    pub uncertainty: Option<&'static str>,
    /// Where it came from.
    pub citation: Citation,
}

/// The rounding applied when converting an empirical input into a declared
/// constant (Rule Q.4).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RoundingRecord {
    /// What the value was rounded to, e.g. `"BEAT"`.
    pub to: &'static str,
    /// The mode, e.g. `"half_even"`.
    pub mode: &'static str,
    /// Signed residual in ticks, as a decimal string.
    pub residual_ticks: &'static str,
    /// The residual rendered in a foreign unit, for human orientation only.
    pub residual_rendered: &'static str,
    /// Why this rounding target was chosen.
    pub rationale: &'static str,
}

/// A machine-readable provenance record for a profile's datum (Rule Q.4).
///
/// Provenance is **data, not prose** — auditable, re-executable, and replaceable
/// without editing specification text. Absence is `UCAL-E0013`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Provenance {
    /// The empirical input, verbatim, with unit and citation.
    pub input: MeasuredValue,
    /// Definitions of any units the chain uses, so the chain is self-contained.
    pub unit_defs: &'static [(&'static str, &'static str)],
    /// The exact conversion chain, step by step. Every step must be reproducible
    /// by exact integer arithmetic; the UC-P0 harness re-executes all of them.
    pub chain: &'static [&'static str],
    /// The rounding applied at the end of the chain.
    pub rounding: RoundingRecord,
    /// An explicit statement of what Earth-derived quantities the chain touches
    /// and why they do not reach any computation (Rule Y, F12).
    pub earth_dependency: &'static str,
    /// Routes a future profile might take instead. Documented so the choice is
    /// visible rather than implicit (D-21, GE-6).
    pub alternative_routes: &'static [&'static str],
}

/// A bridge constant: a profile constant whose sole purpose is conversion to a
/// foreign unit system (Rule A.3).
///
/// Profile `UC-1` declares exactly one. A bridge constant MUST be an exact
/// integer number of ticks, so conversion *into* absolute time is multiplication
/// and never requires rounding (Rule A.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Bridge {
    /// The foreign unit's name, e.g. `"second"`.
    pub name: &'static str,
    /// Its exact length in ticks.
    pub ticks: Ticks,
    /// The largest `n` such that `5^n` divides `ticks`. This is what makes the
    /// §2.4 alignment invariants hold and decimal subdivisions exact (D-3).
    pub divisibility: u32,
    /// What the bridge's zero point is, as a human label. This anchors the
    /// *bridge*, not the datum and not any calendar (§2.1).
    pub epoch_label: &'static str,
}

/// An immutable, named constant set fixing a datum and a tick (§2.1).
pub trait Profile: 'static + Copy + Clone + PartialEq + Eq + core::fmt::Debug {
    /// The profile tag carried by every serialised form (Rule P).
    const TAG: &'static str;

    /// The declared frame (Rule F).
    const FRAME: Frame;

    /// The base tier in ticks: `5^60` for `UC-1` (D-2).
    fn beat() -> Ticks;

    /// The tick value of the bridge epoch, i.e. how far the bridge's zero point
    /// lies after the datum.
    fn origin_offset() -> Ticks;

    /// The largest representable tick value.
    fn domain_max() -> Ticks;

    /// The profile's single door to foreign units (Rule A.3).
    fn bridge() -> Bridge;

    /// The materialised tier grid.
    fn tiers() -> TierTable {
        TierTable::build()
    }

    /// The signed tick window within which this profile asserts the FLRW t→0
    /// limit lies, relative to its own datum (Rule Q.3).
    ///
    /// **Metadata only.** No arithmetic operation may consume this. It exists so
    /// that a user learns the physical *interpretation* is uncertain while the
    /// *arithmetic* is exact. The return type has no operators and no conversion
    /// into [`Delta`], [`Instant`] or `Window`, which is what makes misuse a
    /// compile error rather than a runtime `UCAL-E0025`.
    fn big_bang_claim() -> SignedWindow;

    /// Citation for the `big_bang_claim` window.
    fn big_bang_claim_citation() -> Citation;

    /// The datum provenance record (Rule Q.4). Absence is `UCAL-E0013`, which is
    /// why this returns a `Result` rather than an `Option`.
    fn datum_provenance() -> Result<&'static Provenance>;

    /// The datum statement, in the phrasing Rule Q.1 permits.
    ///
    /// Implementations MUST NOT describe tick 0 as measured, derived, observed, or
    /// as "the creation of the universe". A documentation lint enforces this  // ucal-lint-allow(datum-no-overclaim): mention, not use
    /// (§21.3-5), and `ucal datum` prints this string verbatim (§19.2).
    fn datum_statement() -> &'static str {
        "tick 0 is a stipulated reference point, conventionally identified with \
         the FLRW t→0 limit"
    }

    /// The datum itself.
    fn datum() -> Instant<Self>
    where
        Self: Sized,
    {
        Instant::zero()
    }

    /// The bridge epoch as an instant.
    fn bridge_epoch() -> Result<Instant<Self>>
    where
        Self: Sized,
    {
        Instant::from_ticks(Self::origin_offset())
    }
}

/// Profile `UC-1` — normative (§2, Appendix A).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UC1;

/// The `UC-1` constants.
pub mod uc1 {
    /// The declared constants, `const` on the default backend (§3.3).
    ///
    /// These decimal literals are the only transcribed constants in the crate,
    /// and the UC-P0 harness reproduces every one of them by two independent
    /// exact-integer routes. `§21.3` forbids any hand-transcribed constant the
    /// harness does not reproduce.
    pub mod consts {
        /// `BEAT = 5^60`.
        pub const BEAT_DEC: &str = "867361737988403547205962240695953369140625";

        /// `ORIGIN_OFFSET = 9 304 311 741 502 590 385 x BEAT`.
        ///
        /// 203 bits; 88 base-5 digits; **61** trailing base-5 zeros. (Appendix A
        /// annotates 62; the exact valuation is 61, because
        /// `9 304 311 741 502 590 385` contributes exactly one factor of five.
        /// See `spec/SPEC-DELTAS.md` D-A2. §2.4 requires at least 60, so the
        /// correction has no behavioural effect.)
        pub const ORIGIN_OFFSET_DEC: &str =
            "8070204002895596515944343085635637180530466139316558837890625";

        /// The beat count of `ORIGIN_OFFSET`, retained so the datum's whole-beat
        /// structure is checkable without factoring.
        pub const ORIGIN_OFFSET_BEATS_DEC: &str = "9304311741502590385";

        /// `SECOND = 18 548 584 399 861 x 10^30` ticks — the sole bridge constant.
        pub const SECOND_DEC: &str = "18548584399861000000000000000000000000000000";

        /// The largest `n` with `5^n | SECOND`. D-3 chooses the nearest multiple
        /// of `10^30` to the measured reciprocal Planck time precisely so that
        /// every decimal SI subdivision down to `10^-30` s is exact.
        pub const SECOND_DIVISIBILITY: u32 = 30;

        /// Half-width of `BIG_BANG_CLAIM`, in ticks: `+/- 0.020 Gyr`.
        ///
        /// Private to the profile: reachable only through
        /// [`super::super::Profile::big_bang_claim`], which returns an inert
        /// [`crate::value::SignedWindow`] (Rule Q.3, §3.3).
        pub(in crate::profile) const BIG_BANG_CLAIM_HALFWIDTH_DEC: &str =
            "11706976141141069872000000000000000000000000000000000000000";
    }

    use super::{Citation, MeasuredValue, Provenance, RoundingRecord};

    /// Planck 2018 cosmological parameters.
    pub const PLANCK_2018: Citation = Citation {
        source: "Planck 2018 results VI: Cosmological parameters, A&A 641, A6 (2020)",
        locator: Some("doi:10.1051/0004-6361/201833910"),
    };

    /// The `UC-1` datum provenance record (§2.2, Rule Q.4).
    pub static PROVENANCE: Provenance = Provenance {
        input: MeasuredValue {
            verbatim: "13.787",
            unit: "Gyr",
            quantity: "age_of_universe",
            uncertainty: Some("0.020 Gyr"),
            citation: PLANCK_2018,
        },
        unit_defs: &[(
            "Gyr",
            "10^9 x 31 557 600 s (Julian years, exact by definition)",
        )],
        chain: &[
            "AGE_s = 13 787 000 000 x 31 557 600 = 435 084 631 200 000 000 s (exact)",
            "AGE_ticks = AGE_s x SECOND = \
             8070204002895596516263200000000000000000000000000000000000000 (exact)",
            "beats = round_half_even(AGE_ticks / BEAT) = 9 304 311 741 502 590 385",
            "ORIGIN_OFFSET = beats x BEAT = \
             8070204002895596515944343085635637180530466139316558837890625",
        ],
        rounding: RoundingRecord {
            to: "BEAT",
            mode: "half_even",
            residual_ticks: "-318856914364362819469533860683441162109375",
            residual_rendered: "-0.017190364 s",
            rationale: "a whole-beat datum makes all sub-beat digits of the bridge \
                        epoch zero (§2.4)",
        },
        earth_dependency:
            "The input arrives in Julian years and the bridge anchor is an Earth \
             calendar date. Both are metrology (Rule Y). Neither appears in any \
             computation: ORIGIN_OFFSET is a declared integer of ticks.",
        alternative_routes: &[
            "A future profile MAY anchor provenance on an observable — e.g. CMB last \
             scattering at z = 1089.9 +/- 0.4 — and derive the offset to the datum \
             through ucal-cosmo in ticks, removing the Julian year and the Earth date \
             from the chain. This improves auditability, not exactness: measurement \
             yields a window and a datum is a point, so any route terminates in a \
             stipulation (Rule Q.2). See GE-6.",
        ],
    };
}

impl Profile for UC1 {
    const TAG: &'static str = "UC1";
    const FRAME: Frame = Frame::FlrwComoving;

    fn beat() -> Ticks {
        crate::backend::konst(uc1::consts::BEAT_DEC)
    }

    fn origin_offset() -> Ticks {
        crate::backend::konst(uc1::consts::ORIGIN_OFFSET_DEC)
    }

    fn domain_max() -> Ticks {
        <Ticks as TickInt>::domain_max()
    }

    fn bridge() -> Bridge {
        Bridge {
            name: "second",
            ticks: crate::backend::konst(uc1::consts::SECOND_DEC),
            divisibility: uc1::consts::SECOND_DIVISIBILITY,
            epoch_label: "0000-01-01T00:00:00.000 TT, proleptic Gregorian, \
                          astronomical year numbering",
        }
    }

    fn big_bang_claim() -> SignedWindow {
        SignedWindow::symmetric(Delta::from_ticks(crate::backend::konst(
            uc1::consts::BIG_BANG_CLAIM_HALFWIDTH_DEC,
        )))
    }

    fn big_bang_claim_citation() -> Citation {
        uc1::PLANCK_2018
    }

    fn datum_provenance() -> Result<&'static Provenance> {
        Ok(&uc1::PROVENANCE)
    }
}

/// A profile with no provenance record, used only to prove `UCAL-E0013` fires.
///
/// Rule Q.4 makes provenance mandatory. A conforming implementation must reject
/// such a profile rather than treat the absence as a default, so the failure path
/// needs something to exercise it.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProfileWithoutProvenance;

#[cfg(test)]
impl Profile for ProfileWithoutProvenance {
    const TAG: &'static str = "TEST-NOPROV";
    const FRAME: Frame = Frame::FlrwComoving;
    fn beat() -> Ticks {
        UC1::beat()
    }
    fn origin_offset() -> Ticks {
        UC1::origin_offset()
    }
    fn domain_max() -> Ticks {
        UC1::domain_max()
    }
    fn bridge() -> Bridge {
        UC1::bridge()
    }
    fn big_bang_claim() -> SignedWindow {
        UC1::big_bang_claim()
    }
    fn big_bang_claim_citation() -> Citation {
        UC1::big_bang_claim_citation()
    }
    fn datum_provenance() -> Result<&'static Provenance> {
        Err(crate::error::TimeError::new(crate::error::Code::E0013))
    }
}

/// The base-5 valuation of a tick count: how many trailing base-5 digits are zero.
///
/// This is the quantity the §2.4 alignment invariants are stated in, and the one
/// that decides how many groups a tick-exact rendering needs.
pub fn base5_valuation(ticks: &Ticks) -> u32 {
    if ticks.is_zero_ticks() {
        return 0;
    }
    let five = <Ticks as TickInt>::from_u64(5);
    let mut n = ticks.clone();
    let mut k = 0u32;
    loop {
        let (q, r) = n.quot_rem(&five);
        if !r.is_zero_ticks() {
            return k;
        }
        n = q;
        k += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Code;
    use crate::tier::Tier;
    use crate::value::Sign;

    fn dec(s: &str) -> Ticks {
        <Ticks as TickInt>::from_dec_str(s).unwrap()
    }

    #[test]
    fn beat_is_five_to_the_sixtieth() {
        assert_eq!(UC1::beat(), <Ticks as TickInt>::pow5(60).unwrap());
        assert_eq!(UC1::beat(), Tier::BEAT.ticks());
    }

    #[test]
    fn origin_offset_is_a_whole_number_of_beats() {
        let (q, r) = UC1::origin_offset().quot_rem(&UC1::beat());
        assert!(r.is_zero_ticks(), "the datum must be a whole beat count (§2.2)");
        assert_eq!(q, dec(uc1::consts::ORIGIN_OFFSET_BEATS_DEC));
    }

    #[test]
    fn provenance_chain_reaches_the_declared_origin_offset() {
        // §21.3-2: the chain must re-execute to ORIGIN_OFFSET with the stated
        // residual. The full two-route re-execution lives in the UC-P0 harness;
        // this asserts the endpoint so the library alone cannot drift from it.
        let julian_year = <Ticks as TickInt>::from_u64(31_557_600);
        let age_s = dec("13787")
            .try_mul(&<Ticks as TickInt>::pow5(6).unwrap())
            .and_then(|v| v.try_mul(&<Ticks as TickInt>::from_u64(2u64.pow(6))))
            .and_then(|v| v.try_mul(&julian_year))
            .unwrap();
        // 10^6 = 5^6 x 2^6, kept factored so the test needs no decimal literal.
        assert_eq!(age_s, dec("435084631200000000"));

        let second = UC1::bridge().ticks;
        let age_ticks = age_s.try_mul(&second).unwrap();

        // round half even
        let (q, r) = age_ticks.quot_rem(&UC1::beat());
        let twice = r.try_add(&r).unwrap();
        let beats = match twice.cmp(&UC1::beat()) {
            core::cmp::Ordering::Greater => q.try_add(&<Ticks as TickInt>::one()).unwrap(),
            core::cmp::Ordering::Less => q,
            core::cmp::Ordering::Equal if q.is_odd() => {
                q.try_add(&<Ticks as TickInt>::one()).unwrap()
            }
            core::cmp::Ordering::Equal => q,
        };
        assert_eq!(beats, dec(uc1::consts::ORIGIN_OFFSET_BEATS_DEC));

        let oo = beats.try_mul(&UC1::beat()).unwrap();
        assert_eq!(oo, UC1::origin_offset());

        // The residual is negative: the rounded datum precedes the unrounded age.
        assert!(oo < age_ticks);
        let residual = age_ticks.try_sub(&oo).unwrap();
        assert_eq!(
            residual,
            dec("318856914364362819469533860683441162109375")
        );
        let rec = UC1::datum_provenance().unwrap().rounding;
        assert_eq!(
            rec.residual_ticks,
            "-318856914364362819469533860683441162109375"
        );
        assert_eq!(rec.mode, "half_even");
    }

    #[test]
    fn alignment_invariants_hold() {
        // §2.4 / §21.3-1
        let second = UC1::bridge().ticks;
        assert_eq!(base5_valuation(&second), 30);
        assert_eq!(UC1::bridge().divisibility, 30);

        let ten9 = <Ticks as TickInt>::from_u64(1_000_000_000);
        let (nanosecond, r) = second.quot_rem(&ten9);
        assert!(r.is_zero_ticks(), "SECOND must divide exactly by 10^9");
        assert_eq!(base5_valuation(&nanosecond), 21);

        // SI_EPOCH is zero in all tiers below T0, i.e. v5 >= 60. The exact
        // valuation is 61 (delta D-A2), not the 62 Appendix A annotates.
        assert_eq!(base5_valuation(&UC1::origin_offset()), 61);
        assert!(base5_valuation(&UC1::origin_offset()) >= 60);

        // Every whole SI second keeps at least 30 trailing base-5 zeros.
        for n in 1..64u64 {
            let t = UC1::origin_offset()
                .try_add(&second.try_mul(&<Ticks as TickInt>::from_u64(n)).unwrap())
                .unwrap();
            assert!(base5_valuation(&t) >= 30, "n = {n}");
            let t = UC1::origin_offset()
                .try_add(
                    &nanosecond
                        .try_mul(&<Ticks as TickInt>::from_u64(n))
                        .unwrap(),
                )
                .unwrap();
            assert!(base5_valuation(&t) >= 21, "n = {n}");
        }
    }

    #[test]
    fn origin_offset_structure_matches_appendix_a() {
        let oo = UC1::origin_offset();
        assert_eq!(oo.bit_len(), 203);
        #[cfg(feature = "alloc")]
        assert_eq!(oo.to_radix_string(5).len(), 88);
    }

    #[test]
    fn big_bang_claim_is_symmetric_and_inert() {
        let claim = UC1::big_bang_claim();
        assert_eq!(claim.lo().sign(), Sign::Negative);
        assert_eq!(claim.hi().sign(), Sign::Positive);
        assert_eq!(claim.lo().magnitude(), claim.hi().magnitude());
        // 0.020 Gyr exactly: 20 x 10^6 Julian years x SECOND.
        let expected = <Ticks as TickInt>::from_u64(20)
            .try_mul(&dec("1000000"))
            .and_then(|v| v.try_mul(&<Ticks as TickInt>::from_u64(31_557_600)))
            .and_then(|v| v.try_mul(&UC1::bridge().ticks))
            .unwrap();
        assert_eq!(claim.hi().magnitude().ticks(), &expected);
    }

    #[test]
    fn missing_provenance_is_e0013() {
        // Rule Q.4: absence is an error, never a default.
        let err = ProfileWithoutProvenance::datum_provenance().unwrap_err();
        assert_eq!(err.code, Code::E0013);
        assert_eq!(err.code.exit_code(), 6);
        assert!(UC1::datum_provenance().is_ok());
    }

    #[test]
    fn datum_statement_makes_no_measurement_claim() {
        // Rule Q.1 / §21.3-5. The documentation lint covers the whole tree; this
        // pins the one string that gets printed to users.
        let s = UC1::datum_statement().to_lowercase();
        assert!(s.contains("stipulated"));
        for forbidden in [
            "creation of the universe",
            "age of the universe is",
            "measured",
            "observed",
            "big bang occurred",
        ] {
            assert!(!s.contains(forbidden), "datum statement claims too much: {forbidden}");
        }
    }

    #[test]
    fn frame_is_declared() {
        // Rule F: every profile must declare its frame.
        assert_eq!(UC1::FRAME, Frame::FlrwComoving);
        assert!(UC1::FRAME.describe().contains("FLRW"));
    }

    #[test]
    fn bridge_is_the_only_foreign_unit_door() {
        let b = UC1::bridge();
        assert_eq!(b.name, "second");
        // Rule A.4: exact integer of ticks, so conversion in is multiplication.
        assert_eq!(b.ticks, dec(uc1::consts::SECOND_DEC));
        // D-3: divisible by 10^30, hence by 5^30.
        let p5 = <Ticks as TickInt>::pow5(b.divisibility).unwrap();
        assert!(b.ticks.quot_rem(&p5).1.is_zero_ticks());
    }
}
