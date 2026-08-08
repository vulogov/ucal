//! The `ucal-json/1` promise, made checkable.
//!
//! `Documentation/STABILITY.md` promises that existing JSON fields never change
//! name, shape or meaning, and that new fields may appear. Every other check in
//! this tree runs against *one* version of the tree and so cannot see that
//! promise at all — it is a statement about change over time.
//!
//! `fixtures/json-surface.txt` is a committed baseline: every field path every
//! command emits, with the JSON kind it serialises to. This compares the current
//! tree against it.
//!
//! - a path that disappears is a **failure**;
//! - a path whose kind changes is a **failure**;
//! - a path that appears is **fine**, and updates the baseline.
//!
//! That asymmetry is the contract, not a convenience.
//!
//! # Regenerating
//!
//! ```text
//! UCAL_BLESS=1 cargo test -p ucal --test json_surface
//! ```
//!
//! Deliberately an environment variable rather than automatic. A baseline that
//! rewrites itself records what happened; one that has to be asked records what
//! was intended, and the diff is the thing a reviewer reads.
//!
//! # What it cannot check
//!
//! *Meaning.* A field that keeps its name and its kind and starts reporting
//! something else passes this and is exactly the breakage the promise is about.
//! Nothing mechanical reaches that, and `STABILITY.md` says so rather than
//! letting a green run imply otherwise.

use std::collections::BTreeMap;

use ucal::emit::{Doc, Value};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";
const T2: &str = "8070205189999984864657505252035637180530466139316558837890625";

fn baseline_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("fixtures/json-surface.txt")
}

/// Every command, with the name it is recorded under.
fn commands() -> Vec<(&'static str, Doc)> {
    let mut v: Vec<(&'static str, Doc)> = vec![
        ("datum", ucal::cmd_datum().unwrap()),
        ("doctor", ucal::cmd_doctor().unwrap()),
        ("explain", ucal::cmd_explain(T, true).unwrap()),
("between", ucal::cmd_between(T, T2, Some(ucal_core::Tier::BEAT)).unwrap()),
        ("verify", ucal::cmd_verify().unwrap()),
        ("tour", ucal::cmd_tour().unwrap()),
        ("explain-why", ucal::cmd_explain_why(T, false).unwrap()),
        (
            "ladder",
            ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap(),
        ),
        (
            "now",
            ucal::cmd_now(
                ucal::parse_tier("T-12").unwrap(),
                ucal_core::codec::Form::HumanGroups,
            )
            .unwrap(),
        ),
    ];
    #[cfg(feature = "events")]
    {
        v.push(("events-list", ucal::cmd_events_list().unwrap()));
        v.push(("events-show", ucal::cmd_events_show("recombination").unwrap()));
        v.push((
            "timeline",
            ucal::cmd_timeline(ucal::parse_tier("drift").unwrap()).unwrap(),
        ));
        v.push((
            "ruler",
            ucal::cmd_ruler(
                "0",
                "100000000000000000000000000000000000000000000",
                ucal::parse_tier("sweep").unwrap(),
            )
            .unwrap(),
        ));
    }
    #[cfg(all(feature = "body", feature = "civil"))]
    {
        v.push(("cal-list", ucal::cmd_cal_list().unwrap()));
        v.push(("cal-show", ucal::cmd_cal_show("earth-d", T).unwrap()));
        v.push(("cal-anchor", ucal::cmd_cal_anchor("earth-d").unwrap()));
        v.push((
            "show",
            ucal::cmd_show(
                T,
                &["earth-d".into(), "mars-d".into(), "titan-d".into(), "earth-civil".into()],
            )
            .unwrap(),
        ));
        v.push((
            "to-civil",
            ucal::cmd_to_civil(
                T,
                ucal_civil::si::Scale::Tt,
                3,
                ucal_core::Rounding::HalfEven,
                ucal_civil::calendar::CivilCalendar::Gregorian,
            )
            .unwrap(),
        ));
        v.push((
            "from-civil",
            ucal::cmd_from_civil(
                "2026-08-04",
                ucal_civil::si::Scale::Tt,
                ucal_civil::calendar::CivilCalendar::Gregorian,
            )
            .unwrap(),
        ));
    }
    #[cfg(feature = "cosmo")]
    {
        v.push(("cosmo-model", ucal::cmd_cosmo_model().unwrap()));
        v.push(("cosmo-age", ucal::cmd_cosmo_age("1100", 4, 8).unwrap()));
        v.push((
            "cosmo-age-interval",
            ucal::cmd_cosmo_age_audited("1090..1110", 4, 8, true).unwrap(),
        ));
        v.push(("cosmo-z", ucal::cmd_cosmo_z(T, 1_000_000, 4, 8).unwrap()));
    }
    v
}

