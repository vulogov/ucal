//! Output emission: one command, two renderings (§19.1).
//!
//! Every command builds a [`Doc`] rather than printing. A `Doc` renders either as
//! text for a person or as JSON for a program, and `--json` output is **stable
//! and versioned** as §19.1 requires — the version is emitted in every document
//! so a consumer can tell when it changes.
//!
//! Building a structure rather than printing has a second benefit, which is why
//! the golden tests are cheap: a command is a pure function from arguments to a
//! `Doc`, so it can be tested without spawning a process or capturing a pipe.

use std::fmt::Write as _;

/// The `--json` schema version (§19.1).
pub const JSON_FORMAT: &str = "ucal-json/1";

/// A rendered value: a scalar, a list, or a nested section.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    /// A string. Rendered verbatim in text, quoted in JSON.
    Text(String),
    /// An integer, held as a decimal string because tick counts exceed `u64`.
    Number(String),
    /// A boolean.
    Bool(bool),
    /// An ordered list of strings.
    List(Vec<String>),
    /// A nested section.
    Section(Vec<(String, Value)>),
}

impl Value {
    /// A string value.
    pub fn text(s: impl Into<String>) -> Value {
        Value::Text(s.into())
    }
    /// A numeric value, given as a decimal string.
    pub fn number(s: impl Into<String>) -> Value {
        Value::Number(s.into())
    }
    /// A list of strings.
    pub fn list<I: IntoIterator<Item = S>, S: Into<String>>(it: I) -> Value {
        Value::List(it.into_iter().map(Into::into).collect())
    }
}

/// A document: a titled, ordered sequence of fields.
///
/// Order is significant. §19.2 requires `ucal datum` to print its parts in a
/// stated order, so the container preserves insertion order rather than sorting.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Doc {
    title: Option<String>,
    fields: Vec<(String, Value)>,
    notes: Vec<String>,
}

impl Doc {
    /// An empty document.
    pub fn new() -> Doc {
        Doc::default()
    }

    /// Set the heading shown in text mode.
    pub fn title(mut self, t: impl Into<String>) -> Doc {
        self.title = Some(t.into());
        self
    }

    /// Append a field. Insertion order is preserved and is part of the contract.
    pub fn field(mut self, k: impl Into<String>, v: Value) -> Doc {
        self.fields.push((k.into(), v));
        self
    }

    /// Append a trailing note, shown after the fields in text mode.
    pub fn note(mut self, n: impl Into<String>) -> Doc {
        self.notes.push(n.into());
        self
    }

    /// The field keys, in order. Used by the golden tests to pin §19.2's ordering.
    pub fn keys(&self) -> Vec<&str> {
        self.fields.iter().map(|(k, _)| k.as_str()).collect()
    }

    /// Look up a field.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Render for a person.
    pub fn to_text(&self) -> String {
        let mut s = String::new();
        if let Some(t) = &self.title {
            let _ = writeln!(s, "{t}");
            let _ = writeln!(s, "{}", "─".repeat(t.chars().count()));
        }
        let width = self
            .fields
            .iter()
            .filter(|(_, v)| !matches!(v, Value::Section(_) | Value::List(_)))
            .map(|(k, _)| k.chars().count())
            .max()
            .unwrap_or(0);
        for (k, v) in &self.fields {
            render_field_text(&mut s, k, v, width, 0);
        }
        for n in &self.notes {
            let _ = writeln!(s, "\n{n}");
        }
        s
    }

    /// Render for a program (§19.1).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        let _ = writeln!(s, "{{");
        let _ = writeln!(s, "  \"format\": \"{JSON_FORMAT}\",");
        for (i, (k, v)) in self.fields.iter().enumerate() {
            let comma = if i + 1 == self.fields.len() && self.notes.is_empty() {
                ""
            } else {
                ","
            };
            let _ = write!(s, "  \"{}\": ", escape(k));
            render_value_json(&mut s, v, 1);
            let _ = writeln!(s, "{comma}");
        }
        if !self.notes.is_empty() {
            let _ = write!(s, "  \"notes\": ");
            render_value_json(&mut s, &Value::List(self.notes.clone()), 1);
            let _ = writeln!(s);
        }
        let _ = writeln!(s, "}}");
        s
    }
}

