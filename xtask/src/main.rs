//! UC-P0 constants harness.
//!
//! Exit criterion (§20): "Appendices A, C, I reproduced by both routines; §2.4
//! invariants hold; provenance chain re-executes to the declared ORIGIN_OFFSET
//! and residual."
//!
//! Three independent things are checked, and they are different claims:
//!
//! 1. **Route agreement** — routes A and B produce identical output. This is
//!    evidence the arithmetic is right, and it is the same differential check
//!    Rule W makes a conformance test at the library level.
//! 2. **RFC agreement** — the computed values match what the RFC transcribes.
//!    Disagreements here are RFC errata, recorded in `spec/SPEC-DELTAS.md`.
//! 3. **Internal invariants** — §2.4 alignment, Rule B width, Rule I range,
//!    §9.5's prohibition on 97/400, §21.3's required assertions.
//!
//! Usage: `cargo run -p xtask -- [check | emit | report]`

mod citations;
mod publish;
mod declared;
mod derivation;
mod gendocs;
mod links;
mod lint;
mod route_bigint;
mod route_bnum;

use std::fmt::Write as _;

use declared as d;
use derivation::Derivation;
use sha2::{Digest, Sha256};

/// One checked claim.
struct Check {
    name: String,
    passed: bool,
    detail: String,
    /// Set when the failure is a known RFC erratum rather than a harness fault.
    delta: Option<&'static str>,
}

struct Report {
    checks: Vec<Check>,
}

impl Report {
    fn new() -> Self {
        Report { checks: Vec::new() }
    }

    fn eq<A: PartialEq + std::fmt::Display>(&mut self, name: &str, got: A, want: A) {
        let passed = got == want;
        self.checks.push(Check {
            name: name.to_string(),
            passed,
            detail: if passed {
                got.to_string()
            } else {
                format!("got {got}, want {want}")
            },
            delta: None,
        });
    }

    /// A claim the RFC makes that the harness has shown to be wrong. Recorded as
    /// a *pass* of the corrected expectation, with the erratum named.
    fn erratum<A: PartialEq + std::fmt::Display>(
        &mut self,
        name: &str,
        got: A,
        rfc_says: A,
        actual: A,
        delta: &'static str,
    ) {
        let passed = got == actual && got != rfc_says;
        self.checks.push(Check {
            name: name.to_string(),
            passed,
            detail: format!("computed {got}; RFC says {rfc_says}; corrected value {actual}"),
            delta: Some(delta),
        });
    }

    fn assert_true(&mut self, name: &str, passed: bool, detail: impl Into<String>) {
        self.checks.push(Check {
            name: name.to_string(),
            passed,
            detail: detail.into(),
            delta: None,
        });
    }

    fn failures(&self) -> Vec<&Check> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }
}

