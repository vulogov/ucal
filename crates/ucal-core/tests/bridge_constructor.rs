//! A1 — `Bridge::new`, and the invariant it derives rather than accepts.
//!
//! `Bridge` was `#[non_exhaustive]` with no constructor, so a downstream
//! `Profile` had to borrow a shipped bridge. A second profile could pick its own
//! datum and its own frame and was obliged to convert through *this* profile's
//! second, which is a stranger limitation than anything the specification
//! declares.

use ucal_core::backend::TickInt;
use ucal_core::profile::Bridge;
use ucal_core::{Profile, Ticks, UC1};

/// **The check that matters.** The derived divisibility is what `UC-1` declares.
///
/// `UC1::bridge` keeps a literal: the value is a `const` under §3.3 and the
/// derivation is a loop of divisions on a path that runs on every conversion.
/// Two ways of knowing one number is exactly the arrangement that drifts, so
/// this is the test that says they agree.
#[test]
fn the_derived_divisibility_is_what_uc1_declares() {
    let shipped = UC1::bridge();
    let rebuilt = Bridge::new(shipped.name, shipped.ticks.clone(), shipped.epoch_label);
    assert_eq!(
        rebuilt.divisibility, shipped.divisibility,
        "SECOND is 18548584399861 x 10^30, so five divides it exactly thirty times"
    );
    assert_eq!(rebuilt.divisibility, 30);
}

/// The derivation is arithmetic, not a lookup, and it answers for any input.
#[test]
fn the_valuation_counts_the_fives() {
    let cases: [(u64, u32); 6] = [
        (1, 0),
        (2, 0),
        (5, 1),
        (25, 2),
        (100, 2),      // 2^2 x 5^2
        (3_125, 5),    // 5^5, one tier
    ];
    for (n, want) in cases {
        let b = Bridge::new("probe", <Ticks as TickInt>::from_u64(n), "probe");
        assert_eq!(b.divisibility, want, "for {n}");
    }
}

/// Zero terminates, and says nothing rather than looping.
///
/// Every power of five divides zero. The honest answer is that a bridge of zero
/// ticks is not a bridge; what must not happen is a loop that does not end.
#[test]
fn a_zero_bridge_does_not_hang() {
    let b = Bridge::new("nothing", <Ticks as TickInt>::zero(), "nowhere");
    assert_eq!(b.divisibility, 0);
}

/// A downstream profile can now make a bridge instead of borrowing one.
///
/// The point of the whole exercise, asserted as a fact about the type rather
/// than described in a comment.
#[test]
fn a_bridge_can_be_built_from_its_parts() {
    let b = Bridge::new(
        "kilosecond",
        <Ticks as TickInt>::from_u64(1_000)
            .try_mul(&UC1::bridge().ticks)
            .expect("inside the domain"),
        "an epoch of the caller's choosing",
    );
    assert_eq!(b.name, "kilosecond");
    // 1000 = 2^3 x 5^3, so three more fives than the second has.
    assert_eq!(b.divisibility, UC1::bridge().divisibility + 3);
}
