//! §20 UC-P6 exit criterion: **no path yields tick precision from truncated
//! input.**
//!
//! This is failure mode F2 — "precision invented by zero-filling a truncated
//! timestamp" — and F2's metric is an *exhaustive* parse test, so the claim is
//! only worth as much as its coverage. The test below enumerates every public
//! entry point that can accept a truncated notation, crossed with every tier the
//! grid holds, and asserts that none of them reports `Precision::Tick`.
//!
//! It lives in `tests/` deliberately: an integration test can only reach the
//! public API, so "no path" means the paths a user actually has.

use ucal_core::codec::{parse, parse_window, render, Fmt, Form};
use ucal_core::LocaleId;
use ucal_core::tier::{K_MAX, K_MIN};
use ucal_core::{Instant, Precision, Profile, Tier, TickInt, Ticks, UC1};

type I = Instant<UC1>;

/// Every form that can express a truncated value, with the tier range over which
/// it can express one.
///
/// The human form's floor is T0: §6.4's sub-beat separator is its only anchor, so
/// it cannot state a precision coarser than the beat (delta D-A8). The other two
/// forms cover the whole grid.
fn truncating_forms() -> Vec<(&'static str, Form, i8, i8)> {
    vec![
        ("human", Form::HumanGroups, K_MIN, 0),
        ("digit5", Form::Digit5, K_MIN, K_MAX),
        ("named", Form::Named, K_MIN, K_MAX),
    ]
}

fn sample_instants() -> Vec<I> {
    let mut out = vec![
        I::zero(),
        I::from_u64(1).unwrap(),
        I::from_u64(3124).unwrap(),
        I::from_u64(3125).unwrap(),
        I::from_ticks(UC1::origin_offset()).unwrap(),
        I::from_ticks(UC1::beat()).unwrap(),
        I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap(),
    ];
    // A whole SI second: the case where §2.4 guarantees six trailing zero groups,
    // which is exactly what tempts an implementation to drop them.
    let bridge_unit = UC1::bridge().ticks;
    out.push(
        I::from_ticks(
            <Ticks as TickInt>::try_add(&UC1::origin_offset(), &bridge_unit).unwrap(),
        )
        .unwrap(),
    );
    // Deterministic spread across the domain.
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..64 {
        let mut bytes = [0u8; 64];
        for chunk in bytes.chunks_mut(8) {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            chunk.copy_from_slice(&x.to_be_bytes());
        }
        out.push(I::from_ticks(<Ticks as TickInt>::from_canonical_bytes(&bytes).unwrap()).unwrap());
    }
    out
}

fn fmt_for(form: Form, tier: Tier) -> Fmt {
    Fmt {
        form,
        precision: if tier.is_tick() {
            Precision::Tick
        } else {
            Precision::Tier(tier)
        },
        pad: matches!(form, Form::Digit5),
        locale: LocaleId::En,
        ..Fmt::default()
    }
}

/// The named form needs its profile tag prepended before it can be parsed back.
fn tagged(form: Form, s: &str) -> String {
    if matches!(form, Form::Named) {
        format!("{} {}", UC1::TAG, s)
    } else {
        s.to_string()
    }
}

#[test]
fn no_parse_path_reports_tick_precision_from_truncated_input() {
    let mut checked = 0usize;
    for inst in sample_instants() {
        for (name, form, lo_k, hi_k) in truncating_forms() {
            for k in lo_k..=hi_k {
                let tier = Tier::new(k).unwrap();
                let f = fmt_for(form, tier);

                let Ok(text) = render(&inst, &f) else {
                    // A form that cannot express this precision must refuse to,
                    // not approximate it. That refusal is itself the guarantee.
                    continue;
                };
                let s = tagged(form, &text);
                let (value, precision) = parse::<UC1>(&s, &f)
                    .unwrap_or_else(|e| panic!("{name} T{k} failed to round trip: {e} on {s:?}"));
                checked += 1;

                if tier.is_tick() {
                    // Only a rendering that runs to T-12 may claim a tick.
                    assert_eq!(
                        precision,
                        Precision::Tick,
                        "{name} at T-12 must state tick precision"
                    );
                    assert_eq!(value, inst, "{name} at T-12 must be exact");
                } else {
                    // Everything coarser is a window, and says so.
                    assert_ne!(
                        precision,
                        Precision::Tick,
                        "{name} at T{k} invented tick precision from truncated input \
                         (failure mode F2): {s:?}"
                    );
                    assert_eq!(
                        precision,
                        Precision::Tier(tier),
                        "{name} at T{k} misreported its precision"
                    );

                    // The parsed value is the floor, never the original.
                    assert_eq!(value, inst.floor_to(tier), "{name} at T{k}");

                    // And the window it denotes contains the original.
                    let w = parse_window::<UC1>(&s, &f).unwrap();
                    assert!(
                        w.contains(&inst),
                        "{name} at T{k}: window does not contain the value it came from"
                    );
                    assert!(!w.is_exact(), "{name} at T{k}: a truncated form is not a tick");
                }
            }
        }
    }
    // Guard against the loop silently covering nothing.
    assert!(checked > 2000, "only {checked} cases exercised");
}

#[test]
fn a_truncated_form_never_equals_the_value_it_came_from_unless_aligned() {
    // The converse check: when a value is *not* aligned to the tier, truncation
    // must actually lose information rather than round-tripping by luck.
    let inst = I::from_u64(123_456_789).unwrap();
    for k in (K_MIN + 1)..=0 {
        let tier = Tier::new(k).unwrap();
        let f = fmt_for(Form::HumanGroups, tier);
        let s = render(&inst, &f).unwrap();
        let (v, p) = parse::<UC1>(&s, &f).unwrap();
        assert_eq!(p, Precision::Tier(tier));
        if inst.floor_to(tier) != inst {
            assert_ne!(v, inst, "T{k} round-tripped a value it should have truncated");
        }
    }
}

#[test]
fn exact_paths_are_still_exact() {
    // The rule is that truncated input must not yield tick precision — not that
    // nothing may. The binary and UCID paths carry every bit, so they stay exact,
    // and this test exists so a future over-correction cannot quietly make the
    // whole API imprecise.
    for inst in sample_instants() {
        assert_eq!(I::from_bytes(&inst.to_bytes()).unwrap(), inst);
        if let Ok(u) = inst.to_ucid() {
            assert_eq!(I::from_ucid(&u).unwrap(), inst);
        }
        // Tick-precision text is exact too.
        let f = Fmt::human();
        let s = render(&inst, &f).unwrap();
        let (v, p) = parse::<UC1>(&s, &f).unwrap();
        assert_eq!(v, inst);
        assert_eq!(p, Precision::Tick);
    }
}
