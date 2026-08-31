//! `ucal add` — the operation this program did not have.
//!
//! It could **read** time (`now`, `to-civil`, `cal from`) and **measure** it
//! (`between`, `explain`) and not move through it. `seq` walks between two
//! instants and needs both; `between` measures a span whose ends you already
//! hold. There was no way to say *this instant, plus one Martian year*.
#![cfg(all(feature = "body", feature = "civil"))]

use ucal::Stride;
use ucal_core::backend::TickInt;
use ucal_core::{Code, Ticks};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

fn ticks_of(doc: &ucal::emit::Doc) -> String {
    doc.fields()
        .iter()
        .find(|(k, _)| k == "ticks")
        .map(|(_, v)| v.rendered_text().trim().to_string())
        .expect("a ticks field")
}

/// **Moving out and back returns exactly where it started.**
///
/// Exact because `n × unit` is integer ticks and [`Stride`] refuses a unit that
/// is not a whole number of them — so there is no rounding to accumulate, on any
/// unit, in either direction.
#[test]
fn moving_out_and_back_is_exact() {
    for spec in ["T1", "T0", "mars-d", "mars-d-year", "earth-d"] {
        let unit = Stride::calendar(spec)
            .or_else(|_| ucal::parse_tier(spec).map(Stride::tier))
            .unwrap_or_else(|e| panic!("{spec}: {e}"));
        for n in [1i64, 7, 100, 4_000] {
            let out = ucal::cmd_add(T, n, &unit).unwrap_or_else(|e| panic!("{spec}: {e}"));
            let back = ucal::cmd_add(&ticks_of(&out), -n, &unit)
                .unwrap_or_else(|e| panic!("{spec}: {e}"));
            assert_eq!(
                ticks_of(&back),
                T,
                "{spec}: +{n} then -{n} did not return to the start"
            );
        }
    }
}

/// **And the span it opens measures exactly what was asked for.**
///
/// `between(t, add(t, n, u))` counted in `u` must be `n` whole with **no**
/// remainder. This is the pair of operations agreeing about one unit, which is
/// the argument for `add` and `--at` sharing a vocabulary rather than having
/// two.
#[test]
fn the_span_it_opens_measures_back_to_the_same_count() {
    for spec in ["T1", "mars-d", "mars-d-year"] {
        let unit = Stride::calendar(spec)
            .or_else(|_| ucal::parse_tier(spec).map(Stride::tier))
            .unwrap_or_else(|e| panic!("{spec}: {e}"));
        let moved = ucal::cmd_add(T, 7, &unit).expect("moves");
        let doc = ucal::cmd_between_in(T, &ticks_of(&moved), Some(unit)).expect("measures");
        let Some(ucal::emit::Value::Section(at)) = doc.get("at") else {
            panic!("{spec}: no `at` section");
        };
        let field = |k: &str| {
            at.iter()
                .find(|(n, _)| n == k)
                .map(|(_, v)| v.rendered_text().trim().to_string())
                .unwrap_or_else(|| panic!("{spec}: no `{k}`"))
        };
        assert_eq!(field("whole"), "7", "{spec}");
        assert_eq!(field("remainder_ticks"), "0", "{spec}: not an exact span");
    }
}

/// **Below the datum is an error, not a negative and not a clamp.**
///
/// Absolute time is unsigned (Rule B) and `Ticks` cannot hold a negative, so
/// there is no instant before tick 0. Rule O forbids saturating: clamping to the
/// datum would be a wrong answer where an error was available.
///
/// `UCAL-E0020` is declared *result precedes the datum* — the code named for
/// this operation, which until now no raiser produced for it.
#[test]
fn moving_below_the_datum_is_refused() {
    let unit = Stride::tier(ucal::parse_tier("T1").expect("a tier"));
    let e = ucal::cmd_add("0", -1, &unit).expect_err("there is nothing before the datum");
    assert_eq!(e.code, Code::E0020);
    assert!(e.to_string().contains("unsigned"), "{e}");
}

/// Past the ceiling is `UCAL-E0021`, and also not clamped.
#[test]
fn moving_past_the_ceiling_is_refused() {
    let unit = Stride::tier(ucal::parse_tier("T32").expect("a tier"));
    let e = ucal::cmd_add(T, 100, &unit).expect_err("beyond 2^512 - 1");
    assert_eq!(e.code, Code::E0021);
}

/// Zero moves nowhere, which is worth pinning: the sign branch must not treat
/// `0` as negative and try to subtract.
#[test]
fn zero_is_the_same_instant() {
    let unit = Stride::calendar("mars-d").expect("a unit");
    let out = ucal::cmd_add(T, 0, &unit).expect("moves nowhere");
    assert_eq!(ticks_of(&out), T);
}

/// **`--at` widened rather than gaining a second flag.**
///
/// It has always meant *express this span in this unit* and could only name a
/// tier. A calendar unit leaves `at.tier` absent — a path in `ucal-json/1`,
/// which is why it is still emitted for a tier rather than renamed.
#[test]
fn at_keeps_its_tier_field_for_a_tier_and_omits_it_otherwise() {
    let tier = Stride::tier(ucal::parse_tier("T1").expect("a tier"));
    let doc = ucal::cmd_between_in("0", T, Some(tier)).expect("measures");
    let Some(ucal::emit::Value::Section(at)) = doc.get("at") else {
        panic!("no `at`");
    };
    assert!(at.iter().any(|(k, _)| k == "tier"), "a tier lost `at.tier`");
    assert!(at.iter().any(|(k, _)| k == "unit"));

    let cal = Stride::calendar("mars-d").expect("a unit");
    let doc = ucal::cmd_between_in("0", T, Some(cal)).expect("measures");
    let Some(ucal::emit::Value::Section(at)) = doc.get("at") else {
        panic!("no `at`");
    };
    assert!(
        !at.iter().any(|(k, _)| k == "tier"),
        "a Martian day was reported as a tier"
    );
}

/// A unit that cannot be a whole number of ticks is refused here too, because
/// `add` and `seq` take the same vocabulary and it refuses in one place.
#[test]
fn a_derived_solar_day_is_still_refused_as_a_unit() {
    let e = Stride::calendar("europa-d").expect_err("europa's solar day is derived");
    assert_eq!(e.code, Code::E0043);
    // And the magnitude a caller might have wanted is genuinely not an integer,
    // so this is a fact about the body rather than a limitation of `add`.
    let _ = <Ticks as TickInt>::one();
}
