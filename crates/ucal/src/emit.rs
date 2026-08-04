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

use crate::style::{group_decimal, paint_form, Render, Role, Style};

/// The `--json` schema version (§19.1).
pub const JSON_FORMAT: &str = "ucal-json/1";

/// A rendered value: a scalar, a list, or a nested section.
///
/// `#[non_exhaustive]`: a consumer must carry a wildcard arm. Added in 0.3.0,
/// which already broke exhaustive matches by introducing `Rows` and `Form` — so
/// the cost of requiring the arm was paid this release either way, and paying it
/// once is better than paying it again at every future variant.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
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
    /// A row-shaped field, rendered as a table for a person.
    ///
    /// Holds exactly what [`Value::Section`] holds and serialises identically —
    /// the variant tells the *text* renderer that the shape is a grid, and
    /// touches `ucal-json/1` not at all. See [`crate::table`].
    Rows {
        /// Header for the column holding each row's key.
        key: String,
        /// Header for the value column, when the rows are scalars rather than
        /// sections. `ucal ruler` is the case: an index and a mark.
        value: Option<String>,
        /// The rows, in order.
        rows: Vec<(String, Value)>,
    },
    /// A rendered timestamp form: a `UC1` text form, a `UC1/5` form, or a UCID.
    ///
    /// Distinguished from [`Value::Text`] only so the renderer can tell the
    /// leading zero run from the digits. In JSON it is a string, identical to
    /// what `Text` emits, which is why this variant does not touch
    /// `ucal-json/1`.
    Form(String),
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
    /// A rendered timestamp form.
    pub fn form(s: impl Into<String>) -> Value {
        Value::Form(s.into())
    }
    /// The rows of a section or a table, whichever this is.
    pub fn as_rows(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Section(f) => Some(f),
            Value::Rows { rows, .. } => Some(rows),
            _ => None,
        }
    }
    /// Rows of sections, rendered as a table.
    pub fn rows(key: impl Into<String>, rows: Vec<(String, Value)>) -> Value {
        Value::Rows {
            key: key.into(),
            value: None,
            rows,
        }
    }
    /// Rows of scalars, rendered as two columns.
    pub fn rows_of(
        key: impl Into<String>,
        value: impl Into<String>,
        rows: Vec<(String, Value)>,
    ) -> Value {
        Value::Rows {
            key: key.into(),
            value: Some(value.into()),
            rows,
        }
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

    /// Every field, in order. For tests that must walk what a command emitted.
    pub fn fields(&self) -> &[(String, Value)] {
        &self.fields
    }

    /// The field keys, in order. Used by the golden tests to pin §19.2's ordering.
    pub fn keys(&self) -> Vec<&str> {
        self.fields.iter().map(|(k, _)| k.as_str()).collect()
    }

    /// Look up a field's rows, whether it is a section or a table.
    ///
    /// A consumer reading structure should not have to know which of the two a
    /// command chose — that choice is a rendering decision, and this is the
    /// accessor that keeps it one.
    pub fn rows(&self, key: &str) -> Option<&[(String, Value)]> {
        self.get(key).and_then(Value::as_rows)
    }

    /// Look up a field.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.fields.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Render for a person, without colour.
    ///
    /// Defined as [`to_ansi`](Doc::to_ansi) against [`Style::PLAIN`] rather than
    /// as a renderer of its own. One code path means the coloured and plain
    /// layouts cannot drift, and it is what makes the strip invariant a statement
    /// about styles rather than about two functions agreeing.
    pub fn to_text(&self) -> String {
        self.render(&Render::PLAIN)
    }

    /// Render for a person, with colour and no group separator.
    pub fn to_ansi(&self, style: &Style) -> String {
        self.render(&Render::styled(*style))
    }

    /// Render for a person, with colour.
    ///
    /// The layout is computed from the *unpainted* text and the sequences are
    /// added last, so a coloured column lines up with a plain one. Getting this
    /// backwards is the classic defect here: `{k:<width$}` on an already-painted
    /// string pads to the width of the escape sequences.
    pub fn render(&self, r: &Render) -> String {
        let style = &r.style;
        let mut s = String::new();
        if let Some(t) = &self.title {
            let _ = writeln!(s, "{}", style.paint(Role::Title, t));
            let rule = "─".repeat(t.chars().count());
            let _ = writeln!(s, "{}", style.paint(Role::Title, &rule));
        }
        let width = self
            .fields
            .iter()
            .filter(|(_, v)| !matches!(v, Value::Section(_) | Value::List(_) | Value::Rows { .. }))
            .map(|(k, _)| k.chars().count())
            .max()
            .unwrap_or(0);
        for (k, v) in &self.fields {
            render_field_text(&mut s, k, v, width, 0, r);
        }
        for n in &self.notes {
            // A trailing note hangs at column zero, so it wraps to the margin
            // rather than to a field's value column.
            let painted = style.paint(role_of_prose(n), n);
            let body = if n.chars().count() > r.cols {
                crate::table::wrap_painted(&painted, 0, 0, r.cols)
            } else {
                painted
            };
            let _ = writeln!(s, "\n{body}");
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

/// Which role a run of prose takes.
///
/// A diagnostic code is the one thing in this output that a reader scans for
/// rather than reads, so it earns a colour. The test is the emitted code itself —
/// `UCAL-W` for a warning, `UCAL-E` for an error — and not a guess at tone, so a
/// note that happens to read gravely stays a note.
fn role_of_prose(t: &str) -> Role {
    if t.contains("UCAL-W") {
        Role::Warning
    } else if t.contains("UCAL-E") {
        Role::Error
    } else {
        Role::Note
    }
}

/// Paint one scalar value.
///
/// Shared with [`crate::table`] so a cell and a field render a value the same
/// way — grouping, leading-zero dimming and diagnostic colour all follow the
/// value into a grid.
pub(crate) fn render_scalar(r: &Render, v: &Value) -> String {
    match v {
        Value::Text(t) => r.style.paint(role_of_prose(t), t),
        Value::Number(n) => group_decimal(r, n),
        Value::Form(f) => paint_form(r, f),
        Value::Bool(b) => r.style.paint(Role::Value, &b.to_string()),
        // Not scalars; `crate::table::render` refuses rows containing them, and
        // the field renderer handles them before reaching here.
        Value::Section(_) | Value::List(_) | Value::Rows { .. } => String::new(),
    }
}

/// Pad `text` to `width` *display* columns, then paint it.
///
/// The order is the point: padding is measured on the characters a reader sees,
/// and the sequences go on afterwards.
fn padded(style: &Style, role: Role, text: &str, width: usize) -> String {
    let n = text.chars().count();
    let mut out = style.paint(role, text);
    for _ in n..width {
        out.push(' ');
    }
    out
}

/// Write one `key  value` line, wrapping the value under itself when it is too
/// long for the width.
///
/// A tick count is 61 digits and a base-5 form is over 200. Left alone, the
/// terminal wraps them back to column zero, so the second half of a value lands
/// under the field names and reads as though it belonged to a different row.
/// Hanging it under its own column keeps it one value.
///
/// Nothing is shortened and nothing is broken that has an alternative: the wrap
/// prefers a separator, and the value is recoverable by rejoining the lines.
fn write_field(s: &mut String, r: &Render, pad: &str, key: &str, width: usize, painted: &str) {
    let style = &r.style;
    let label = padded(style, Role::Key, key, width);
    let col = pad.chars().count() + width.max(key.chars().count()) + 2;
    let shown = crate::style::strip_ansi(painted).chars().count();
    let body = if col + shown > r.cols {
        crate::table::wrap_painted(painted, col, col, r.cols)
    } else {
        painted.to_string()
    };
    let _ = writeln!(s, "{pad}{label}  {body}");
}

fn render_field_text(s: &mut String, k: &str, v: &Value, width: usize, depth: usize, r: &Render) {
    let style = &r.style;
    let pad = "  ".repeat(depth);
    match v {
        Value::Section(fields) => {
            let _ = writeln!(s, "{pad}{}:", style.paint(Role::Key, k));
            let inner = fields
                .iter()
                .filter(|(_, v)| !matches!(v, Value::Section(_) | Value::List(_) | Value::Rows { .. }))
                .map(|(k, _)| k.chars().count())
                .max()
                .unwrap_or(0);
            for (ik, iv) in fields {
                render_field_text(s, ik, iv, inner, depth + 1, r);
            }
        }
        Value::List(items) => {
            let _ = writeln!(s, "{pad}{}:", style.paint(Role::Key, k));
            let col = pad.chars().count() + 2;
            for i in items {
                let painted = style.paint(role_of_prose(i), i);
                let body = if col + i.chars().count() > r.cols {
                    crate::table::wrap_painted(&painted, col, col, r.cols)
                } else {
                    painted
                };
                let _ = writeln!(s, "{pad}  {body}");
            }
        }
        Value::Text(t) => {
            write_field(s, r, &pad, k, width, &style.paint(role_of_prose(t), t));
        }
        Value::Number(n) => {
            write_field(s, r, &pad, k, width, &group_decimal(r, n));
        }
        Value::Rows { key, value, rows } => {
            let _ = writeln!(s, "{pad}{}:", style.paint(Role::Key, k));
            let indent = pad.len() + 2;
            let mut body = String::new();
            if crate::table::render(&mut body, r, indent, key, value.as_deref(), rows) {
                s.push_str(&body);
            } else {
                // Not grid-shaped. The nested rendering is still correct, and a
                // guessed flattening would not be.
                for (rk, rv) in rows {
                    render_field_text(s, rk, rv, 0, depth + 1, r);
                }
            }
        }
        Value::Form(f) => {
            write_field(s, r, &pad, k, width, &paint_form(r, f));
        }
        Value::Bool(b) => {
            write_field(s, r, &pad, k, width, &style.paint(Role::Value, &b.to_string()));
        }
    }
}

fn render_value_json(s: &mut String, v: &Value, depth: usize) {
    let pad = "  ".repeat(depth);
    match v {
        // A Form is a string in JSON, byte-identical to what Text emits. That
        // is the whole reason the variant does not bump `ucal-json/1`.
        Value::Text(t) | Value::Form(t) => {
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
        // Rows is Section with a rendering hint. Emitting it through the same
        // arm is what keeps `ucal-json/1` fixed.
        Value::Rows { rows, .. } => render_value_json(s, &Value::Section(rows.clone()), depth),
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