fn main() {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "report".into());

    if mode == "lint" {
        std::process::exit(run_lints());
    }
    if mode == "gen-docs" || mode == "check-docs" {
        std::process::exit(run_docs(&mode));
    }
    if mode == "check-links" {
        std::process::exit(links::run(&workspace_root()));
    }
    if mode == "verify-vectors" {
        std::process::exit(run_verify_vectors());
    }
    if mode == "publish" {
        let execute = std::env::args().any(|a| a == "--execute");
        std::process::exit(publish::run(execute));
    }

    println!("UC-P0 constants harness — RFC UCAL-1, profile UC-1\n");

    // --- const self-check on the default backend (§3.3) ---
    let const_bad = route_bnum::const_selfcheck();

    let a = route_bnum::derive();
    let b = route_bigint::derive();

    let mut r = Report::new();

    // ---------------------------------------------------------- 1. routes agree
    let route_diff = derivation::diff(&a, &b);
    r.assert_true(
        "routes A and B agree on every field",
        route_diff.is_empty(),
        if route_diff.is_empty() {
            format!(
                "{} fixtures, {} cf tables, {} tiers compared",
                a.fixtures.len(),
                a.cf_tables.len(),
                a.tiers.len()
            )
        } else {
            let mut s = String::new();
            for (f, x, y) in &route_diff {
                let _ = writeln!(s, "\n      {f}:\n        A = {x}\n        B = {y}");
            }
            s
        },
    );
    r.assert_true(
        "§3.3 profile constants are const-constructible and correct",
        const_bad.is_empty(),
        if const_bad.is_empty() {
            "BEAT, SECOND, ORIGIN_OFFSET, BIG_BANG_CLAIM_HALFWIDTH".into()
        } else {
            const_bad.join(", ")
        },
    );

    // ------------------------------------------------ 2. Appendix A primitives
    r.eq(
        "BEAT: 5^60 (computed) == literal",
        &a.beat_computed,
        &a.beat_parsed,
    );
    r.eq(
        "SECOND: mantissa x 10^30 (D-3) == literal",
        &a.second_computed,
        &a.second_parsed,
    );
    r.eq(
        "ORIGIN_OFFSET: provenance chain == literal",
        &a.origin_offset_from_chain,
        &a.origin_offset_parsed,
    );
    r.eq(
        "ORIGIN_OFFSET: beats x BEAT == literal",
        &a.origin_offset_from_beats,
        &a.origin_offset_parsed,
    );
    r.eq(
        "BIG_BANG_CLAIM: 631152 x mantissa x 10^39 == literal",
        &a.bbc_computed,
        &a.bbc_parsed,
    );

    // -------------------------------------- provenance chain re-execution (§2.2)
    r.eq(
        "provenance: AGE_s",
        &a.age_s,
        &d::provenance::AGE_S.to_string(),
    );
    r.eq(
        "provenance: AGE_ticks",
        &a.age_ticks,
        &d::provenance::AGE_TICKS.to_string(),
    );
    r.eq(
        "provenance: beats = round_half_even(AGE_ticks / BEAT)",
        &a.beats,
        &d::provenance::BEATS.to_string(),
    );
    r.eq(
        "provenance: residual (ticks)",
        &a.residual.render(),
        &d::provenance::RESIDUAL_TICKS.to_string(),
    );
    r.eq(
        "provenance: residual rendered in seconds",
        &a.residual_seconds_rendered,
        &d::provenance::RESIDUAL_SECONDS_RENDERED.to_string(),
    );

    // ------------------------------------------- structural claims, incl. D-A2
    r.eq("ORIGIN_OFFSET bit length", a.oo_bits, d::appendix_a::OO_BITS);
    r.eq(
        "ORIGIN_OFFSET base-5 digit count",
        a.oo_base5_digits,
        d::appendix_a::OO_BASE5_DIGITS,
    );
    r.erratum(
        "ORIGIN_OFFSET trailing base-5 zeros",
        a.oo_trailing_base5_zeros,
        d::appendix_a::OO_TRAILING_BASE5_ZEROS_CLAIMED,
        d::appendix_a::OO_TRAILING_BASE5_ZEROS_ACTUAL,
        "D-A2",
    );

    // ------------------------------------------------- BIG_BANG_CLAIM relations
    r.eq(
        "BIG_BANG_CLAIM is exactly 0.020 Gyr",
        &a.bbc_over_point02_gyr,
        &"1/1".to_string(),
    );
    r.eq(
        "BIG_BANG_CLAIM in drifts (5^80)",
        &a.bbc_in_drifts,
        &"141.53".to_string(),
    );

    // --------------------------------------------- 3. §2.4 alignment invariants
    r.assert_true(
        "§2.4 v5(SECOND) >= 30",
        a.v5_second >= d::alignment::WHOLE_SECOND_MIN_V5,
        format!("v5 = {}", a.v5_second),
    );
    r.assert_true(
        "§2.4 v5(NANOSECOND) >= 21",
        a.v5_nanosecond >= d::alignment::WHOLE_NANOSECOND_MIN_V5,
        format!("v5 = {}", a.v5_nanosecond),
    );
    r.assert_true(
        "§2.4 v5(SI_EPOCH) >= 60 (zero in all tiers below T0)",
        a.v5_origin_offset >= d::alignment::SI_EPOCH_MIN_V5,
        format!("v5 = {}", a.v5_origin_offset),
    );
    r.assert_true(
        "§2.4 whole SI seconds keep 30 trailing base-5 zeros (n = 1..400)",
        a.min_v5_whole_seconds >= d::alignment::WHOLE_SECOND_MIN_V5,
        format!("min v5 = {}", a.min_v5_whole_seconds),
    );
    r.assert_true(
        "§2.4 whole nanoseconds keep 21 trailing base-5 zeros (n = 1..400)",
        a.min_v5_whole_nanoseconds >= d::alignment::WHOLE_NANOSECOND_MIN_V5,
        format!("min v5 = {}", a.min_v5_whole_nanoseconds),
    );

    // ------------------------------------------------------------- tier grid
    r.eq("tier table entry count", a.tiers.len(), d::tiers::COUNT);
    r.assert_true(
        "tier T32 = 5^220 fits the 512-bit domain",
        a.tiers.last().map(|t| t.len()) == Some(154),
        format!("5^220 has {} decimal digits", a.tiers.last().unwrap().len()),
    );

    // -------------------------------------------------- Appendix C fixtures
    for decl in d::fixtures::ALL {
        let got = a
            .fixtures
            .iter()
            .find(|f| f.name == decl.name)
            .expect("fixture missing from derivation");

        r.eq(
            &format!("fixture ticks: {}", decl.name),
            &got.ticks,
            &decl.ticks.to_string(),
        );

        // Rule B: canonical binary is 64 bytes on every backend.
        r.assert_true(
            &format!("Rule B 64-byte encoding: {}", decl.name),
            got.bytes_hex.len() == 128,
            format!("{} hex chars", got.bytes_hex.len()),
        );

        // Rule I: UCID defined below 2^256.
        r.eq(
            &format!("fixture UCID: {}", decl.name),
            &got.ucid.clone().unwrap_or_else(|| "<out of range>".into()),
            &decl.ucid.to_string(),
        );
        r.eq(
            &format!("fixture human (T5..T0): {}", decl.name),
            &got
                .human_exact
                .trim_start_matches("UC1 ")
                .split(':')
                .next()
                .unwrap()
                .to_string(),
            &decl.human_beat.to_string(),
        );

        // D-A4: Appendix C prints five sub-beat groups (T-1..T-5). That is
        // tick-exact only when the instant's base-5 valuation reaches 35. For an
        // instant at a whole SI second the guaranteed valuation is 30 (§2.4), so
        // T-6 can be — and generally is — non-zero, making the printed form a
        // T-5 *window* rather than the tick it is labelled as.
        if let Some(sub_rfc) = decl.human_sub_rfc {
            let got_trunc = got.human_trunc_t5.split(':').nth(1).unwrap_or("").to_string();
            r.eq(
                &format!("D-A4 T-5 truncation reproduces RFC's quote: {}", decl.name),
                &got_trunc,
                &sub_rfc.to_string(),
            );
            let lossy = got.lowest_nonzero_tier.map(|k| k <= -6).unwrap_or(false);
            r.assert_true(
                &format!(
                    "D-A4 lossiness agrees with valuation: {} ({})",
                    decl.name,
                    if lossy { "lossy" } else { "exact at T-5" }
                ),
                lossy == (got.v5 < 35),
                format!(
                    "v5 = {}, lowest non-zero tier T{}, RFC's five groups are {}",
                    got.v5,
                    got.lowest_nonzero_tier.unwrap_or(0),
                    if lossy {
                        "a window, not a tick"
                    } else {
                        "tick-exact"
                    }
                ),
            );
        }
    }

    // Appendix C's five-group form is lossy exactly for instants whose base-5
    // valuation is below 35. §2.4 guarantees only 30 for a whole SI second, so
    // the lossy set is non-empty by construction and every member sits in
    // [30, 34] — the five digits Appendix C silently drops.
    let (lossy, exact): (Vec<_>, Vec<_>) = a
        .fixtures
        .iter()
        .filter(|f| f.lowest_nonzero_tier.is_some())
        .partition(|f| f.lowest_nonzero_tier.unwrap() <= -6);
    r.assert_true(
        "D-A4: every lossy fixture has v5 in [30, 34], every exact one v5 >= 35",
        !lossy.is_empty()
            && lossy.iter().all(|f| (30..35).contains(&f.v5))
            && exact.iter().all(|f| f.v5 >= 35),
        format!(
            "{} lossy (v5 {:?}), {} exact at T-5 (v5 {:?})",
            lossy.len(),
            lossy.iter().map(|f| f.v5).collect::<Vec<_>>(),
            exact.len(),
            exact.iter().map(|f| f.v5).collect::<Vec<_>>()
        ),
    );

    // digit5 range: the RFC's 22-group line is T9..T-12 (delta D-A4).
    let f2026 = a
        .fixtures
        .iter()
        .find(|f| f.name.starts_with("2026-07-29"))
        .unwrap();
    r.eq(
        "Appendix C digit5 line is the T9..T-12 range",
        &f2026.digit5_t9_tm12.trim_start_matches("UC1/5 ").to_string(),
        &d::fixtures::DIGIT5_2026.to_string(),
    );

    // ---------------------------------------------------- Appendix I vectors
    let i_vectors = [
        (&d::appendix_i::I1_EARTH_INTERCALATION, 0usize),
        (&d::appendix_i::I2_EARTH_GROUPING, 1),
        (&d::appendix_i::I3_MARS_INTERCALATION, 2),
        (&d::appendix_i::I5_TITAN_INTERCALATION, 3),
    ];
    for (decl, idx) in i_vectors {
        let got = &a.cf_tables[idx];
        // I.2 prints the whole ratio's expansion; the others print the fraction's.
        let is_whole_form = decl.cf_frac.first() != Some(&0);
        let (got_cf, got_cv) = if is_whole_form {
            (&got.cf_whole, &got.convergents_whole)
        } else {
            (&got.cf_frac, &got.convergents_frac)
        };
        r.assert_true(
            &format!("{}: continued fraction", decl.label),
            got_cf.len() >= decl.cf_frac.len() && got_cf[..decl.cf_frac.len()] == *decl.cf_frac,
            format!("{:?}", &got_cf[..decl.cf_frac.len().min(got_cf.len())]),
        );
        let want: Vec<(String, String)> = decl
            .convergents
            .iter()
            .map(|(n, dd)| (n.to_string(), dd.to_string()))
            .collect();
        r.assert_true(
            &format!("{}: convergents", decl.label),
            got_cv.len() >= want.len() && got_cv[..want.len()] == want[..],
            format!(
                "{:?}",
                got_cv[..want.len().min(got_cv.len())]
                    .iter()
                    .map(|(n, dd)| format!("{n}/{dd}"))
                    .collect::<Vec<_>>()
            ),
        );
    }

    // §21.3-6: 97/400 must not appear at any depth.
    let earth_int = &a.cf_tables[0];
    let greg = (
        d::appendix_i::GREGORIAN_RULE.0.to_string(),
        d::appendix_i::GREGORIAN_RULE.1.to_string(),
    );
    r.assert_true(
        "§21.3-6: 97/400 absent from Earth's intercalation convergents at every depth",
        !earth_int.convergents_frac.contains(&greg),
        format!(
            "{} convergents walked: {}",
            earth_int.convergents_frac.len(),
            earth_int
                .convergents_frac
                .iter()
                .map(|(n, dd)| format!("{n}/{dd}"))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    );
    r.assert_true(
        "§21.3-6: 1/4 (Julian) is convergent 1",
        earth_int.convergents_frac.first() == Some(&("1".into(), "4".into())),
        format!("{:?}", earth_int.convergents_frac.first()),
    );

    // §21.3-7: Metonic 235/19 present for Earth.
    let earth_grp = &a.cf_tables[1];
    let met = (
        d::appendix_i::METONIC.0.to_string(),
        d::appendix_i::METONIC.1.to_string(),
    );
    r.assert_true(
        "§21.3-7: Metonic 235/19 present in Earth's grouping convergents",
        earth_grp.convergents_whole.contains(&met),
        format!(
            "{}",
            earth_grp
                .convergents_whole
                .iter()
                .map(|(n, dd)| format!("{n}/{dd}"))
                .collect::<Vec<_>>()
                .join(" ")
        ),
    );

    // ------------------------------------------- Appendix I.4 / delta D-A5
    r.eq(
        "I.4 Phobos synodic period (sols)",
        &a.phobos_synodic_sols[..6].to_string(),
        &d::appendix_i::PHOBOS_SYNODIC_SOLS.to_string(),
    );
    r.eq(
        "I.4 Deimos synodic period (sols)",
        &a.deimos_synodic_sols[..6].to_string(),
        &d::appendix_i::DEIMOS_SYNODIC_SOLS.to_string(),
    );
    let de = dec_cmp::Decimal::parse(&a.deimos_synodic_sols);
    let inside = de.ge_int(d::appendix_i::CYCLE_BOUNDS_SOLS.0)
        && de.le_int(d::appendix_i::CYCLE_BOUNDS_SOLS.1);
    // The figure above is computed by §9.6's formula as written, which D-A12
    // later showed measures the wrong quantity. It is reproduced here because
    // Appendix I.4 published it, and a reproduction check has to reproduce what
    // was published rather than what it should have said. Under the *corrected*
    // formula Deimos is 1.2315 sols and falls outside the bound comfortably.
    r.assert_true(
        "I.4 as published: Deimos falls inside D-11's [5,100] bound (see D-A12)",
        inside,
        format!(
            "{} sols is within [{}, {}] under §9.6 as written. D-A12 corrects that \
             formula, under which Deimos is 1.2315 sols and the bound never admits \
             it; D-A5's amendment stands on its own merits, not on this admission",
            a.deimos_synodic_sols,
            d::appendix_i::CYCLE_BOUNDS_SOLS.0,
            d::appendix_i::CYCLE_BOUNDS_SOLS.1
        ),
        );

    // ------------------------------------------------------------- reporting
    let mut passed = 0usize;
    let mut failed = 0usize;
    for c in &r.checks {
        if c.passed {
            passed += 1;
        } else {
            failed += 1;
        }
    }

    if mode == "report" || mode == "check" {
        for c in &r.checks {
            let tag = if c.passed { "ok  " } else { "FAIL" };
            let delta = c.delta.map(|x| format!("  [{x}]")).unwrap_or_default();
            println!("  {tag}  {}{delta}", c.name);
            if !c.passed || c.delta.is_some() {
                println!("        {}", c.detail);
            }
        }
        println!();
    }

    if mode == "emit" || mode == "report" {
        let path = "fixtures/vectors.json";
        let json = emit_vectors(&a, &b);
        std::fs::create_dir_all("fixtures").expect("mkdir fixtures");
        std::fs::write(path, &json).expect("write vectors");
        let mut h = Sha256::new();
        h.update(json.as_bytes());
        let digest = h.finalize();
        let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
        std::fs::write(
            "fixtures/SHA256SUMS",
            format!("{hex}  vectors.json\n"),
        )
        .expect("write manifest");
        println!("  wrote {path} ({} bytes)", json.len());
        println!("  wrote fixtures/SHA256SUMS");
        println!("  sha256 {hex}");
        println!(
            "  NOTE: §20 calls for a *signed* vector file. SHA256SUMS is the \
             artefact to sign; signing needs a key and is a release step, not a \
             harness step. Procedure: spec/CONFORMANCE.md. Verify a checkout \
             with `cargo run -p xtask -- verify-vectors`."
        );
        println!();
    }

    println!("  {passed} passed, {failed} failed");
    if failed > 0 {
        println!("\n  FAILURES:");
        for c in r.failures() {
            println!("    - {}: {}", c.name, c.detail);
        }
        // §19.5 exit code 6: data error.
        std::process::exit(6);
    }
    println!("\n  UC-P0 exit criterion met.");
}