fn render_field_text(s: &mut String, k: &str, v: &Value, width: usize, depth: usize) {
    let pad = "  ".repeat(depth);
    match v {
        Value::Section(fields) => {
            let _ = writeln!(s, "{pad}{k}:");
            let inner = fields
                .iter()
                .filter(|(_, v)| !matches!(v, Value::Section(_) | Value::List(_)))
                .map(|(k, _)| k.chars().count())
                .max()
                .unwrap_or(0);
            for (ik, iv) in fields {
                render_field_text(s, ik, iv, inner, depth + 1);
            }
        }
        Value::List(items) => {
            let _ = writeln!(s, "{pad}{k}:");
            for i in items {
                let _ = writeln!(s, "{pad}  {i}");
            }
        }
        Value::Text(t) => {
            let _ = writeln!(s, "{pad}{k:<width$}  {t}");
        }
        Value::Number(n) => {
            let _ = writeln!(s, "{pad}{k:<width$}  {n}");
        }
        Value::Bool(b) => {
            let _ = writeln!(s, "{pad}{k:<width$}  {b}");
        }
    }
}

fn render_value_json(s: &mut String, v: &Value, depth: usize) {
    let pad = "  ".repeat(depth);
    match v {
        Value::Text(t) => {
            let _ = write!(s, "\"{}\"", escape(t));
        }
        // Tick counts exceed every JSON number implementation in practice, and a
        // consumer that silently converted one to a double would lose the
        // exactness the whole specification exists to provide. They are emitted
        // as strings, deliberately.
        Value::Number(n) => {
            let _ = write!(s, "\"{}\"", escape(n));
        }
        Value::Bool(b) => {
            let _ = write!(s, "{b}");
        }
        Value::List(items) => {
            if items.is_empty() {
                let _ = write!(s, "[]");
                return;
            }
            let _ = writeln!(s, "[");
            for (i, it) in items.iter().enumerate() {
                let comma = if i + 1 == items.len() { "" } else { "," };
                let _ = writeln!(s, "{pad}  \"{}\"{comma}", escape(it));
            }
            let _ = write!(s, "{pad}]");
        }
        Value::Section(fields) => {
            if fields.is_empty() {
                let _ = write!(s, "{{}}");
                return;
            }
            let _ = writeln!(s, "{{");
            for (i, (k, val)) in fields.iter().enumerate() {
                let comma = if i + 1 == fields.len() { "" } else { "," };
                let _ = write!(s, "{pad}  \"{}\": ", escape(k));
                render_value_json(s, val, depth + 1);
                let _ = writeln!(s, "{comma}");
            }
            let _ = write!(s, "{pad}}}");
        }
    }
}

fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_is_versioned() {
        let d = Doc::new().field("a", Value::text("b"));
        assert!(d.to_json().contains("\"format\": \"ucal-json/1\""));
    }

    #[test]
    fn field_order_is_preserved() {
        // §19.2 makes ordering part of the contract, so the container must not
        // sort or deduplicate.
        let d = Doc::new()
            .field("z", Value::text("1"))
            .field("a", Value::text("2"))
            .field("m", Value::text("3"));
        assert_eq!(d.keys(), ["z", "a", "m"]);
        let json = d.to_json();
        let zi = json.find("\"z\"").unwrap();
        let ai = json.find("\"a\"").unwrap();
        let mi = json.find("\"m\"").unwrap();
        assert!(zi < ai && ai < mi);
    }

    #[test]
    fn tick_counts_are_emitted_as_strings() {
        // A 61-digit integer cannot survive a JSON double. Emitting it as a
        // string is what keeps the exactness the specification promises.
        let big = "8070205189123984864657505252035637180530466139316558837890625";
        let d = Doc::new().field("ticks", Value::number(big));
        assert!(d.to_json().contains(&format!("\"ticks\": \"{big}\"")));
    }

    #[test]
    fn strings_are_escaped() {
        let d = Doc::new().field("k", Value::text("a\"b\\c\nd"));
        let j = d.to_json();
        assert!(j.contains(r#""a\"b\\c\nd""#));
    }

    #[test]
    fn nested_sections_render_both_ways() {
        let d = Doc::new().title("T").field(
            "outer",
            Value::Section(vec![
                ("inner".into(), Value::number("42")),
                ("list".into(), Value::list(["x", "y"])),
            ]),
        );
        let t = d.to_text();
        assert!(t.contains("outer:"));
        assert!(t.contains("inner"));
        assert!(t.contains("42"));
        let j = d.to_json();
        assert!(j.contains("\"outer\""));
        assert!(j.contains("\"inner\": \"42\""));
        assert!(j.contains("\"x\""));
    }

    #[test]
    fn empty_containers_are_valid_json() {
        let d = Doc::new()
            .field("l", Value::List(vec![]))
            .field("s", Value::Section(vec![]));
        let j = d.to_json();
        assert!(j.contains("\"l\": []"));
        assert!(j.contains("\"s\": {}"));
    }
}
