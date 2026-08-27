//! `ucal verify` — the self-check, checked.
//!
//! The command exists so that someone who typed `cargo install ucal` can find
//! out whether their binary reproduces the published constants without cloning
//! a repository. Two things have to be true for that to be worth anything:
//!
//! 1. the numbers it derives are the ones in `fixtures/vectors.json`, whose
//!    digest is signed — otherwise it is confidently reporting the wrong
//!    values;
//! 2. it *derives* them rather than echoing the profile back — otherwise it
//!    agrees with itself by construction and would keep agreeing through any
//!    corruption of the constants it is supposed to be checking.
//!
//! The second is the one that is easy to get wrong and impossible to see by
//! reading the output.

use ucal::emit::{Doc, Value};

fn section<'a>(doc: &'a Doc, key: &str) -> &'a [(String, Value)] {
    doc.fields()
        .iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.as_rows())
        .unwrap_or_else(|| panic!("`{key}` is not a section in the document"))
}

fn cell(rows: &[(String, Value)], key: &str) -> String {
    rows.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.rendered_text().trim().to_string())
        .unwrap_or_else(|| panic!("`{key}` is missing"))
}

/// The long decimal that `vectors.json` records for a named constant.
///
/// Not a JSON parser: the file is generated, the values are long decimal runs,
/// and finding them by name needs no dependency — the same approach
/// `xtask`'s `contact-constants` check takes, for the same reason.
fn from_vectors(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("fixtures/vectors.json");
    let json = std::fs::read_to_string(path).expect("fixtures/vectors.json");
    let key = format!("\"{name}\"");
    let at = json.find(&key).unwrap_or_else(|| panic!("{name} is not in vectors.json"));
    json[at + key.len()..]
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit())
        .collect()
}

/// What the binary derives is what the signed vectors say.
///
/// This is the claim a stranger acts on. If it were ever false, `ucal verify`
/// would be telling people to expect numbers that the conformance vectors do
/// not contain — worse than not shipping the command, because it would waste
/// the one resource C1 depends on.
#[test]
fn the_derived_constants_are_the_published_ones() {
    let doc = ucal::cmd_verify().unwrap();
    let constants = section(&doc, "constants");

    for name in ["BEAT", "SECOND", "ORIGIN_OFFSET"] {
        let rows = constants
            .iter()
            .find(|(k, _)| k == name)
            .and_then(|(_, v)| v.as_rows())
            .unwrap_or_else(|| panic!("`{name}` is not reported"));

        let derived = cell(rows, "derived");
        assert_eq!(
            derived,
            from_vectors(name),
            "`ucal verify` derives a {name} that fixtures/vectors.json does not agree with"
        );
        assert_eq!(
            cell(rows, "value"),
            derived,
            "the profile and the derivation disagree about {name}"
        );
        assert_eq!(cell(rows, "agrees"), "true");
    }
}

/// A healthy build reports agreement, and every invariant holds.
#[test]
fn a_good_build_agrees_with_itself() {
    let doc = ucal::cmd_verify().unwrap();
    assert_eq!(
        doc.fields()
            .iter()
            .find(|(k, _)| k == "agrees")
            .map(|(_, v)| v.rendered_text().trim().to_string())
            .unwrap(),
        "true"
    );
    for (name, v) in section(&doc, "invariants") {
        assert!(
            matches!(v, Value::Bool(true)),
            "invariant `{name}` does not hold"
        );
    }
}

/// The derivation is independent of the profile it checks.
///
/// The failure this guards against is a `verify` that reads `UC1::beat()` twice
/// and compares it with itself: it would print `agrees true` on a build whose
/// constants had been corrupted, which is precisely the build it exists to
/// catch. Recomputing the derivation here — by the same definitions, in this
/// file, with no reference to the profile — and requiring it to match what the
/// command reports as `derived` keeps the two paths apart.
#[test]
fn the_derivation_does_not_come_from_the_profile() {
    use ucal_core::backend::TickInt;
    use ucal_core::Ticks;

    let five = <Ticks as TickInt>::from_u64(5);
    let mut beat = <Ticks as TickInt>::one();
    for _ in 0..60 {
        beat = beat.try_mul(&five).unwrap();
    }

    let doc = ucal::cmd_verify().unwrap();
    let rows = section(&doc, "constants")
        .iter()
        .find(|(k, _)| k == "BEAT")
        .and_then(|(_, v)| v.as_rows())
        .unwrap();

    assert_eq!(
        cell(rows, "derived"),
        beat.to_dec_string(),
        "`derived` is not 5^60 computed independently — it may be echoing the profile"
    );
}

/// The output says what it does not establish.
///
/// A green self-check is exactly the kind of thing a reader promotes into
/// "verified", and the command would then be arguing against C1 rather than
/// for it. The disclaimer is load-bearing, so it is tested like anything else
/// that is.
#[test]
fn the_output_refuses_to_be_mistaken_for_verification() {
    let doc = ucal::cmd_verify().unwrap();
    let text = doc.to_text();
    assert!(
        text.contains("self-check"),
        "the output must not let a green run read as independent verification"
    );
    assert!(
        text.contains("CONTACT.md"),
        "the output should point at the check that would establish it"
    );
}

/// G12 — `doctor` names **every** optional feature this binary can have.
///
/// It reported `u512, std, civil` for a binary built with `--features full`:
/// `body`, `events`, `cosmo` and `tui` were compiled in and unlisted, so four of
/// the commands a reader could run were invisible to the command whose whole job
/// is to enumerate what is in the build.
///
/// The failure mode is a feature added and `cmd_doctor` not revisited, which is
/// exactly what happened — so this checks the list against the manifest rather
/// than against a copy of itself.
#[test]
fn doctor_knows_every_feature_in_the_manifest() {
    let manifest = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/Cargo.toml"
    ))
    .expect("the ucal manifest");

    // The `[features]` table, minus the aggregates that are lists of others.
    let aggregates = ["default", "full"];
    let mut declared: Vec<String> = Vec::new();
    let mut in_features = false;
    for line in manifest.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_features = l == "[features]";
            continue;
        }
        if !in_features || l.is_empty() || l.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = l.split_once('=') {
            let name = name.trim();
            // `dep:` entries and optional-dependency shims are not user-facing
            // feature names a reader would type.
            if !aggregates.contains(&name) && !name.starts_with("dep:") {
                declared.push(name.to_string());
            }
        }
    }
    assert!(!declared.is_empty(), "no features parsed from the manifest");

    let known: Vec<&str> = ucal::ALL_FEATURES.iter().map(|(n, _)| *n).collect();
    for d in &declared {
        assert!(
            known.contains(&d.as_str()),
            "`{d}` is a feature of this crate and `doctor` has never heard of it. \
             Add it to ALL_FEATURES.\nknown: {known:?}"
        );
    }
}

/// And the ones that are on are the ones reported.
#[test]
fn doctor_reports_the_features_that_are_on() {
    let doc = ucal::cmd_doctor().expect("a report");
    let listed = doc
        .get("features")
        .map(ucal::emit::Value::rendered_text)
        .unwrap_or_default();
    for (name, on) in ucal::ALL_FEATURES {
        assert_eq!(
            listed.split_whitespace().any(|w| w == *name),
            *on,
            "`{name}` is {} and the report says otherwise: {listed}",
            if *on { "compiled in" } else { "absent" }
        );
    }
}