/// Comparison of a rendered decimal against an integer bound without a float
/// (Rule E applies to the harness too — it is the oracle).
mod dec_cmp {
    pub struct Decimal {
        pub int: u64,
        pub frac_nonzero: bool,
    }
    impl Decimal {
        pub fn parse(s: &str) -> Self {
            match s.split_once('.') {
                None => Decimal {
                    int: s.parse().unwrap_or(0),
                    frac_nonzero: false,
                },
                Some((i, f)) => Decimal {
                    int: i.parse().unwrap_or(0),
                    frac_nonzero: f.chars().any(|c| c != '0'),
                },
            }
        }
        /// `self >= n`
        pub fn ge_int(&self, n: u32) -> bool {
            self.int >= n as u64
        }
        /// `self <= n`
        pub fn le_int(&self, n: u32) -> bool {
            self.int < n as u64 || (self.int == n as u64 && !self.frac_nonzero)
        }
    }
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn emit_vectors(a: &Derivation, b: &Derivation) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    let _ = writeln!(s, "  \"format\": \"ucal-vectors/1\",");
    let _ = writeln!(s, "  \"rfc\": \"UCAL-1 final draft 2026-07-29\",");
    let _ = writeln!(s, "  \"profile\": \"UC-1\",");
    let _ = writeln!(
        s,
        "  \"deltas\": \"see spec/SPEC-DELTAS.md (D-A1 .. D-A6)\","
    );
    let _ = writeln!(s, "  \"routes\": [\"{}\", \"{}\"],", esc(&a.route), esc(&b.route));
    let _ = writeln!(s, "  \"constants\": {{");
    let _ = writeln!(s, "    \"BEAT\": \"{}\",", a.beat_parsed);
    let _ = writeln!(s, "    \"SECOND\": \"{}\",", a.second_parsed);
    let _ = writeln!(s, "    \"ORIGIN_OFFSET\": \"{}\",", a.origin_offset_parsed);
    let _ = writeln!(
        s,
        "    \"BIG_BANG_CLAIM_HALFWIDTH\": \"{}\",",
        a.bbc_parsed
    );
    let _ = writeln!(s, "    \"DOMAIN_MAX\": \"{}\"", a.domain_max);
    let _ = writeln!(s, "  }},");
    let _ = writeln!(s, "  \"provenance\": {{");
    let _ = writeln!(s, "    \"AGE_s\": \"{}\",", a.age_s);
    let _ = writeln!(s, "    \"AGE_ticks\": \"{}\",", a.age_ticks);
    let _ = writeln!(s, "    \"beats\": \"{}\",", a.beats);
    let _ = writeln!(s, "    \"residual_ticks\": \"{}\",", a.residual.render());
    let _ = writeln!(
        s,
        "    \"residual_seconds\": \"{}\"",
        a.residual_seconds_rendered
    );
    let _ = writeln!(s, "  }},");
    let _ = writeln!(s, "  \"alignment\": {{");
    let _ = writeln!(s, "    \"v5_SECOND\": {},", a.v5_second);
    let _ = writeln!(s, "    \"v5_NANOSECOND\": {},", a.v5_nanosecond);
    let _ = writeln!(s, "    \"v5_ORIGIN_OFFSET\": {},", a.v5_origin_offset);
    let _ = writeln!(
        s,
        "    \"min_v5_whole_seconds\": {},",
        a.min_v5_whole_seconds
    );
    let _ = writeln!(
        s,
        "    \"min_v5_whole_nanoseconds\": {}",
        a.min_v5_whole_nanoseconds
    );
    let _ = writeln!(s, "  }},");

