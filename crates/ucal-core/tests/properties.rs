//! F2, F3, F4 — the invariants as properties over generated input.
//!
//! # What was missing
//!
//! The suite has always been strong on *cases*: every Appendix C fixture, every
//! declared constant, every worked example. A case shows that a rule holds where
//! somebody thought to look, and the last three cycles found defect after defect
//! in places nobody had thought to look.
//!
//! These are the same rules, asserted over generated instants and tiers rather
//! than over a list.
//!
//! # Why a hand-rolled generator and no dependency
//!
//! A property-testing crate would bring a random seed and a shrinker. The seed
//! is the problem: a suite that passes today and fails tomorrow on the same tree
//! is a suite this project cannot use, because "CI green on every push with no
//! known-failing job" is a stated criterion and a flaky test destroys it.
//!
//! So the generator is a deterministic xorshift over a fixed seed. The same tree
//! generates the same inputs on every machine forever, and widening the search
//! is a visible edit to `ROUNDS` rather than a lucky run. What that costs is
//! shrinking: a failure reports the input that broke and not a minimal one.
//!
//! # F3, and why it is a digest
//!
//! Rule W says the two backends accept and reject exactly the same values. It
//! has been verified by running the same suite twice, which shows only that the
//! two agree on inputs somebody thought of — the same weakness as everything
//! else here.
//!
//! The backends are mutually exclusive, so no single process can compare them.
//! What a single process *can* do is reduce every generated result to one
//! number and assert it equals a committed constant: `RULE_W_DIGEST`. Both
//! backends must reach it. A divergence anywhere in ten thousand inputs changes
//! the digest, and the build fails on whichever backend is wrong.
//!
//! The digest is a plain FNV-1a over the decimal renderings — not a
//! cryptographic claim, and no dependency. It has to detect accident, not
//! forgery.

use ucal_core::backend::TickInt;
use ucal_core::codec::{self, Fmt};
use ucal_core::{Delta, Instant, Precision, Profile, Rounding, Ticks, Tier, Ucid, UC1};

/// How many inputs each property sees. Raising it is a deliberate edit.
const ROUNDS: usize = 10_000;

/// A deterministic 64-bit xorshift. No entropy, by design — see the module note.
struct Gen(u64);

