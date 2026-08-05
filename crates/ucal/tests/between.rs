//! `ucal between` — the arithmetic the command claims, checked.
//!
//! The invariant tests in `certification.rs`, `color_invariant.rs`,
//! `no_earth_units.rs`, `manual_fields.rs`, `tables.rs` and `json_surface.rs`
//! all cover this command as one document among many: that its numbers carry
//! certifications, that colour is decoration, that no Earth unit escapes, that
//! every field is documented, that the table renders at 80 columns and that its
//! JSON surface is baselined.
//!
//! What none of them check is whether the decomposition is *right*. A ladder
//! that renders beautifully and reassembles to the wrong number is the failure
//! this file exists to catch.

use ucal::emit::{Doc, Value};
use ucal_core::backend::TickInt;
use ucal_core::{Tier, Ticks};

const A: &str = "8070205189123984864657505252035637180530466139316558837890625";
const B: &str = "8070205189999984864657505252035637180530466139316558837890625";

fn field<'a>(doc: &'a Doc, key: &str) -> &'a Value {
    doc.fields()
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .unwrap_or_else(|| panic!("`{key}` is not in the document"))
}

/// A field's rendered text, trimmed.
///
/// `rendered_text` pads to the column width it was laid out for, which is the
/// renderer's business and not this file's.
fn text(doc: &Doc, key: &str) -> String {
    field(doc, key).rendered_text().trim().to_string()
}

/// A row or section value's rendered text, trimmed.
fn cell(v: &Value) -> String {
    v.rendered_text().trim().to_string()
}

fn int(s: &str) -> Ticks {
    <Ticks as TickInt>::from_dec_str(s).expect("a decimal integer")
}

/// The tier a row label names: `T4 drift` and `T-12 tick` both start with it.
fn tier_of_label(label: &str) -> Tier {
    let id = label.split_whitespace().next().expect("a tier id");
    let k: i16 = id.trim_start_matches('T').parse().expect("a tier index");
    Tier::all_descending()
        .find(|t| t.to_string() == format!("T{k}"))
        .expect("a tier on the grid")
}

/// The decomposition reassembles to the difference, exactly.
///
/// This is the whole claim. Every row is a whole count of its tier, and the
/// tiers are powers of five, so the sum is exact — there is nothing to round and
/// a discrepancy of one tick is a defect rather than a tolerance.
#[test]
fn the_ladder_reassembles_to_the_difference() {
    for (from, to) in [(A, B), (B, A), (A, A)] {
        let doc = ucal::cmd_between(from, to, None).unwrap();
        let rows = doc.rows("on_the_ladder").expect("the decomposition");

        let mut sum = <Ticks as TickInt>::zero();
        for (label, v) in rows {
            let whole = int(&cell(v));
            let contribution = whole
                .try_mul(&tier_of_label(label).ticks())
                .expect("within the domain");
            sum = sum.try_add(&contribution).expect("within the domain");
        }

        assert_eq!(
            sum.to_dec_string(),
            text(&doc, "ticks"),
            "the ladder does not reassemble for {from} -> {to}"
        );
    }
}

/// Reversing the order keeps the magnitude and flips the statement.
///
/// [`ucal_core::Instant::between`] returns a `Signed` because the domain is
/// unsigned (Rule Z) and a difference need not be. Taking the magnitude quietly
/// would make these two documents identical, which is the convenience Rule Q
/// refuses — so the test is that they are *not*.
#[test]
fn the_sign_is_reported_rather_than_absorbed() {
    let forward = ucal::cmd_between(A, B, None).unwrap();
    let backward = ucal::cmd_between(B, A, None).unwrap();

    assert_eq!(
        text(&forward, "ticks"),
        text(&backward, "ticks"),
        "the magnitude must not depend on the order"
    );
    assert_ne!(
        text(&forward, "direction"),
        text(&backward, "direction"),
        "reversing the arguments must change what the document says"
    );
    assert!(text(&forward, "direction").contains("later"));
    assert!(text(&backward, "direction").contains("earlier"));
}

/// `--at` is a divmod, and says so arithmetically.
#[test]
fn at_a_tier_is_a_whole_count_and_a_remainder() {
    for tier in [Tier::BEAT, Tier::DEEP, Tier::SPARK, Tier::TICK] {
        let doc = ucal::cmd_between(A, B, Some(tier)).unwrap();
        let at = field(&doc, "at").as_rows().expect("a section");
        let get = |k: &str| {
            at.iter()
                .find(|(n, _)| n == k)
                .map(|(_, v)| cell(v))
                .unwrap_or_else(|| panic!("`at.{k}` is missing"))
        };

        let whole = int(&get("whole"));
        let rem = int(&get("remainder_ticks"));
        let total = whole
            .try_mul(&tier.ticks())
            .and_then(|w| w.try_add(&rem))
            .expect("within the domain");

        assert_eq!(
            total.to_dec_string(),
            text(&doc, "ticks"),
            "whole x {tier} + remainder != ticks"
        );
        assert!(
            rem < tier.ticks(),
            "the remainder at {tier} is not reduced"
        );
    }
}

/// Zero is a difference like any other, and does not become an error or a gap.
#[test]
fn the_same_instant_is_zero_and_has_no_tier() {
    let doc = ucal::cmd_between(A, A, None).unwrap();
    assert_eq!(text(&doc, "ticks"), "0");
    assert_eq!(text(&doc, "direction"), "the same instant");
    assert!(
        text(&doc, "natural_tier").starts_with('—'),
        "zero is contained by no tier, and the field should say so"
    );
    // Still one row: an empty table would read as a rendering failure.
    assert_eq!(doc.rows("on_the_ladder").expect("rows").len(), 1);
}

/// Leading zeros are dropped, and an interior zero is kept.
///
/// The two are different: a leading zero says "nothing this coarse", which the
/// `natural_tier` field already says once; an interior zero is a digit of the
/// answer, and dropping it would make the rows unreassemblable.
#[test]
fn leading_zeros_go_and_interior_zeros_stay() {
    // Exactly one deep, and nothing else: 5^85 ticks apart.
    let lo = <Ticks as TickInt>::zero();
    let hi = Tier::DEEP.ticks();
    let doc = ucal::cmd_between(&lo.to_dec_string(), &hi.to_dec_string(), None).unwrap();
    let rows = doc.rows("on_the_ladder").expect("rows");

    assert_eq!(rows[0].0.split_whitespace().next(), Some("T5"), "{rows:?}");
    assert_eq!(cell(&rows[0].1), "1");
    // Every tier below it is present and zero — dropped only while leading.
    assert!(rows.len() > 1, "the tiers below deep were dropped");
    assert!(
        rows[1..].iter().all(|(_, v)| cell(v) == "0"),
        "one whole deep should leave nothing over"
    );
}