    let _ = writeln!(s, "  \"tiers\": [");
    for (i, t) in a.tiers.iter().enumerate() {
        let k = d::tiers::K_MIN as i32 + i as i32;
        let name = d::tiers::NAMED
            .iter()
            .find(|(kk, _, _)| *kk as i32 == k)
            .map(|(_, _, n)| *n)
            .unwrap_or("");
        let comma = if i + 1 == a.tiers.len() { "" } else { "," };
        let _ = writeln!(
            s,
            "    {{ \"k\": {k}, \"exp\": {}, \"name\": \"{name}\", \"ticks\": \"{t}\" }}{comma}",
            60 + 5 * k
        );
    }
    let _ = writeln!(s, "  ],");

    let _ = writeln!(s, "  \"fixtures\": [");
    for (i, f) in a.fixtures.iter().enumerate() {
        let comma = if i + 1 == a.fixtures.len() { "" } else { "," };
        let _ = writeln!(s, "    {{");
        let _ = writeln!(s, "      \"name\": \"{}\",", esc(&f.name));
        let _ = writeln!(s, "      \"ticks\": \"{}\",", f.ticks);
        let _ = writeln!(s, "      \"human_exact\": \"{}\",", esc(&f.human_exact));
        let _ = writeln!(
            s,
            "      \"lowest_nonzero_tier\": {},",
            f.lowest_nonzero_tier
                .map(|k| k.to_string())
                .unwrap_or_else(|| "null".into())
        );
        let _ = writeln!(s, "      \"v5\": {},", f.v5);
        let _ = writeln!(
            s,
            "      \"human_truncated_T-5\": \"{}\",",
            esc(&f.human_trunc_t5)
        );
        let _ = writeln!(
            s,
            "      \"truncation_window\": [\"{}\", \"{}\"],",
            f.trunc_window_lo, f.trunc_window_hi
        );
        let _ = writeln!(
            s,
            "      \"digit5_T9_to_T-12\": \"{}\",",
            esc(&f.digit5_t9_tm12)
        );
        let _ = writeln!(
            s,
            "      \"digit5_T5_to_T-12\": \"{}\",",
            esc(&f.digit5_t5_tm12)
        );
        let _ = writeln!(
            s,
            "      \"ucid\": {},",
            f.ucid
                .as_ref()
                .map(|u| format!("\"{u}\""))
                .unwrap_or_else(|| "null".into())
        );
        let _ = writeln!(s, "      \"bytes_be_hex\": \"{}\"", f.bytes_hex);
        let _ = writeln!(s, "    }}{comma}");
    }
    let _ = writeln!(s, "  ],");

