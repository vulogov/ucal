//! Compile-fail tests required by §21.3.
//!
//! Assertions 3 and 11 are not testable at runtime by construction: they say that
//! certain programs must not *exist*. `trybuild` compiles each case and asserts it
//! fails with the expected diagnostic, so a refactor that quietly re-opens the
//! path — say by adding a `From<SignedWindow> for Delta` — turns into a test
//! failure rather than a silent loss of the guarantee.

#[test]
fn rule_q3_and_rule_p_are_enforced_by_the_type_system() {
    let t = trybuild::TestCases::new();
    // Rule Q.3 — metadata cannot reach arithmetic (§21.3-3).
    t.compile_fail("tests/compile_fail/signed_window_as_operand.rs");
    t.compile_fail("tests/compile_fail/signed_window_into_delta.rs");
    t.compile_fail("tests/compile_fail/signed_window_arithmetic.rs");
    t.compile_fail("tests/compile_fail/frame_bridge_claim_as_operand.rs");

    // And the other direction: `Profile` must stay implementable from outside
    // this crate. `cargo semver-checks` does not catch a required method added
    // to a trait, so this is what does. See the fixture's header.
    t.pass("tests/compile_pass/profile_is_implementable.rs");
    t.pass("tests/compile_pass/calendar_identity_is_implementable.rs");
    t.pass("tests/compile_pass/tick_int_is_implementable.rs");
    // Rule U — a window cannot silently collapse to an instant.
    t.compile_fail("tests/compile_fail/window_into_instant.rs");
    // Rule T — a stated value cannot be used as a tick-precise one.
    t.compile_fail("tests/compile_fail/stated_is_not_an_instant.rs");
    // Rule P — profiles do not mix (§21.3-11).
    t.compile_fail("tests/compile_fail/cross_profile_arithmetic.rs");
    t.compile_fail("tests/compile_fail/cross_profile_comparison.rs");
    // Rule F — a profile that declares no frame does not exist.
    t.compile_fail("tests/compile_fail/profile_without_a_frame.rs");
}
