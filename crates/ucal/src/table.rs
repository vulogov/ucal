//! Table rendering for row-shaped fields.
//!
//! # Why this changes no JSON
//!
//! [`Value::Rows`](crate::emit::Value::Rows) holds exactly what
//! [`Value::Section`](crate::emit::Value::Section) holds — an ordered map from a
//! row key to that row's fields — and serialises identically. The variant exists
//! so the *text* renderer knows the shape is a grid. §19.1 makes `--json` a
//! stable, versioned contract, and a legibility change has no business moving it.
//!
//! # Width
//!
//! 80 columns is the baseline and the assumed width whenever stdout is not a
//! terminal. That is not a stylistic preference: if width followed the terminal
//! on a redirected stream, the same command would emit different bytes into a
//! pipe than into a file, and the golden tests would be testing the machine they
//! ran on. A wider terminal is used when there is one; the floor is never
//! crossed downward.
//!
//! # What happens to a column that does not fit
//!
//! It is **promoted**, not truncated and not dropped. A tick count is 61 digits
//! and a base-5 form is over 200; neither fits any grid, and an exact integer
//! with an ellipsis in it is not an exact integer. A promoted column is written
//! on its own continuation line beneath its row, entire, wrapping at its own
//! separators when it has to:
//!
//! ```text
//! tier  exponent  name   beats
//! ────  ────────  ─────  ──────────────────
//! T5    85        deep   298023223876953125
//!       ticks     258493941422821148397315216271863391739316284656524658203125
//! ```

use std::fmt::Write as _;

use crate::emit::Value;
use crate::style::{strip_ansi, Render, Role};

/// The documented baseline width, and the assumed width off a terminal.
pub const BASELINE_WIDTH: usize = 80;

/// Gutter between columns, in spaces.
const GUTTER: usize = 2;

/// The cell shown where a row has no value for a column.
///
/// Rows are not required to be uniform — an event carries a `warning` field only
/// when it has one — and a blank cell would read as an empty value rather than
/// as an absent one.
const ABSENT: &str = "—";

/// Painted cell text, and the display width of what it will show.
fn cell(r: &Render, v: &Value) -> (String, usize) {
    let painted = crate::emit::render_scalar(r, v);
    let width = strip_ansi(&painted).chars().count();
    (painted, width)
}

/// Pad an already-painted run to `w` display columns.
fn pad_to(painted: &str, shown: usize, w: usize) -> String {
    let mut s = painted.to_string();
    for _ in shown..w {
        s.push(' ');
    }
    s
}

/// Whether a value can occupy a table cell.
///
/// A nested section or list has no single-line rendering, so a row containing one
/// is not a grid row. Rather than inventing a flattening, such a row falls back
/// to the nested rendering it would have had.
fn is_scalar(v: &Value) -> bool {
    match v {
        Value::Section(_) | Value::List(_) | Value::Rows { .. } => false,
        Value::Bridge(inner) => is_scalar(inner),
        _ => true,
    }
}

