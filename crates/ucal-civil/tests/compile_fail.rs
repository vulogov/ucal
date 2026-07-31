//! Compile-fail tests for §8.6 and §6.6.
//!
//! Both guarantees are about programs that must not *exist*, so neither can be
//! checked at runtime. `trybuild` compiles each case and asserts it fails.

#[test]
fn legacy_calendars_stay_qualified_and_stay_legacy() {
    let t = trybuild::TestCases::new();
    // §6.6 — a local rendering cannot lose its qualifier.
    t.compile_fail("tests/compile_fail/legacy_fields_cannot_render.rs");
    // Rule K.6 — a legacy calendar cannot stand in for a derivation.
    t.compile_fail("tests/compile_fail/legacy_is_not_derived.rs");
}
