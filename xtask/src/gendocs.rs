//! Documentation generation (§13.5).
//!
//! §13.5: "The tier table, the locale table, and the docs table in §4.1 MUST be
//! generated from one source of truth so they cannot drift."
//!
//! The source of truth is `ucal_core::tier` plus `ucal_core::locale`. This module
//! renders them to `docs/TIERS.md`, and `cargo run -p xtask -- check-docs`
//! re-renders and compares — so a change to the grid or a locale that is not
//! reflected in the documentation fails CI rather than going unnoticed.
//!
//! It also closes delta D-A3. Appendix B's seconds column disagreed with exact
//! evaluation in the fifth significant figure for the upper tiers, in a pattern
//! consistent with chain-computing each row from its neighbour by ×3125 and
//! keeping the rounding. Generated rows cannot drift that way: every value here
//! is rendered from the exact rational `5^e / SECOND` in one step.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ucal_core::backend::TickInt;
use ucal_core::locale::{self, LocaleId};
use ucal_core::num::Ratio;
use ucal_core::tier::{Tier, K_MAX, K_MIN};
use ucal_core::{Profile, Rounding, Ticks, UC1};

/// Where the generated table lives.
pub fn docs_path(root: &Path) -> PathBuf {
    root.join("docs").join("TIERS.md")
}

fn pow10(e: u32) -> Ratio {
    let ten = <Ticks as TickInt>::from_u64(10);
    let mut acc = <Ticks as TickInt>::one();
    for _ in 0..e {
        acc = acc.try_mul(&ten).expect("within domain");
    }
    Ratio::from_int(acc)
}

/// Exact scientific notation: `d.ddddde±XX`, from a rational, with no float.
///
/// The exponent is found by comparing against powers of ten rather than by a
/// logarithm, and the mantissa is rendered under an explicit rounding mode
/// (Rule R). Nothing here approximates until the final digit.
fn scientific(value: &Ratio, digits: u32) -> String {
    if value.is_zero() {
        return "0".to_string();
    }
    // A first guess from the decimal lengths, then corrected. The guess is never
    // more than one out, but it is checked rather than trusted.
    let num_len = value.numer().to_dec_string().len() as i32;
    let den_len = value.denom().to_dec_string().len() as i32;
    let mut k = num_len - den_len;

    let ten = Ratio::from_u64(10);
    let one = Ratio::one();
    loop {
        let scaled = scale_by_pow10(value, -k);
        if scaled.cmp_exact(&one) == std::cmp::Ordering::Less {
            k -= 1;
        } else if scaled.cmp_exact(&ten) != std::cmp::Ordering::Less {
            k += 1;
        } else {
            let m = scaled
                .to_decimal_string(digits, Rounding::HalfEven)
                .unwrap_or_else(|_| scaled.to_ratio_string());
            // Rounding the mantissa can carry it to 10.0000; renormalise.
            if m.starts_with("10") {
                k += 1;
                continue;
            }
            return format!("{m}e{}{}", if k < 0 { "-" } else { "+" }, k.abs());
        }
    }
}

/// `value * 10^e` for a signed exponent, exactly.
fn scale_by_pow10(value: &Ratio, e: i32) -> Ratio {
    if e >= 0 {
        value.mul(&pow10(e as u32)).expect("within domain")
    } else {
        value.div(&pow10((-e) as u32)).expect("non-zero")
    }
}

/// A tier's magnitude in bridge units, exactly.
fn in_bridge_units(tier: Tier) -> Ratio {
    Ratio::new(tier.ticks(), UC1::bridge().ticks).expect("non-zero bridge constant")
}

/// A tier's magnitude in **beats** — the universe second (§0.5).
///
/// Exact and always a power of five, because the beat is itself a tier. This is
/// the ladder's own unit; the bridge column beside it is a foreign one.
fn in_beats(tier: Tier) -> Ratio {
    Ratio::new(tier.ticks(), UC1::beat()).expect("non-zero beat")
}