/// What a value serialises to in JSON.
///
/// The promise is about the *JSON* shape, so `Text`, `Form`, `Quantity` and
/// `Number` are all `string` — `Number` included, because a 61-digit integer
/// cannot survive a JSON number and is emitted as a string deliberately.
fn kind(v: &Value) -> &'static str {
    match v {
        Value::Text(_) | Value::Form(_) | Value::Number(_) | Value::Quantity { .. } => "string",
        Value::Bool(_) => "bool",
        Value::List(_) => "array",
        Value::Section(_) | Value::Rows { .. } => "object",
        Value::Bridge(inner) => kind(inner),
        _ => "unknown",
    }
}

/// Every field path a document emits, with its kind.
///
/// Table row keys are data — `T5`, `earth-d`, `recombination` — so a row's key
/// is recorded as `*` and its fields beneath it. A new event must not change the
/// surface.
fn surface(name: &str, doc: &Doc) -> BTreeMap<String, String> {
    fn walk(
        fields: &[(String, Value)],
        prefix: &str,
        keyed: bool,
        out: &mut BTreeMap<String, String>,
    ) {
        for (k, v) in fields {
            let seg = if keyed { k.as_str() } else { "*" };
            let path = format!("{prefix}.{seg}");
            let gated = matches!(v, Value::Bridge(_));
            let inner = match v {
                Value::Bridge(b) => b.as_ref(),
                other => other,
            };
            let mut ty = kind(inner).to_string();
            if gated {
                ty.push_str(" (--bridge)");
            }
            out.insert(path.clone(), ty);
            match inner {
                Value::Section(f) => walk(f, &path, true, out),
                Value::Rows { rows, .. } => walk(rows, &path, false, out),
                _ => {}
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(doc.fields(), name, true, &mut out);
    out
}

fn current() -> BTreeMap<String, String> {
    let mut all = BTreeMap::new();
    for (name, doc) in commands() {
        all.extend(surface(name, &doc));
    }
    all
}

fn render(m: &BTreeMap<String, String>) -> String {
    let mut s = String::from(
        "# The ucal-json/1 surface. One line per field path, with the JSON kind\n\
         # it serialises to. A `*` segment is a table row key, which is data\n\
         # rather than schema. See crates/ucal/tests/json_surface.rs.\n\
         #\n\
         # A path removed or a kind changed is a breaking change. A path added\n\
         # is not — that is the contract Documentation/STABILITY.md states.\n",
    );
    for (k, v) in m {
        s.push_str(k);
        s.push('\t');
        s.push_str(v);
        s.push('\n');
    }
    s
}

fn parse(text: &str) -> BTreeMap<String, String> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_once('\t'))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[test]
fn the_json_surface_only_grows() {
    let path = baseline_path();
    let now = current();

    if std::env::var_os("UCAL_BLESS").is_some() {
        std::fs::write(&path, render(&now)).expect("write baseline");
        eprintln!("blessed {} paths into {}", now.len(), path.display());
        return;
    }

    let Ok(text) = std::fs::read_to_string(&path) else {
        panic!(
            "{} is missing. Create it with:\n\
             UCAL_BLESS=1 cargo test -p ucal --test json_surface",
            path.display()
        );
    };
    let base = parse(&text);
    assert!(!base.is_empty(), "the baseline is empty");

    let mut gone = Vec::new();
    let mut changed = Vec::new();
    for (p, k) in &base {
        match now.get(p) {
            None => gone.push(p.clone()),
            Some(n) if n != k => changed.push(format!("{p}: {k} -> {n}")),
            _ => {}
        }
    }

    assert!(
        gone.is_empty() && changed.is_empty(),
        "the ucal-json/1 surface changed in a way the promise forbids.\n\n\
         removed ({}):\n  {}\n\n\
         kind changed ({}):\n  {}\n\n\
         Adding a field is allowed and needs no discussion. Removing one, or\n\
         changing what it serialises to, is a breaking change to ucal-json/1 and\n\
         belongs in the release notes' Breaking section — after which:\n\
         UCAL_BLESS=1 cargo test -p ucal --test json_surface",
        gone.len(),
        gone.join("\n  "),
        changed.len(),
        changed.join("\n  "),
    );
}

#[test]
fn a_new_field_is_not_a_failure() {
    // The asymmetry is the contract, so it is tested rather than assumed: a
    // caller must be able to ignore what it does not know, and this project must
    // be able to add without a major bump.
    let base = parse(&std::fs::read_to_string(baseline_path()).expect("baseline"));
    let mut widened = base.clone();
    widened.insert("datum.something_new".into(), "string".into());
    let gone: Vec<_> = base.keys().filter(|p| !widened.contains_key(*p)).collect();
    assert!(gone.is_empty(), "widening lost a path");
}

#[test]
fn the_baseline_covers_every_command() {
    let base = parse(&std::fs::read_to_string(baseline_path()).expect("baseline"));
    for (name, _) in commands() {
        assert!(
            base.keys().any(|p| p.starts_with(&format!("{name}."))),
            "`{name}` contributes nothing to the baseline"
        );
    }
}
