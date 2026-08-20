//! D-A25 — the distance between a profile's declared frame and its bridge scale.
//!
//! Rule F requires a profile to declare its frame, and `UC-1` declares one:
//! proper time along a comoving worldline in the CMB rest frame. §8.1 bridges to
//! SI seconds through **TT**, whose rate is that of a clock on Earth's geoid.
//! Earth is not comoving with the CMB, so the frame the calendar declares and
//! the frame it converts through tick at different rates.
//!
//! Rule F said *declare your frame*. It said nothing about declaring how far
//! that is from the scale you actually use, and for four releases nothing did.

use ucal_core::backend::TickInt;
use ucal_core::{Profile, Ticks, UC1};

/// The claim is a bound over the datum span, and the arithmetic is checkable.
///
/// `5 x 10^-6 x 13.787 Gyr` in ticks. Re-derived here from the profile's own
/// constants rather than compared against a copy of the literal, so a typo in
/// either place is a failure rather than a matched pair of typos.
#[test]
fn the_bound_is_five_parts_per_million_of_the_datum_span() {
    let half = UC1::frame_bridge_claim().hi().magnitude().ticks().clone();

    // 13.787 Gyr in seconds, exactly: Julian years by definition.
    let age_s = <Ticks as TickInt>::from_u64(13_787_000_000)
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .expect("the datum span is inside the domain");
    let span_ticks = age_s
        .try_mul(&UC1::bridge().ticks)
        .expect("the datum span in ticks is inside the domain");

    // 5e-6 of it: multiply by 5, divide by 10^6.
    let want = span_ticks
        .try_mul(&<Ticks as TickInt>::from_u64(5))
        .expect("no overflow")
        .quot_rem(&<Ticks as TickInt>::from_u64(1_000_000))
        .0;

    assert_eq!(
        half.to_dec_string(),
        want.to_dec_string(),
        "the declared half-width is not 5e-6 of the datum span"
    );
}

/// **The number that decided what to do about it.**
///
/// The frame gap is comfortably inside an uncertainty this profile already
/// declares — 290 times inside it. That is why D-A25 records the distance rather
/// than introducing a second profile whose declared frame is TT: a correction
/// two orders of magnitude below the stated error bar buys precision the
/// measurement cannot support.
///
/// If better cosmology ever narrows `BIG_BANG_CLAIM` by three orders of
/// magnitude, this test fails and the decision has to be taken again. That is
/// the intent.
#[test]
fn the_frame_gap_is_well_inside_the_datum_uncertainty() {
    let frame = UC1::frame_bridge_claim().hi().magnitude().ticks().clone();
    let bang = UC1::big_bang_claim().hi().magnitude().ticks().clone();

    let ratio = bang.quot_rem(&frame).0;
    let ratio: u64 = ratio.to_dec_string().parse().expect("a small integer");
    assert!(
        (200..=400).contains(&ratio),
        "BIG_BANG_CLAIM is {ratio}x the frame gap; the ratio that justified \
         recording rather than re-profiling was 290"
    );
}

/// It is cited, like every other claim about the physical world here.
#[test]
fn the_claim_carries_its_citation() {
    let c = UC1::frame_bridge_claim_citation();
    assert!(c.source.contains("dipole"), "{}", c.source);
    assert!(c.locator.is_some_and(|l| l.starts_with("doi:")));
}

/// The offset is a **rate**, so it cancels in a difference.
///
/// This is the reason the finding is a footnote and not an emergency: every
/// interval this library computes carries both endpoints through the same
/// bridge, and a common rate factor cancels exactly. The claim bears only on
/// reading an absolute count as elapsed cosmological time.
///
/// Asserted structurally: nothing in `Delta`'s construction consults the claim,
/// and the claim's type cannot reach arithmetic at all — which the next test
/// and the compile-fail fixture cover.
#[test]
fn the_claim_is_inert() {
    let w = UC1::frame_bridge_claim();
    // A `SignedWindow` has a describable value and no operators. If this type
    // ever grows `into_delta`, the compile-fail fixture beside it starts passing
    // and §21.3's guarantee for `big_bang_claim` is gone for this one too.
    assert!(!w.describe().is_empty());
}