/// Render a row-shaped field as a table.
///
/// `key_header` names the column holding each row's key. `value_header` is
/// `Some` when the rows are scalars rather than sections, which is the two-column
/// case — `ucal ruler`, whose rows are an index and a mark.
///
/// Returns `false` when the rows are not grid-shaped, so the caller can fall back
/// to nested rendering rather than this module guessing at a layout.
pub fn render(
    out: &mut String,
    r: &Render,
    indent: usize,
    key_header: &str,
    value_header: Option<&str>,
    rows: &[(String, Value)],
) -> bool {
    if rows.is_empty() {
        return false;
    }
    let style = &r.style;

    // --- columns, in first-seen order -------------------------------------
    let mut columns: Vec<String> = Vec::new();
    match value_header {
        Some(h) => {
            if !rows.iter().all(|(_, v)| is_scalar(v)) {
                return false;
            }
            columns.push(h.to_string());
        }
        None => {
            for (_, v) in rows {
                let Value::Section(fields) = v else {
                    return false;
                };
                for (k, fv) in fields {
                    // A foreign-unit column is a column, and is omitted for the
                    // same reason a foreign-unit field is: `--bridge` was not
                    // asked for. Missing this left `ucal ladder`'s "seconds
                    // (bridge)" column in place while every other rendering had
                    // dropped it.
                    if matches!(fv, Value::Bridge(_)) && !r.bridge {
                        continue;
                    }
                    if is_scalar(fv) && !columns.iter().any(|c| c == k) {
                        columns.push(k.clone());
                    }
                }
            }
            if columns.is_empty() {
                return false;
            }
        }
    }

    // --- cells, painted once and measured once ----------------------------
    let absent = (style.paint(Role::Value, ABSENT), ABSENT.chars().count());
    let mut grid: Vec<Vec<(String, usize)>> = Vec::with_capacity(rows.len());
    for (_, v) in rows {
        let mut line = Vec::with_capacity(columns.len());
        for c in &columns {
            let found = match (value_header, v) {
                (Some(_), val) => Some(cell(r, val)),
                (None, Value::Section(fields)) => fields
                    .iter()
                    .find(|(k, _)| k == c)
                    .map(|(_, fv)| cell(r, fv)),
                _ => None,
            };
            line.push(found.unwrap_or_else(|| absent.clone()));
        }
        grid.push(line);
    }

    let key_w = rows
        .iter()
        .map(|(k, _)| k.chars().count())
        .chain(core::iter::once(key_header.chars().count()))
        .max()
        .unwrap_or(0);
    let col_w: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            grid.iter()
                .map(|row| row[i].1)
                .chain(core::iter::once(c.chars().count()))
                .max()
                .unwrap_or(0)
        })
        .collect();

    // --- fit ---------------------------------------------------------------
    // The key column is always in the grid; it is what a promoted line hangs
    // under. Data columns are taken in order while they fit, and the first one
    // that does not ends the grid — a later narrow column jumping ahead of an
    // earlier wide one would reorder the output for no reason a reader can see.
    // Written out rather than as `saturating_sub`, and not to appease the lint.
    // Rule O's objection is to arithmetic that quietly produces a wrong answer
    // instead of failing; an explicit branch says what the zero case is, and with
    // `overflow-checks = true` a subtraction the branch did not guard panics
    // rather than wrapping. That is the behaviour the rule wants.
    let avail = if r.cols > indent { r.cols - indent } else { 0 };
    let mut fitted = 0;
    let mut used = key_w;
    for w in &col_w {
        if used + GUTTER + w > avail {
            break;
        }
        used += GUTTER + w;
        fitted += 1;
    }

    let hang = indent + key_w + GUTTER;

    // --- header and rule ---------------------------------------------------
    let pad = " ".repeat(indent);
    let mut header = pad_to(&style.paint(Role::Key, key_header), key_header.chars().count(), key_w);
    for i in 0..fitted {
        header.push_str(&" ".repeat(GUTTER));
        header.push_str(&pad_to(
            &style.paint(Role::Key, &columns[i]),
            columns[i].chars().count(),
            col_w[i],
        ));
    }
    let _ = writeln!(out, "{pad}{}", header.trim_end());

    let mut rule = style.paint(Role::Separator, &"─".repeat(key_w));
    for w in col_w.iter().take(fitted) {
        rule.push_str(&" ".repeat(GUTTER));
        rule.push_str(&style.paint(Role::Separator, &"─".repeat(*w)));
    }
    let _ = writeln!(out, "{pad}{rule}");

    // --- rows --------------------------------------------------------------
    for (row_i, (key, _)) in rows.iter().enumerate() {
        let mut line = pad_to(&style.paint(Role::Value, key), key.chars().count(), key_w);
        for i in 0..fitted {
            let (painted, shown) = &grid[row_i][i];
            line.push_str(&" ".repeat(GUTTER));
            line.push_str(&pad_to(painted, *shown, col_w[i]));
        }
        let _ = writeln!(out, "{pad}{}", line.trim_end());

        for (i, name) in columns.iter().enumerate().skip(fitted) {
            let (painted, shown) = &grid[row_i][i];
            if *shown == 0 || strip_ansi(painted) == ABSENT {
                continue;
            }
            // Each promoted column hangs under its own label rather than under
            // a width shared with the others. Sharing is tidier and costs the
            // requirement: aligning `ticks` behind `seconds (bridge)` spends
            // eleven columns that the 61-digit value needs, and wraps a value
            // that fits inside 80 otherwise.
            write_promoted(out, r, hang, name.chars().count(), name, painted);
        }
    }
    true
}