/// A human-scale rendering: the largest familiar unit that gives a value at
/// least one, with three decimals.
///
/// Informative only (Rule A.5). It exists because the ladder is deliberately
/// unfamiliar — nothing on it is near a second or an hour (D-2) — and a reader
/// needs *some* purchase on the magnitudes.
fn human_scale(tier: Tier) -> String {
    let s = in_bridge_units(tier);
    // (label, size in bridge units), descending.
    const JULIAN_YEAR: u64 = 31_557_600;
    let units: [(&str, u64); 6] = [
        ("Gyr", JULIAN_YEAR * 1_000_000_000),
        ("Myr", JULIAN_YEAR * 1_000_000),
        ("kyr", JULIAN_YEAR * 1_000),
        ("yr", JULIAN_YEAR),
        ("d", 86_400),
        ("s", 1),
    ];
    for (label, size) in units {
        let size_r = Ratio::from_u64(size);
        if s.cmp_exact(&size_r) != std::cmp::Ordering::Less {
            let v = s.div(&size_r).expect("non-zero");
            let rendered = v
                .to_decimal_string(3, Rounding::HalfEven)
                .unwrap_or_default();
            // Beyond a few thousand Gyr there is no familiar unit left, and a
            // ninety-digit count of gigayears gives a reader no purchase at all.
            // The scientific column already carries the magnitude, so say nothing
            // rather than say it unreadably.
            if rendered.split('.').next().map(str::len).unwrap_or(0) > 7 {
                return "—".to_string();
            }
            return format!("{rendered} {label}");
        }
    }
    // Below one bridge unit: sub-multiples, down to the yoctosecond. The tick
    // itself is 5.39e-44, twenty orders below even that, which is the point.
    let sub: [(&str, u32); 8] = [
        ("ms", 3),
        ("us", 6),
        ("ns", 9),
        ("ps", 12),
        ("fs", 15),
        ("as", 18),
        ("zs", 21),
        ("ys", 24),
    ];
    for (label, e) in sub {
        let scaled = scale_by_pow10(&s, e as i32);
        if scaled.cmp_exact(&Ratio::one()) != std::cmp::Ordering::Less {
            let rendered = scaled
                .to_decimal_string(3, Rounding::HalfEven)
                .unwrap_or_default();
            return format!("{rendered} {label}");
        }
    }
    "—".to_string()
}

/// Render the generated documentation.
pub fn render() -> String {
    let mut s = String::new();
    let _ = writeln!(s, "# The tier grid");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "**Generated from `ucal_core::tier` and `ucal_core::locale` by \
         `cargo run -p xtask -- gen-docs`. Do not edit by hand.**"
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "§13.5 requires the tier table, the locale table and the documentation \
         table to come from one source of truth so they cannot drift. This file \
         is that requirement discharged; `cargo run -p xtask -- check-docs` fails \
         if it is stale."
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "Rule G: tiers are the powers `5^(5k)`, indexed relative to the beat, so \
         `T[k] = 5^(60 + 5k)`. Each tier is exactly five base-5 digits — 3125 \
         units of the tier below. Rule N: a tier's canonical identity is its \
         **exponent**; the names are display aliases and nothing decides \
         behaviour from one."
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "The **beat** column is the ladder's own unit. §0.5 names the beat the \
         *universe second*: 5^60 ticks, a pure power of the tick with no Earth \
         content. Every tier is a whole power of five of it, so those values are \
         exact by construction."
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "The **bridge** column is a foreign unit, shown alongside as §4.3 \
         requires and never instead. Note that the two seconds are incommensurable \
         above T-6: one bridge second is 21.385061835 beats, because `BEAT` carries \
         `5^60` while `SECOND` carries only `5^30`. They share a common measure \
         only at the tick — which is why Rule A.1 makes the tick primitive rather \
         than either second."
    );
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "The bridge column is informative (Rule A.5). It is rendered from the \
         exact rational `5^e / SECOND` under half-even rounding, in one step — \
         not chained from the neighbouring row, which is how Appendix B's \
         published column came to disagree in the fifth significant figure \
         (delta D-A3)."
    );
    let _ = writeln!(s);

    let _ = writeln!(
        s,
        "| k | exponent | beats (universe seconds) | bridge units | human | en | ru | ticks |"
    );
    let _ = writeln!(s, "|---:|---:|---:|---:|---:|---|---|---:|");
    for k in (K_MIN..=K_MAX).rev() {
        let tier = Tier::new(k).expect("on the grid");
        let en = locale::names_of(LocaleId::En, tier)
            .map(|n| n.singular)
            .unwrap_or("—");
        let ru = locale::names_of(LocaleId::Ru, tier)
            .map(|n| n.singular)
            .unwrap_or("—");
        let ticks = tier.ticks().to_dec_string();
        let ticks = if ticks.len() > 24 {
            format!("`5^{}`", tier.exponent())
        } else {
            format!("`{ticks}`")
        };
        let _ = writeln!(
            s,
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            k,
            tier.exponent(),
            scientific(&in_beats(tier), 4),
            scientific(&in_bridge_units(tier), 4),
            human_scale(tier),
            en,
            ru,
            ticks
        );
    }
    let _ = writeln!(s);
    let _ = writeln!(s, "## Notes");
    let _ = writeln!(s);
    let _ = writeln!(
        s,
        "- **T32 is the ceiling.** `5^220` is 511 bits, the largest power of five \
         the 512-bit domain holds, so the grid cannot extend further without \
         widening the domain — and Rule B makes the width a wire-format \
         commitment (D-4)."
    );
    let _ = writeln!(
        s,
        "- **T−12 is the floor.** One tick is the finest addressable interval \
         (G2). There is no sub-tick representation and intervals shorter than one \
         tick must not be approximated (N10)."
    );
    let _ = writeln!(
        s,
        "- **Unnamed tiers are not second-class.** D-20 leaves everything above T5 \
         and below T−3 unnamed and addressable by index; Rule N requires `T[k]` \
         and `5^e` to be accepted wherever a name is."
    );
    let _ = writeln!(
        s,
        "- **Nothing on the ladder is near a second or an hour.** That is the \
         accepted cost of leaving the Earth paradigm (D-2), which is why the \
         bridge column is always printed alongside."
    );
    s
}

