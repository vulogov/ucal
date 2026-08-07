//! M3 — a JSON Schema for `ucal-json/1`, generated from the surface baseline.
//!
//! # Why generated, and generated from *that* file
//!
//! `Documentation/STABILITY.md` promise 4 says a `ucal-json/1` field never
//! changes name, shape or meaning, and that new fields may appear. A consumer
//! has had that as a sentence and nothing to validate against.
//!
//! The schema is derived from `fixtures/json-surface.txt`, which is not a
//! description of the surface but the *definition* of it: every field path and
//! the JSON kind it serialises to, regenerated from the real documents with
//! `UCAL_BLESS=1` and checked on every push by `json_surface.rs`. Deriving the
//! schema from that file means the two cannot disagree — a hand-written schema
//! would be a third copy of the surface, and this project has learned what a
//! third copy does.
//!
//! # The shape of what comes out
//!
//! One `$defs` entry per command, because `--json` output is per-command. A
//! consumer validating `ucal datum --json` picks `#/$defs/datum`.
//!
//! Three properties of the surface are encoded rather than described:
//!
//! **Additive growth.** `additionalProperties` is true everywhere, because a
//! new field is permitted and a consumer must ignore what it does not know.
//! Refusing unknown fields would make the schema stricter than the promise and
//! break on the first minor release.
//!
//! **Nothing is `required`, and that is not laziness.** The first version
//! marked every baseline field required, on the reading that a field which
//! never disappears is a field always present. Running the schema against real
//! output refuted it immediately: `explain.claim` appears only under `--claim`;
//! `cal-list`'s rows carry `anchor_revision` and `body` for a derived calendar
//! and not for a legacy one; `events-list` carries `warning` only for the
//! events that have one.
//!
//! The baseline is a **union** over documents, not a per-document contract, and
//! promise 4 says a field never changes *name, shape or meaning* — it does not
//! say a field is always emitted. A schema with `required` would have been
//! stricter than the promise and would have rejected the program's own output,
//! which is the failure a schema exists to prevent rather than to commit.
//!
//! **Row keys are data.** A `*` segment in the baseline is a table key — a
//! tier, a body, an event id — and becomes `additionalProperties` on that
//! object rather than a property name, so adding a body does not change the
//! schema.

use std::collections::BTreeMap;
use std::path::Path;

/// A node in the surface tree.
#[derive(Default)]
struct Node {
    kind: Option<String>,
    bridge: bool,
    /// Named children.
    fields: BTreeMap<String, Node>,
    /// The `*` child, if this object's keys are data.
    rows: Option<Box<Node>>,
}

impl Node {
    fn insert(&mut self, path: &[&str], kind: &str, bridge: bool) {
        match path.split_first() {
            None => {
                self.kind = Some(kind.to_string());
                self.bridge = bridge;
            }
            Some((head, rest)) if *head == "*" => {
                let child = self.rows.get_or_insert_with(|| Box::new(Node::default()));
                child.insert(rest, kind, bridge);
            }
            Some((head, rest)) => {
                self.fields
                    .entry((*head).to_string())
                    .or_default()
                    .insert(rest, kind, bridge);
            }
        }
    }
}

/// Render a node as a JSON Schema fragment.
fn render(node: &Node, indent: usize) -> String {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let kind = node.kind.as_deref().unwrap_or("object");

    if kind != "object" {
        let ty = match kind {
            "bool" => "boolean",
            "array" => "array",
            _ => "string",
        };
        if ty == "array" {
            return format!(
                "{{\n{inner}\"type\": \"array\",\n{inner}\"items\": {{ \"type\": \"string\" }}\n{pad}}}"
            );
        }
        return format!("{{ \"type\": \"{ty}\" }}");
    }

    let mut out = format!("{{\n{inner}\"type\": \"object\"");

    if !node.fields.is_empty() {
        out.push_str(&format!(",\n{inner}\"properties\": {{"));
        let mut first = true;
        for (name, child) in &node.fields {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&format!(
                "\n{inner}  {}: {}",
                json_string(name),
                render(child, indent + 4)
            ));
        }
        out.push_str(&format!("\n{inner}}}"));
    }

    match &node.rows {
        // Keys are data: any key, with this shape.
        Some(rows) => out.push_str(&format!(
            ",\n{inner}\"additionalProperties\": {}",
            render(rows, indent + 2)
        )),
        // A new field may appear at any time (promise 4).
        None => out.push_str(&format!(",\n{inner}\"additionalProperties\": true")),
    }

    out.push_str(&format!("\n{pad}}}"));
    out
}

fn json_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Build the schema text from the surface baseline.
pub fn generate(root: &Path) -> Result<String, String> {
    let path = root.join("fixtures/json-surface.txt");
    let text = std::fs::read_to_string(&path)
        .map_err(|e| format!("{}: {e}", path.display()))?;

    let mut commands: BTreeMap<String, Node> = BTreeMap::new();
    let mut paths = 0usize;
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((path, kind)) = line.split_once('\t') else {
            continue;
        };
        let bridge = kind.contains("(--bridge)");
        let kind = kind.split_whitespace().next().unwrap_or("string");
        let segments: Vec<&str> = path.split('.').collect();
        let Some((command, rest)) = segments.split_first() else {
            continue;
        };
        commands
            .entry((*command).to_string())
            .or_default()
            .insert(rest, kind, bridge);
        paths += 1;
    }
    if commands.is_empty() {
        return Err("the surface baseline yielded no commands".into());
    }

    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n");
    out.push_str("  \"$id\": \"https://github.com/vulogov/ucal/blob/main/fixtures/ucal-json-1.schema.json\",\n");
    out.push_str("  \"title\": \"ucal-json/1\",\n");
    out.push_str(&format!(
        "  \"description\": \"Generated from fixtures/json-surface.txt by `cargo run -p xtask -- gen-schema`; do not edit. {paths} field paths across {} commands. `--json` output is per-command: validate `ucal datum --json` against #/$defs/datum. Every object permits additional properties, because ucal-json/1 promises that new fields may appear and existing ones never change name, shape or meaning; a consumer must ignore what it does not recognise. Fields shown only under `--bridge` are described but not required. An object whose keys are data - a tier, a body, an event id - is expressed as additionalProperties rather than named properties, so adding one changes no schema.\",\n",
        commands.len()
    ));
    out.push_str("  \"$defs\": {");
    let mut first = true;
    for (name, node) in &commands {
        if !first {
            out.push(',');
        }
        first = false;
        out.push_str(&format!(
            "\n    {}: {}",
            json_string(name),
            render(node, 4)
        ));
    }
    out.push_str("\n  }\n}\n");
    Ok(out)
}

/// Write the schema. Returns the path written.
pub fn write(root: &Path) -> Result<std::path::PathBuf, String> {
    let text = generate(root)?;
    let path = root.join("fixtures/ucal-json-1.schema.json");
    std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(path)
}

/// Fail if the committed schema is not what the baseline would produce.
///
/// The same shape as §13.5's generated-docs check: a generated artefact that is
/// committed must be regenerable, or it is a copy that has started to drift.
pub fn check(root: &Path) -> Result<usize, String> {
    let want = generate(root)?;
    let path = root.join("fixtures/ucal-json-1.schema.json");
    let have = std::fs::read_to_string(&path)
        .map_err(|_| format!("{} is missing; run `cargo run -p xtask -- gen-schema`", path.display()))?;
    if have == want {
        Ok(want.lines().count())
    } else {
        Err(format!(
            "{} is stale; run `cargo run -p xtask -- gen-schema`",
            path.display()
        ))
    }
}
