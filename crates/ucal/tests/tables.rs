//! Tables, against the real commands rather than a fixture.
//!
//! Two claims are checked here, and the second is the one that constrains the
//! design:
//!
//! - no table line exceeds the width it was rendered at;
//! - no value is shortened to achieve that.
//!
//! The second is why columns are promoted rather than truncated. A tick count is
//! 61 digits and a base-5 form is over 200; an exact integer with an ellipsis in
//! it is not an exact integer, so a layout that cannot fit a value moves it, and
//! never edits it.
//!
//! Prose is in scope too, since a long field value hangs under its own label.
//! A terminal soft-wrapping a 225-character form back to column zero puts half
//! of it under the field names, where it reads as another row; hanging it under
//! its own column keeps it one value. Breaks prefer a word boundary, then a
//! separator, then nothing — so prose breaks between words and a base-5 form
//! breaks between groups.
//!
//! `--json` is untouched by any of it. Wrapping happens in the text renderer,
//! and a consumer still receives one string.

use ucal::emit::{Doc, Value};
use ucal::style::{strip_ansi, Render, Style};
use ucal::table::BASELINE_WIDTH;

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

/// Every command that renders a table, with the field that holds it.
fn tabular() -> Vec<(&'static str, &'static str, Doc)> {
    let mut v: Vec<(&'static str, &'static str, Doc)> = vec![
        (
            "ladder",
            "tiers",
            ucal::cmd_ladder(ucal_core::LocaleId::En, false).unwrap(),
        ),
        (
            "ladder ru",
            "tiers",
            ucal::cmd_ladder(ucal_core::LocaleId::Ru, false).unwrap(),
        ),
    ];
    #[cfg(feature = "events")]
    {
        v.push(("events list", "events", ucal::cmd_events_list().unwrap()));
        v.push((
            "timeline",
            "events",
            ucal::cmd_timeline(ucal::parse_tier("drift").unwrap()).unwrap(),
        ));
        v.push((
            "ruler",
            "marks",
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
        v.push(("cal list", "calendars", ucal::cmd_cal_list().unwrap()));
        v.push((
            "show",
            "calendars",
            ucal::cmd_show(T, &["earth-d".into(), "mars-d".into(), "earth-civil".into()]).unwrap(),
        ));
    }
    v
}

/// The lines a table produced: everything between its field header and the next
/// unindented line.
fn table_lines(doc: &Doc, field: &str, width: usize) -> Vec<String> {
    let text = doc.render(&Render::PLAIN.width(width));
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line == format!("{field}:") {
            inside = true;
            continue;
        }
        if inside {
            if !line.starts_with(' ') && !line.is_empty() {
                break;
            }
            out.push(line.to_string());
        }
    }
    out
}

#[test]
fn every_command_renders_its_field_as_a_table() {
    for (name, field, doc) in tabular() {
        let lines = table_lines(&doc, field, BASELINE_WIDTH);
        assert!(!lines.is_empty(), "`{name}`: no table produced for `{field}`");
        // A table has a rule line under its header.
        assert!(
            lines.iter().take(3).any(|l| l.contains('─')),
            "`{name}`: no header rule — rendered as nested sections?"
        );
    }
}

#[test]
fn no_table_line_exceeds_its_width() {
    for width in [BASELINE_WIDTH, 100, 132, 200] {
        for (name, field, doc) in tabular() {
            for line in table_lines(&doc, field, width) {
                let n = line.chars().count();
                assert!(
                    n <= width,
                    "`{name}` at {width}: {n}-character line\n{line}"
                );
            }
        }
    }
}

#[test]
fn no_value_is_shortened_to_make_it_fit() {
    // Rendered at the narrowest width, every exact value from the structured
    // data must still be findable in the text once layout whitespace is removed.
    // That is what makes "promoted, not truncated" a checked claim.
    for (name, field, doc) in tabular() {
        let text: String = doc
            .render(&Render::PLAIN.width(BASELINE_WIDTH))
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let rows = doc.rows(field).expect("a tabular field");
        for (key, row) in rows {
            let Some(fields) = row.as_rows() else {
                // A scalar row: the value itself must survive.
                if let Value::Form(f) | Value::Text(f) | Value::Number(f) = row {
                    let want: String = f.chars().filter(|c| !c.is_whitespace()).collect();
                    assert!(text.contains(&want), "`{name}`/{key}: value lost");
                }
                continue;
            };
            for (col, v) in fields {
                let Value::Number(n) = v else { continue };
                assert!(
                    text.contains(n.as_str()),
                    "`{name}`/{key}: column `{col}` was shortened or dropped\n\
                     wanted: {n}"
                );
            }
        }
    }
}

#[test]
fn no_output_contains_an_elision() {
    for (name, _, doc) in tabular() {
        let text = doc.render(&Render::PLAIN.width(BASELINE_WIDTH));
        assert!(!text.contains('…'), "`{name}`: an ellipsis reached the output");
        assert!(!text.contains("..."), "`{name}`: an elision reached the output");
    }
}

#[test]
fn a_wider_terminal_pulls_columns_into_the_grid() {
    // The promotion is a response to the width, not a fixed layout: at 200
    // columns the ladder's wide columns belong in the grid and the continuation
    // lines go away.
    let doc = ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap();
    let narrow = table_lines(&doc, "tiers", BASELINE_WIDTH).len();
    let wide = table_lines(&doc, "tiers", 200).len();
    assert!(
        wide < narrow,
        "a wider terminal produced no fewer lines ({wide} vs {narrow})"
    );
    // And the header names the column once it fits.
    assert!(table_lines(&doc, "tiers", 200)[0].contains("ticks"));
    assert!(!table_lines(&doc, "tiers", BASELINE_WIDTH)[0].contains("ticks"));
}

#[test]
fn the_width_floor_is_one_directional() {
    // Narrower than the baseline is refused, so a redirected stream and a small
    // terminal produce the same bytes as everything else.
    for w in [0, 1, 40, 79] {
        assert_eq!(Render::PLAIN.width(w).cols, BASELINE_WIDTH);
    }
    assert_eq!(Render::PLAIN.width(120).cols, 120);
    // Off a terminal, the baseline regardless of what a terminal would have said.
    assert_eq!(Render::resolve_width(None, None), BASELINE_WIDTH);
    assert_eq!(Render::resolve_width(None, Some(200)), 200);
    assert_eq!(Render::resolve_width(None, Some(40)), BASELINE_WIDTH);
    assert_eq!(Render::resolve_width(Some(150), Some(40)), 150);
    assert_eq!(Render::resolve_width(Some(10), None), BASELINE_WIDTH);
}

#[test]
fn colour_does_not_change_the_layout() {
    // The cells are measured on what a reader sees, so a coloured table lines up
    // exactly where a plain one does.
    for (name, field, doc) in tabular() {
        let plain = table_lines(&doc, field, BASELINE_WIDTH);
        let painted: Vec<String> = doc
            .render(&Render::styled(Style::colored()).width(BASELINE_WIDTH))
            .lines()
            .map(strip_ansi)
            .collect();
        for l in &plain {
            assert!(
                painted.iter().any(|p| p == l),
                "`{name}`: coloured layout differs from plain at\n{l}"
            );
        }
    }
}


// ------------------------------------------------- long values hang under their label

/// Documents whose fields include values too long for 80 columns.
fn long_valued() -> Vec<(&'static str, Doc)> {
    let mut v: Vec<(&'static str, Doc)> = vec![
        ("doctor", ucal::cmd_doctor().unwrap()),
        ("explain", ucal::cmd_explain(T, false).unwrap()),
        ("datum", ucal::cmd_datum().unwrap()),
    ];
    #[cfg(all(feature = "body", feature = "civil"))]
    v.push((
        "show",
        ucal::cmd_show(T, &["earth-d".into(), "mars-d".into()]).unwrap(),
    ));
    v
}

#[test]
fn a_long_value_hangs_under_its_own_column() {
    // Left alone the terminal wraps a 225-character form back to column zero, so
    // half of it lands under the field names and reads as another row.
    for (name, doc) in long_valued() {
        for line in doc.render(&Render::PLAIN.width(BASELINE_WIDTH)).lines() {
            assert!(
                line.chars().count() <= BASELINE_WIDTH,
                "`{name}`: {}-character line\n{line}",
                line.chars().count()
            );
        }
    }
}

#[test]
fn a_wrapped_value_is_recoverable() {
    // The property that makes wrapping safe for an exact quantity: rejoining the
    // continuation lines returns the value, character for character.
    for (name, doc) in long_valued() {
        let text = doc.render(&Render::PLAIN.width(BASELINE_WIDTH));
        let joined: String = text.lines().map(str::trim_start).collect();
        for (_, v) in doc.fields() {
            let want = match v {
                Value::Number(n) => n.clone(),
                Value::Form(f) => f.clone(),
                _ => continue,
            };
            assert!(
                joined.contains(&want),
                "`{name}`: a value did not survive wrapping\n  wanted: {want}"
            );
        }
    }
}

#[test]
fn no_wrapped_line_carries_trailing_whitespace() {
    for (name, doc) in long_valued() {
        for line in doc.render(&Render::PLAIN.width(BASELINE_WIDTH)).lines() {
            assert!(!line.ends_with(' '), "`{name}`: trailing space on\n{line:?}");
        }
    }
}

#[test]
fn wrapping_a_value_breaks_at_a_separator_not_mid_group() {
    // A base-5 form should break between groups, so a reader never has to
    // reassemble one across two lines.
    let doc = ucal::cmd_explain(T, false).unwrap();
    let text = doc.render(&Render::PLAIN.width(BASELINE_WIDTH));
    let wrapped: Vec<&str> = text
        .lines()
        .filter(|l| l.trim_start().starts_with("00000.") || l.contains("UC1/5"))
        .collect();
    assert!(wrapped.len() > 1, "digit5 did not wrap at 80 columns");
    for l in &wrapped[..wrapped.len() - 1] {
        assert!(
            l.ends_with('.'),
            "a base-5 line broke inside a group:\n{l}"
        );
    }
}

#[test]
fn colour_wraps_in_exactly_the_same_places() {
    // Break positions are computed from visible characters only, so the coloured
    // rendering breaks where the plain one does — which is what keeps the strip
    // invariant true across wrapping.
    for (name, doc) in long_valued() {
        let plain = doc.render(&Render::PLAIN.width(BASELINE_WIDTH));
        let painted = doc.render(&Render::styled(Style::colored()).width(BASELINE_WIDTH));
        assert_eq!(strip_ansi(&painted), plain, "`{name}`: wrapping differs with colour");
    }
}