/// Write the generated documentation.
pub fn write(root: &Path) -> std::io::Result<PathBuf> {
    let path = docs_path(root);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    std::fs::write(&path, render())?;
    Ok(path)
}

/// Whether the committed documentation matches what generation produces.
pub fn check(root: &Path) -> Result<(), String> {
    let path = docs_path(root);
    let want = render();
    match std::fs::read_to_string(&path) {
        Err(_) => Err(format!(
            "{} is missing; run `cargo run -p xtask -- gen-docs`",
            path.display()
        )),
        Ok(got) if got == want => Ok(()),
        Ok(got) => {
            let (gl, wl) = (got.lines().count(), want.lines().count());
            let first = got
                .lines()
                .zip(want.lines())
                .position(|(a, b)| a != b)
                .map(|i| i + 1);
            Err(format!(
                "{} is stale ({gl} lines committed, {wl} generated{}). \
                 Run `cargo run -p xtask -- gen-docs`.",
                path.display(),
                first
                    .map(|l| format!("; first difference at line {l}"))
                    .unwrap_or_default()
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scientific_notation_is_exact_and_normalised() {
        // Every mantissa must lie in [1, 10).
        for k in K_MIN..=K_MAX {
            let t = Tier::new(k).unwrap();
            let s = scientific(&in_bridge_units(t), 4);
            let mantissa: f64_free::Mantissa = f64_free::Mantissa::parse(&s);
            assert!(
                mantissa.leading >= 1 && mantissa.leading <= 9,
                "T{k} rendered as {s}, mantissa not normalised"
            );
        }
    }

    #[test]
    fn known_tiers_match_appendix_b_where_appendix_b_is_right() {
        // Appendix B's *human* column is correct; its seconds column is not
        // (D-A3). These are the human values.
        assert_eq!(human_scale(Tier::DEEP), "441.607 Myr");
        assert_eq!(human_scale(Tier::DRIFT), "141.314 kyr");
        assert_eq!(human_scale(Tier::SPAN), "45.221 yr");
        assert_eq!(human_scale(Tier::SWEEP), "5.285 d");
        assert_eq!(human_scale(Tier::ARC), "146.130 s");
        assert_eq!(human_scale(Tier::BEAT), "46.762 ms");
        assert_eq!(human_scale(Tier::FLICKER), "14.964 us");
        assert_eq!(human_scale(Tier::GLINT), "4.788 ns");
        assert_eq!(human_scale(Tier::SPARK), "1.532 ps");
    }

    #[test]
    fn the_beat_column_is_exact_powers_of_five() {
        // The ladder measured in its own unit is nothing but powers of five —
        // T[k] is exactly 5^(5k) beats. That is what it means for the beat to be
        // the universe second (§0.5) rather than a foreign one: no rounding, no
        // remainder, at any tier.
        for k in K_MIN..=K_MAX {
            let tier = Tier::new(k).unwrap();
            let v = in_beats(tier);
            let five_to = |e: u32| {
                let mut acc = <Ticks as TickInt>::one();
                for _ in 0..e {
                    acc = acc.try_mul(&<Ticks as TickInt>::from_u64(5)).unwrap();
                }
                acc
            };
            let expect = if k >= 0 {
                Ratio::from_int(five_to(5 * k as u32))
            } else {
                Ratio::new(<Ticks as TickInt>::one(), five_to(5 * (-k) as u32)).unwrap()
            };
            assert_eq!(v, expect, "T{k} is not an exact power of five in beats");
        }
        // The beat is one beat.
        assert_eq!(in_beats(Tier::BEAT), Ratio::one());
    }

    #[test]
    fn the_two_seconds_are_incommensurable_above_the_shared_tier() {
        // One bridge second is 21.385061835 beats, not a whole number: BEAT
        // carries 5^60 while SECOND carries only 5^30, so they share a common
        // measure only at T-6. This is why Rule A.1 makes the *tick* primitive
        // rather than either second.
        let one_bridge_second_in_beats =
            Ratio::new(UC1::bridge().ticks, UC1::beat()).unwrap();
        assert!(
            !one_bridge_second_in_beats.is_integer(),
            "if this ever became an integer the ladder and the bridge would share \
             a unit above the tick, and the docs above would be wrong"
        );
        assert_eq!(
            one_bridge_second_in_beats
                .to_decimal_string(9, Rounding::HalfEven)
                .unwrap(),
            "21.385061835"
        );
        // ...and the beat, conversely, is not a whole number of bridge seconds.
        let beat_in_seconds = in_bridge_units(Tier::BEAT);
        assert!(!beat_in_seconds.is_integer());
    }

    #[test]
    fn the_generated_seconds_column_corrects_appendix_b() {
        // D-A3: Appendix B prints 1.3934e16 for the deep; the exact value is
        // 1.3936e16. The generated column must carry the exact one.
        let deep = scientific(&in_bridge_units(Tier::DEEP), 4);
        assert_eq!(deep, "1.3936e+16", "got {deep}");
        assert_ne!(deep, "1.3934e+16", "Appendix B's published value is wrong");

        let drift = scientific(&in_bridge_units(Tier::DRIFT), 4);
        assert_eq!(drift, "4.4595e+12");
        // ...and the tick, where Appendix B is right.
        let tick = scientific(&in_bridge_units(Tier::TICK), 4);
        assert_eq!(tick, "5.3912e-44");
    }

    #[test]
    fn every_grid_row_is_rendered() {
        let doc = render();
        for k in K_MIN..=K_MAX {
            assert!(
                doc.contains(&format!("| {k} | {} |", Tier::new(k).unwrap().exponent())),
                "T{k} is missing from the generated table"
            );
        }
        // Both locales appear.
        assert!(doc.contains("глубь"));
        assert!(doc.contains("deep"));
        // Unnamed tiers show an em dash rather than being omitted.
        assert!(doc.contains("| — | — |"));
    }

    /// A tiny mantissa parser, so the test needs no float (Rule E).
    mod f64_free {
        pub struct Mantissa {
            pub leading: u32,
        }
        impl Mantissa {
            pub fn parse(s: &str) -> Mantissa {
                let leading = s
                    .bytes()
                    .find(|b| b.is_ascii_digit())
                    .map(|b| (b - b'0') as u32)
                    .unwrap_or(0);
                Mantissa { leading }
            }
        }
    }
}

#[cfg(test)]
mod drift {
    use super::*;

    fn root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("xtask lives under the workspace root")
            .to_path_buf()
    }

    /// §13.5's drift test: the committed table must equal what generation
    /// produces. A change to the tier grid or to a locale that is not reflected
    /// in the documentation fails here rather than going unnoticed.
    #[test]
    fn generated_docs_are_current() {
        check(&root()).unwrap_or_else(|e| panic!("{e}"));
    }

    #[test]
    fn generation_is_deterministic() {
        // A generator whose output depended on iteration order or hashing would
        // make the drift test flap rather than catch anything.
        assert_eq!(render(), render());
    }

    #[test]
    fn the_committed_table_covers_the_whole_grid() {
        let committed = std::fs::read_to_string(docs_path(&root())).expect("generated docs");
        let rows = committed
            .lines()
            .filter(|l| l.starts_with("| ") && !l.starts_with("| k "))
            .count();
        assert_eq!(
            rows,
            (K_MAX as i32 - K_MIN as i32 + 1) as usize,
            "the table must have one row per tier"
        );
    }
}
