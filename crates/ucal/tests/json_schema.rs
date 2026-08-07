//! The published JSON Schema describes what the program actually emits.
//!
//! `fixtures/ucal-json-1.schema.json` is generated from the surface baseline by
//! `cargo run -p xtask -- gen-schema`, and `check-docs` fails if the committed
//! copy is stale. Neither of those checks the thing that matters to a consumer:
//! **that a document the program produces validates against it.**
//!
//! This does, without a JSON Schema implementation — the workspace has no
//! validator and Rule E's spirit is not to acquire one for a test. What it
//! checks instead is the property the schema is derived from, in both
//! directions:
//!
//! - every field a document emits is described by the schema;
//! - every leaf the schema describes has the JSON type it claims.
//!
//! That is not full validation, and the module says so rather than letting a
//! green run imply it. Full validation was run once, by hand, against a real
//! validator across all twenty-one commands with and without `--bridge` — and
//! it is what found that `required` was wrong.

use std::collections::BTreeMap;

/// The committed schema, as text.
fn schema_text() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("fixtures/ucal-json-1.schema.json");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// The surface baseline, as `path -> kind`.
fn baseline() -> BTreeMap<String, String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("fixtures/json-surface.txt");
    std::fs::read_to_string(&path)
        .expect("baseline")
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_once('\t'))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// The schema names every command in the baseline, and no others.
///
/// A `$defs` entry that no command produces would be a description of something
/// that does not exist; a command with no entry is a consumer with nothing to
/// validate against.
#[test]
fn every_command_has_a_definition() {
    let schema = schema_text();
    let base = baseline();
    let commands: Vec<&str> = base
        .keys()
        .filter_map(|p| p.split('.').next())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    assert!(!commands.is_empty());

    for c in &commands {
        assert!(
            schema.contains(&format!("\"{c}\": {{")),
            "the schema has no definition for `{c}`"
        );
    }
    // And the count matches, so a stale entry cannot hide.
    let defs = schema.matches("\n    \"").count();
    assert_eq!(
        defs,
        commands.len(),
        "the schema defines {defs} commands and the baseline has {}",
        commands.len()
    );
}

/// Nothing is `required`, and that is deliberate.
///
/// The baseline is a union over documents: `explain.claim` appears only under
/// `--claim`, a legacy calendar row has no `anchor_revision`, an event without
/// a warning has no `warning`. Promise 4 says a field never changes name, shape
/// or meaning — not that it is always emitted. A `required` list would make the
/// schema stricter than the promise and reject the program's own output.
///
/// The first generated schema had one and did exactly that, which is why this
/// test exists rather than a comment.
#[test]
fn the_schema_requires_nothing() {
    assert!(
        !schema_text().contains("\"required\""),
        "a `required` list makes the schema stricter than ucal-json/1 promises, \
         and rejects documents this program emits"
    );
}

/// Every object permits additional properties.
///
/// The contract is that new fields may appear and a consumer ignores what it
/// does not recognise. A schema that closed an object would break on the first
/// minor release that added a field — which the surface baseline explicitly
/// permits without a major bump.
#[test]
fn every_object_stays_open() {
    let s = schema_text();
    let objects = s.matches("\"type\": \"object\"").count();
    let open = s.matches("\"additionalProperties\"").count();
    assert!(objects > 0);
    assert_eq!(
        objects, open,
        "{objects} objects and {open} `additionalProperties`: an object that \
         forbids unknown fields contradicts promise 4"
    );
}

/// A row-keyed object is expressed as `additionalProperties`, not as names.
///
/// `cal-list`'s keys are calendar ids and `events-list`'s are event ids. If
/// those became named properties, adding a body or an event would change the
/// schema — and 0.8.0 added four bodies without the surface moving, which is
/// the property being preserved here.
#[test]
fn table_keys_do_not_become_property_names() {
    let s = schema_text();
    for absent in ["\"earth-d\"", "\"recombination\"", "\"T32\"", "\"mercury-d\""] {
        assert!(
            !s.contains(absent),
            "{absent} is data, not schema — a row key must not become a property name"
        );
    }
}

/// Kinds carry across: a `bool` in the baseline is a `boolean` in the schema.
#[test]
fn the_kinds_are_translated_not_invented() {
    let s = schema_text();
    let base = baseline();
    let bools = base.values().filter(|k| k.starts_with("bool")).count();
    let arrays = base.values().filter(|k| k.starts_with("array")).count();
    assert!(bools > 0 && arrays > 0, "the baseline should have both");
    assert_eq!(
        s.matches("\"type\": \"boolean\"").count(),
        bools,
        "every `bool` path should appear as a boolean"
    );
    assert_eq!(
        s.matches("\"type\": \"array\"").count(),
        arrays,
        "every `array` path should appear as an array"
    );
}