/// One promoted column, beneath its row, in full.
///
/// Wrapping happens at the value's own separators when it has them, so a base-5
/// form breaks between groups rather than through one. A run with no separator
/// and no room — which no shipped value has, since 61 digits fit under 80
/// columns — is written long rather than broken: overflowing a terminal is a
/// visual problem, and splitting an exact integer across lines is a correctness
/// one.
fn write_promoted(
    out: &mut String,
    r: &Render,
    hang: usize,
    name_w: usize,
    name: &str,
    painted: &str,
) {
    let style = &r.style;
    let pad = " ".repeat(hang);
    let label = pad_to(&style.paint(Role::Key, name), name.chars().count(), name_w);
    let value_indent = hang + name_w + GUTTER;
    let avail = if r.cols > value_indent {
        r.cols - value_indent
    } else {
        0
    };

    let plain = strip_ansi(painted);
    if plain.chars().count() <= avail || avail < 8 {
        let _ = writeln!(out, "{pad}{label}{}{painted}", " ".repeat(GUTTER));
        return;
    }

    // Wrap the *plain* text and re-render each piece, rather than slicing a
    // painted string — cutting through an escape sequence would leave a terminal
    // in whatever state the fragment set.
    let pieces = wrap_at_separators(&plain, avail);
    for (i, piece) in pieces.iter().enumerate() {
        let l = if i == 0 {
            label.clone()
        } else {
            " ".repeat(name_w)
        };
        let _ = writeln!(
            out,
            "{pad}{l}{}{}",
            " ".repeat(GUTTER),
            style.paint(Role::Digits, piece)
        );
    }
}