    let _ = writeln!(s, "  \"continued_fractions\": [");
    for (i, t) in a.cf_tables.iter().enumerate() {
        let comma = if i + 1 == a.cf_tables.len() { "" } else { "," };
        let _ = writeln!(s, "    {{");
        let _ = writeln!(s, "      \"label\": \"{}\",", esc(&t.label));
        let _ = writeln!(s, "      \"ratio\": \"{}\",", t.ratio);
        let _ = writeln!(s, "      \"whole\": \"{}\",", t.whole);
        let _ = writeln!(s, "      \"cf_fractional_part\": {:?},", t.cf_frac);
        let _ = writeln!(s, "      \"cf_whole_ratio\": {:?},", t.cf_whole);
        let _ = writeln!(
            s,
            "      \"convergents_fractional\": [{}],",
            t.convergents_frac
                .iter()
                .map(|(n, dd)| format!("\"{n}/{dd}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(
            s,
            "      \"convergents_whole\": [{}],",
            t.convergents_whole
                .iter()
                .map(|(n, dd)| format!("\"{n}/{dd}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(
            s,
            "      \"drift_per_1000_periods\": [{}],",
            t.drift_per_1000
                .iter()
                .map(|x| format!("\"{x}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(
            s,
            "      \"one_unit_slips_in\": [{}]",
            t.slips_in
                .iter()
                .map(|x| format!("\"{x}\""))
                .collect::<Vec<_>>()
                .join(", ")
        );
        let _ = writeln!(s, "    }}{comma}");
    }
    let _ = writeln!(s, "  ],");

    let _ = writeln!(s, "  \"mars_satellites\": {{");
    let _ = writeln!(
        s,
        "    \"note\": \"delta D-A5: Deimos is inside D-11's [5,100] bound, so the RFC's stated algorithm admits it. Grouping is now declared per body.\",",
    );
    let _ = writeln!(
        s,
        "    \"phobos_synodic_sols\": \"{}\",",
        a.phobos_synodic_sols
    );
    let _ = writeln!(
        s,
        "    \"deimos_synodic_sols\": \"{}\",",
        a.deimos_synodic_sols
    );
    let _ = writeln!(
        s,
        "    \"mars_year_over_deimos_synodic\": \"{}\",",
        a.mars_deimos_cycles_per_year
    );
    let _ = writeln!(
        s,
        "    \"hypothetical_convergents\": [{}]",
        a.mars_deimos_convergents
            .iter()
            .map(|(n, dd)| format!("\"{n}/{dd}\""))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let _ = writeln!(s, "  }}");
    s.push_str("}\n");
    s
}

/// `cargo run -p xtask -- lint` — the §21.3 invariant lints.
fn run_lints() -> i32 {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives under the workspace root")
        .to_path_buf();
    println!("UC lint — workspace {}\n", root.display());
    let violations = lint::run(&root);
    let allowed = lint::suppressions(&root);
    if violations.is_empty() {
        println!("  ok    Rule E: no float token in a shipped crate");
        println!("  ok    Rules A.2/Y: ucal-core names no foreign unit");
        println!("  ok    Rule O: no wrapping or saturating arithmetic");
        println!("  ok    Rule Q.1: no overclaiming prose about tick 0");
        println!("  ok    §12: dependency direction");
        print_suppressions(&root, &allowed);
        println!("\n  0 violations");
        return 0;
    }
    print_suppressions(&root, &allowed);
    let mut by_lint: std::collections::BTreeMap<&str, Vec<&lint::Violation>> = Default::default();
    for v in &violations {
        by_lint.entry(v.lint).or_default().push(v);
    }
    for (name, vs) in &by_lint {
        println!("  FAIL  {name}  ({} violations)", vs.len());
        println!("        {}", vs[0].rule);
        for v in vs.iter().take(20) {
            let rel = v.file.strip_prefix(&root).unwrap_or(&v.file);
            println!("          {}:{}  {}", rel.display(), v.line, v.text);
        }
        if vs.len() > 20 {
            println!("          ... and {} more", vs.len() - 20);
        }
    }
    println!("\n  {} violations", violations.len());
    6
}

/// List the exemptions the lints honoured.
///
/// A green run with an unlisted suppression is the same failure as a red run
/// nobody read: the rule stopped being enforced and nothing said so.
fn print_suppressions(root: &std::path::Path, allowed: &[lint::Suppression]) {
    if allowed.is_empty() {
        return;
    }
    println!("\n  {} exemption(s) honoured:", allowed.len());
    for s in allowed {
        let rel = s.file.strip_prefix(root).unwrap_or(&s.file);
        println!(
            "        {}  {}:{}{}",
            s.lint,
            rel.display(),
            s.line,
            if s.region { "  (region)" } else { "" }
        );
    }
}

/// `verify-vectors` — re-derive the conformance vectors and check them against
/// the committed digest.
///
/// This answers "does this checkout produce the vectors it claims to", which is
/// the half of §20's requirement that needs no key. The other half — "did the
/// maintainer vouch for this digest" — needs a signature, and
/// `spec/CONFORMANCE.md` documents how to make and check one.
///
/// Separating the two matters. A digest proves the file was not corrupted; only
/// a signature proves who stood behind it, and reporting the first as if it were
/// the second is the kind of overclaim Rule Q exists to prevent.
fn run_verify_vectors() -> i32 {
    let root = workspace_root();
    let a = route_bnum::derive();
    let b = route_bigint::derive();
    let json = emit_vectors(&a, &b);

    let mut h = Sha256::new();
    h.update(json.as_bytes());
    let hex: String = h.finalize().iter().map(|b| format!("{b:02x}")).collect();

    let manifest = root.join("fixtures/SHA256SUMS");
    let committed = match std::fs::read_to_string(&manifest) {
        Ok(t) => t.split_whitespace().next().unwrap_or("").to_string(),
        Err(e) => {
            eprintln!("  FAIL  cannot read fixtures/SHA256SUMS: {e}");
            return 6;
        }
    };

    if hex != committed {
        eprintln!("  FAIL  vectors do not match the committed digest");
        eprintln!("          re-derived {hex}");
        eprintln!("          committed  {committed}");
        eprintln!("        Either this checkout changed a constant, or the manifest is stale.");
        return 6;
    }
    println!("  ok    vectors re-derive to the committed digest");
    println!("        sha256 {hex}");

    let sig = root.join("fixtures/SHA256SUMS.minisig");
    if sig.exists() {
        println!("  ok    a detached signature is present: fixtures/SHA256SUMS.minisig");
        println!("        verify it: minisign -Vm fixtures/SHA256SUMS -P <public key>");
    } else {
        println!("  --    UNSIGNED. The digest is self-consistent and nobody has vouched");
        println!("        for it. See spec/CONFORMANCE.md to sign a release.");
    }
    0
}

/// Workspace root, from this crate's manifest.
fn workspace_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives under the workspace root")
        .to_path_buf()
}

/// `gen-docs` writes the generated tables; `check-docs` fails if they are stale.
fn run_docs(mode: &str) -> i32 {
    let root = workspace_root();
    if mode == "gen-docs" {
        match gendocs::write(&root) {
            Ok(p) => {
                println!("wrote {}", p.display());
                0
            }
            Err(e) => {
                eprintln!("failed to write generated docs: {e}");
                6
            }
        }
    } else {
        let mut code = match gendocs::check(&root) {
            Ok(()) => {
                println!("  ok    §13.5: generated docs are current");
                0
            }
            Err(e) => {
                eprintln!("  FAIL  §13.5: {e}");
                6
            }
        };
        // Citation integrity. A dangling `§` or `Rule` is a lost explanation,
        // and the only thing that keeps a thousand of them honest is a check.
        match citations::check(&root) {
            Ok(n) => println!("  ok    citations resolve against spec/ ({n} distinct)"),
            Err(bad) => {
                eprintln!("  FAIL  {} citation(s) resolve to nothing:", bad.len());
                for d in bad.iter().take(20) {
                    eprintln!("          {} `{}`  ({} site(s))", d.kind, d.citation, d.sites);
                }
                if bad.len() > 20 {
                    eprintln!("          ... and {} more", bad.len() - 20);
                }
                code = 6;
            }
        }
        // The CLI manual's *surface*. Its prose cannot be generated — what
        // `remainder_ticks` means is not derivable from a type — but a command
        // that exists and is undocumented, or a section for a command that no
        // longer exists, are defects a reader hits and nothing else catches.
        match citations::check_contact_constants(&root) {
            Ok(n) => println!("  ok    contact materials quote vectors.json ({n} constants)"),
            Err(bad) => {
                eprintln!("  FAIL  the contact materials have drifted from the vectors:");
                for b in &bad {
                    eprintln!("          {b}");
                }
                code = 6;
            }
        }
        // The key is published in several places so that one beyond the
        // author's reach can contradict a repository that has been rewritten.
        // Copies that disagree for an innocent reason destroy exactly that.
        match citations::check_signing_key(&root) {
            Ok(n) => println!("  ok    the signing key is published identically ({n} places)"),
            Err(bad) => {
                eprintln!("  FAIL  the published copies of the signing key disagree:");
                for b in &bad {
                    eprintln!("          {b}");
                }
                code = 6;
            }
        }
        match citations::check_ci_covers_the_procedure(&root) {
            Ok(n) => println!("  ok    CI runs the documented verification block ({n} commands)"),
            Err(bad) => {
                eprintln!("  FAIL  CI and the release procedure have drifted:");
                for b in &bad {
                    eprintln!("          {b}");
                }
                code = 6;
            }
        }
        match citations::check_cli_docs(&root) {
            Ok(n) => println!("  ok    Documentation/CLI.md covers the CLI surface ({n} items)"),
            Err(bad) => {
                eprintln!("  FAIL  Documentation/CLI.md is out of step:");
                for b in &bad {
                    eprintln!("          {b}");
                }
                code = 6;
            }
        }
        code
    }
}