impl Gen {
    fn new(seed: u64) -> Gen {
        Gen(seed | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    /// An instant spread across the whole domain, not clustered near zero.
    ///
    /// Built from up to eight 64-bit words so that the high tiers are exercised:
    /// a generator that only produced `u64`-sized values would test 64 bits of a
    /// 512-bit domain and report that everything was fine.
    fn instant(&mut self) -> Instant<UC1> {
        let words = 1 + (self.next() % 8) as usize;
        let mut v = <Ticks as TickInt>::zero();
        let shift = <Ticks as TickInt>::from_u64(u64::MAX);
        for _ in 0..words {
            v = v
                .try_mul(&shift)
                .and_then(|x| x.try_add(&<Ticks as TickInt>::from_u64(self.next())))
                .unwrap_or(v);
        }
        Instant::from_ticks(v).unwrap_or_else(|_| Instant::zero())
    }
    fn tier(&mut self) -> Tier {
        let all: Vec<Tier> = Tier::all_descending().collect();
        all[(self.next() % all.len() as u64) as usize]
    }
}

// ---------------------------------------------------------------- F2: round trips

/// Every text form parses back to the value it was rendered from.
///
/// §6 defines the forms and Rule S the ordering they support; nothing asserted
/// that rendering and parsing were inverses over anything but the fixtures.
#[test]
fn f2_every_text_form_round_trips() {
    let mut g = Gen::new(0x5EED_0001);
    for i in 0..ROUNDS {
        let t = g.instant();
        for (name, fmt) in [("human", Fmt::human()), ("digit5", Fmt::digit5())] {
            let Ok(text) = codec::render(&t, &fmt) else {
                continue;
            };
            let (back, precision) = codec::parse::<UC1>(&text, &fmt)
                .unwrap_or_else(|e| panic!("round {i}: `{name}` rendered {text} and would not parse back: {e}"));
            assert_eq!(
                back.ticks().to_dec_string(),
                t.ticks().to_dec_string(),
                "round {i}: `{name}` did not round-trip"
            );
            assert!(
                matches!(precision, Precision::Tick),
                "round {i}: a fully rendered `{name}` should parse at tick precision"
            );
        }
    }
}

/// Every UCID round-trips, and the canonical bytes do too.
///
/// §6.5 and Rule B. The byte form is a wire-format commitment: 64 bytes, and the
/// same 64 bytes on either backend.
#[test]
fn f2_identifiers_and_bytes_round_trip() {
    let mut g = Gen::new(0x5EED_0002);
    let mut ucids = 0;
    for i in 0..ROUNDS {
        let t = g.instant();

        let bytes = t.ticks().to_canonical_bytes();
        assert_eq!(bytes.len(), 64, "Rule B fixes the width at 64 bytes");
        let back = <Ticks as TickInt>::from_canonical_bytes(&bytes)
            .unwrap_or_else(|| panic!("round {i}: canonical bytes would not decode"));
        assert_eq!(back, *t.ticks(), "round {i}: bytes did not round-trip");

        // A UCID exists only below 2^256; above it, `UCAL-E0031` is the answer
        // and not a failure.
        if let Ok(u) = t.to_ucid() {
            ucids += 1;
            let s = u.to_string();
            let parsed = Ucid::parse(&s)
                .unwrap_or_else(|e| panic!("round {i}: UCID {s} would not parse: {e}"));
            let t2 = Instant::<UC1>::from_ucid(&parsed)
                .unwrap_or_else(|e| panic!("round {i}: UCID {s} would not decode: {e}"));
            assert_eq!(t2.ticks(), t.ticks(), "round {i}: UCID did not round-trip");
        }
    }
    assert!(ucids > 0, "no generated instant was inside UCID range");
}

// ------------------------------------------------------ F4: the rules, as properties

/// Rule Z: nothing precedes the datum, and subtraction says so rather than wrapping.
#[test]
fn f4_no_result_precedes_the_datum() {
    let mut g = Gen::new(0x5EED_0003);
    for i in 0..ROUNDS {
        let (a, b) = (g.instant(), g.instant());
        let (lo, hi) = if a <= b { (a, b) } else { (b, a) };

        // The larger minus the smaller is always a value.
        let d = hi
            .since(&lo)
            .unwrap_or_else(|e| panic!("round {i}: a valid difference was refused: {e}"));

        // The smaller minus the larger is always an error, never a wrap.
        if lo < hi {
            assert!(
                lo.since(&hi).is_err(),
                "round {i}: a result before the datum was produced instead of refused"
            );
        }

        // And the difference reconstructs.
        let back = lo
            .checked_add(&d)
            .unwrap_or_else(|e| panic!("round {i}: lo + (hi - lo) overflowed: {e}"));
        assert_eq!(back.ticks(), hi.ticks(), "round {i}: lo + (hi - lo) != hi");
    }
}

/// Rule T: a value stated to a coarser tier denotes an interval that contains it.
///
/// This is the rule the project treats as central — a truncated statement is an
/// interval, not a point with trailing zeros — and it was asserted on a handful
/// of tiers.
#[test]
fn f4_a_coarser_statement_contains_the_value() {
    let mut g = Gen::new(0x5EED_0004);
    for i in 0..ROUNDS {
        let t = g.instant();
        let tier = g.tier();

        let floored = t.floor_to(tier);
        let Ok(w) = floored.window_at(Precision::Tier(tier)) else {
            continue;
        };
        assert!(
            w.lo().ticks() <= t.ticks() && t.ticks() <= w.hi().ticks(),
            "round {i}: the window at {tier} does not contain the instant it came from"
        );
        assert!(
            w.lo().ticks() <= w.hi().ticks(),
            "round {i}: an inverted window is a wrong answer, not a wide one"
        );

        // Flooring is idempotent, and ceiling is never below flooring.
        assert_eq!(
            floored.floor_to(tier).ticks(),
            floored.ticks(),
            "round {i}: floor_to is not idempotent at {tier}"
        );
        if let Ok(c) = t.ceil_to(tier) {
            assert!(
                c.ticks() >= floored.ticks(),
                "round {i}: ceil_to fell below floor_to at {tier}"
            );
        }
    }
}

/// Rule O: the arithmetic refuses rather than wraps, at the ceiling as anywhere.
#[test]
fn f4_the_ceiling_refuses_rather_than_wraps() {
    let max = Instant::<UC1>::from_ticks(<Ticks as TickInt>::domain_max())
        .expect("domain_max is in the domain");
    for n in [1u64, 2, 1_000, u64::MAX] {
        let d = Delta::from_u64(n);
        assert!(
            max.checked_add(&d).is_err(),
            "adding {n} at the ceiling produced a value instead of an error"
        );
    }
    // And one tick below the ceiling, adding one is exactly reachable.
    let one = Delta::from_u64(1);
    let below = max.checked_sub(&one).expect("one below the ceiling exists");
    assert_eq!(
        below.checked_add(&one).expect("reachable").ticks(),
        max.ticks()
    );
}

// -------------------------------------------------- F3: Rule W, over generated input

/// FNV-1a. Not a cryptographic claim: it has to detect accident, not forgery.
fn fnv(acc: u64, s: &str) -> u64 {
    let mut h = acc;
    for b in s.as_bytes() {
        h ^= *b as u64;
        // ucal-lint-allow-begin(no-wrapping-arithmetic): Rule O forbids wrapping
        // arithmetic because a *time* value that wraps is a wrong answer wearing
        // a right one. A hash is the opposite: wrapping multiplication is its
        // definition, the value is never a quantity, and it is compared for
        // equality and nothing else. The exemption is declared rather than the
        // hash rewritten, because rewriting it would produce a worse hash to
        // satisfy a rule about something else.
        h = h.wrapping_mul(0x0000_0100_0000_01B3);
        // ucal-lint-allow-end(no-wrapping-arithmetic)
    }
    h
}

/// The digest both backends must reach.
///
/// Produced by running this test and reading the number it reports. It is a
/// committed constant rather than a comparison between two live backends
/// because the two cannot be compiled together — the `u512`/`bigint` guard is a
/// 2.0 problem, and this is what can be done under it.
const RULE_W_DIGEST: u64 = 0xce17_7574_e51d_526c;

/// Rule W: the backends agree on every generated input, not only on the ones
/// somebody thought of.
///
/// Ten thousand instants, each reduced through every operation that produces a
/// rendering, folded into one number. Run on both backends; a divergence
/// anywhere changes the digest.
#[test]
fn f3_both_backends_reach_the_same_digest() {
    let mut g = Gen::new(0x5EED_0005);
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for _ in 0..ROUNDS {
        let t = g.instant();
        let tier = g.tier();

        h = fnv(h, &t.ticks().to_dec_string());
        h = fnv(h, &t.ticks().to_radix_string(5));
        h = fnv(h, &codec::render(&t, &Fmt::human()).unwrap_or_default());
        h = fnv(h, &codec::render(&t, &Fmt::digit5()).unwrap_or_default());
        h = fnv(h, &t.to_ucid().map(|u| u.to_string()).unwrap_or_default());
        h = fnv(h, &t.floor_to(tier).ticks().to_dec_string());
        h = fnv(h, &t.tier_value(tier).to_string());
        h = fnv(
            h,
            &t.round_to(tier, Rounding::HalfEven)
                .map(|v| v.ticks().to_dec_string())
                .unwrap_or_default(),
        );
        let (q, r) = t.ticks().quot_rem(&UC1::beat());
        h = fnv(h, &q.to_dec_string());
        h = fnv(h, &r.to_dec_string());
    }

    if RULE_W_DIGEST == 0 {
        panic!(
            "RULE_W_DIGEST is unset. This run produced {h:#018x} — commit it, then \
             confirm the other backend reaches the same value."
        );
    }
    assert_eq!(
        h, RULE_W_DIGEST,
        "this backend disagrees with the committed Rule W digest over {ROUNDS} \
         generated inputs. Rule W says both backends accept and reject exactly \
         the same values; one of them no longer does."
    );
}