/// Break `s` into runs of at most `width`.
///
/// Three preferences, in order: a word boundary, then any separator, then a hard
/// break. Prose gets broken between words; a grouped form with no spaces gets
/// broken between groups; a bare digit run gets broken wherever it must be, which
/// is better than a line that runs off the terminal.
///
/// A candidate is only taken if it is past the halfway mark, so a separator near
/// the start of a piece does not produce a nearly empty line.
fn wrap_at_separators(s: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut after_space = 0usize;
    let mut after_sep = 0usize;
    for c in s.chars() {
        cur.push(c);
        let n = cur.chars().count();
        if c.is_whitespace() {
            after_space = n;
        }
        if !c.is_alphanumeric() {
            after_sep = n;
        }
        if n >= width {
            let half = width / 2;
            let (cut, at_word) = if after_space > half {
                (after_space, true)
            } else if after_sep > half {
                (after_sep, false)
            } else {
                (n, false)
            };
            let head: String = cur.chars().take(cut).collect();
            let tail: String = cur.chars().skip(cut).collect();
            // The break character is dropped only when it is the space we broke
            // *at* — everywhere else nothing is discarded, so a value survives
            // wrapping intact and can be reassembled from the lines.
            out.push(if at_word {
                head.trim_end().to_string()
            } else {
                head
            });
            cur = tail;
            after_space = 0;
            after_sep = 0;
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::emit::Value;

    fn r(width: usize) -> Render {
        Render {
            cols: width,
            ..Render::PLAIN
        }
    }

    fn rows() -> Vec<(String, Value)> {
        vec![
            (
                "T5".into(),
                Value::Section(vec![
                    ("exponent".into(), Value::number("85")),
                    ("name".into(), Value::text("deep")),
                    (
                        "ticks".into(),
                        Value::number(
                            "258493941422821148397315216271863391739316284656524658203125",
                        ),
                    ),
                ]),
            ),
            (
                "T0".into(),
                Value::Section(vec![
                    ("exponent".into(), Value::number("60")),
                    ("name".into(), Value::text("beat")),
                    (
                        "ticks".into(),
                        Value::number("867361737988403547205962240695953369140625"),
                    ),
                ]),
            ),
        ]
    }

    fn render_to_string(width: usize) -> String {
        let mut s = String::new();
        assert!(render(&mut s, &r(width), 0, "tier", None, &rows()));
        s
    }

    #[test]
    fn no_line_exceeds_the_width() {
        for w in [80, 100, 120, 200] {
            for line in render_to_string(w).lines() {
                assert!(
                    line.chars().count() <= w,
                    "width {w}: line of {} chars: {line}",
                    line.chars().count()
                );
            }
        }
    }

    #[test]
    fn the_wide_column_is_promoted_whole_not_truncated() {
        let out = render_to_string(80);
        assert!(out.contains("258493941422821148397315216271863391739316284656524658203125"));
        assert!(out.contains("867361737988403547205962240695953369140625"));
        assert!(!out.contains('…'), "a value was elided");
        assert!(!out.contains("..."), "a value was elided");
    }

    #[test]
    fn a_wide_terminal_pulls_the_column_into_the_grid() {
        // 61 digits plus the other columns needs about 85; at 200 it belongs in
        // the grid, and the promoted line disappears.
        let narrow = render_to_string(80);
        let wide = render_to_string(200);
        assert!(narrow.lines().count() > wide.lines().count());
        // The header row carries the column once it is in the grid.
        assert!(wide.lines().next().unwrap().contains("ticks"));
        assert!(!narrow.lines().next().unwrap().contains("ticks"));
    }

    #[test]
    fn absent_cells_are_marked_not_blank() {
        let rows = vec![
            (
                "a".into(),
                Value::Section(vec![
                    ("x".into(), Value::text("1")),
                    ("warning".into(), Value::text("W")),
                ]),
            ),
            ("b".into(), Value::Section(vec![("x".into(), Value::text("2"))])),
        ];
        let mut s = String::new();
        assert!(render(&mut s, &r(80), 0, "k", None, &rows));
        let last = s.lines().last().unwrap();
        assert!(last.contains(ABSENT), "absent cell not marked: {last}");
    }

    #[test]
    fn a_row_holding_a_section_is_declined() {
        // Not a grid row. Returning false lets the caller fall back rather than
        // this module inventing a flattening.
        let rows = vec![(
            "a".into(),
            Value::Section(vec![(
                "nested".into(),
                Value::Section(vec![("y".into(), Value::text("1"))]),
            )]),
        )];
        let mut s = String::new();
        assert!(!render(&mut s, &r(80), 0, "k", None, &rows));
        assert!(s.is_empty(), "declined but still wrote output");
    }

    #[test]
    fn scalar_rows_render_as_two_columns() {
        let rows = vec![
            ("0".into(), Value::form("UC1 0000·0001")),
            ("1".into(), Value::form("UC1 0000·0002")),
        ];
        let mut s = String::new();
        assert!(render(&mut s, &r(80), 0, "n", Some("at"), &rows));
        assert!(s.lines().next().unwrap().starts_with("n"));
        assert!(s.lines().next().unwrap().contains("at"));
        assert!(s.contains("UC1 0000·0002"));
    }

    #[test]
    fn wrapping_a_value_loses_nothing() {
        // The property that matters for an exact quantity: the lines reassemble
        // into the value. Only a break taken *at* a space discards anything, and
        // a value has no spaces in it.
        let s = "00000.11111.22222.33333.44444.55555.66666.77777";
        for w in [12, 20, 33, 47] {
            let pieces = wrap_at_separators(s, w);
            assert_eq!(pieces.concat(), s, "width {w}: wrapping lost characters");
            for p in &pieces {
                assert!(p.chars().count() <= w, "width {w}: piece too long: {p}");
            }
        }
        let pieces = wrap_at_separators(s, 12);
        assert!(pieces[0].ends_with('.'), "did not break at a separator");
    }

    #[test]
    fn prose_breaks_between_words() {
        let s = "no anchor: complete in units, intercalation and cycles, incomplete in phase";
        let pieces = wrap_at_separators(s, 30);
        for p in &pieces {
            assert!(p.chars().count() <= 30, "piece too long: {p}");
            assert!(!p.ends_with(' '), "trailing space left on a wrapped line");
        }
        // Words survive: only the break-point spaces are dropped.
        assert_eq!(
            pieces.join(" ").split_whitespace().collect::<Vec<_>>(),
            s.split_whitespace().collect::<Vec<_>>()
        );
    }

    #[test]
    fn wrapping_a_run_with_no_separator_still_loses_nothing() {
        let s = "1234567890".repeat(9);
        let pieces = wrap_at_separators(&s, 25);
        assert_eq!(pieces.concat(), s);
        for p in &pieces {
            assert!(p.chars().count() <= 25);
        }
    }
}

/// Wrap an already-painted run to `width` columns, hanging under `indent`.
///
/// Works on the painted string rather than on plain text and then re-painting,
/// which matters: a rendered form carries several roles across its length — a
/// dimmed leading-zero region, banded digit groups, separators — and re-painting
/// wrapped pieces with one role would throw all of that away.
///
/// Escape sequences occupy no columns and are emitted where they fall, so a
/// colour opened before a break stays open across it and its reset arrives on
/// the continuation line. Break positions are computed from *visible characters
/// only*, which is what keeps the strip invariant true: the coloured rendering
/// breaks in exactly the places the plain one does.
///
/// `first` is the column the run starts at, so the first line gets the room it
/// actually has rather than a full width.
pub fn wrap_painted(painted: &str, first: usize, indent: usize, width: usize) -> String {
    if width <= indent + 8 {
        // No usable room for a hanging indent. Overflowing a terminal is a
        // visual problem; breaking a value into a shape nobody asked for is a
        // worse one.
        return painted.to_string();
    }

    // Split into escape sequences and visible characters, so column arithmetic
    // can ignore the former without losing them.
    enum Seg {
        Esc(String),
        Ch(char),
    }
    let mut segs = Vec::new();
    let mut it = painted.chars().peekable();
    while let Some(c) = it.next() {
        if c != '\u{1b}' {
            segs.push(Seg::Ch(c));
            continue;
        }
        let mut e = String::from(c);
        if let Some('[') = it.peek() {
            e.push(it.next().unwrap_or('['));
            for c in it.by_ref() {
                e.push(c);
                if ('\u{40}'..='\u{7e}').contains(&c) {
                    break;
                }
            }
        } else if let Some(c) = it.next() {
            e.push(c);
        }
        segs.push(Seg::Esc(e));
    }

    let mut out = String::with_capacity(painted.len());
    let mut line = String::new();
    let mut col = first;
    // Candidate break points: after the last space, and after the last
    // separator of any kind. Each records where in `line` it falls and what
    // column it would leave behind. A word boundary is preferred, so prose
    // breaks between words rather than after a bracket, and a value with no
    // spaces still breaks between its groups.
    let mut brk_space: Option<(usize, usize)> = None;
    let mut brk_sep: Option<(usize, usize)> = None;
    let flush = |out: &mut String, line: &mut String| {
        out.push_str(line);
        out.push('\n');
        out.push_str(&" ".repeat(indent));
        line.clear();
    };

    for seg in &segs {
        match seg {
            Seg::Esc(e) => line.push_str(e),
            Seg::Ch(c) => {
                if col >= width {
                    let half = (indent + width) / 2;
                    let chosen = match (brk_space, brk_sep) {
                        (Some((a, bc)), _) if bc > half => Some((a, bc, true)),
                        (_, Some((a, bc))) if bc > half => Some((a, bc, false)),
                        _ => None,
                    };
                    match chosen {
                        // Break at the candidate when it is far enough in that
                        // the head is not nearly empty.
                        Some((at, bcol, was_space)) => {
                            let tail = line.split_off(at);
                            let tail_cols = col - bcol;
                            // A break taken *at* a space consumes it, which is
                            // what prose wants and what leaves no trailing
                            // whitespace. It is safe for a value because the only
                            // space a rendered form contains is in its tag —
                            // `UC1 `, `UC1/5 ` — which sits before the halfway
                            // mark and is therefore never chosen here.
                            if was_space {
                                while line.ends_with(' ') {
                                    line.pop();
                                }
                            }
                            flush(&mut out, &mut line);
                            line.push_str(&tail);
                            col = indent + tail_cols;
                        }
                        _ => {
                            flush(&mut out, &mut line);
                            col = indent;
                        }
                    }
                    brk_space = None;
                    brk_sep = None;
                }
                line.push(*c);
                col += 1;
                if !c.is_alphanumeric() {
                    brk_sep = Some((line.len(), col));
                    if c.is_whitespace() {
                        brk_space = Some((line.len(), col));
                    }
                }
            }
        }
    }
    out.push_str(&line);
    out
}
