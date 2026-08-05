//! The one property that makes colour safe here.
//!
//! ```text
//! strip_ansi(doc.to_ansi(&style)) == doc.to_text()      // byte for byte
//! ```
//!
//! A style may add SGR sequences. It may never change a character. That is what
//! guarantees anything a reader learns from colour is learnable without it — down
//! a pipe, in a log, on a terminal with no colour at all — and it is checked
//! rather than promised for the same reason `SignedWindow` has no operators.
//!
//! The documents below are the real ones. A synthetic `Doc` would test the
//! renderer against a shape no command emits, which is the failure this file is
//! meant to catch.

use ucal::emit::{Doc, Value};
use ucal::style::{group_decimal, paint_form, strip_ansi, ColorChoice, Render, Role, Style};

/// Every document the shipped commands can produce, without reading the clock.
///
/// `now` is deliberately absent: it varies between runs, and a test that has to
/// be re-read to know whether it failed is not a test. Everything it renders is
/// covered by `explain` on a fixed instant.
fn documents() -> Vec<(&'static str, Doc)> {
    // A fixed instant, so this file says the same thing on every run.
    const T: &str = "8070205189123984864657505252035637180530466139316558837890625";
    const T2: &str = "8070205189999984864657505252035637180530466139316558837890625";

    let mut v: Vec<(&'static str, Doc)> = vec![
        ("datum", ucal::cmd_datum().unwrap()),
        ("doctor", ucal::cmd_doctor().unwrap()),
        ("explain", ucal::cmd_explain(T, false).unwrap()),
("between", ucal::cmd_between(T, T2, Some(ucal_core::Tier::BEAT)).unwrap()),
        ("verify", ucal::cmd_verify().unwrap()),
        ("explain --claim", ucal::cmd_explain(T, true).unwrap()),
        (
            "ladder",
            ucal::cmd_ladder(ucal_core::LocaleId::En, false).unwrap(),
        ),
        (
            "ladder --named-only",
            ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap(),
        ),
        (
            "ladder ru",
            ucal::cmd_ladder(ucal_core::LocaleId::Ru, true).unwrap(),
        ),
    ];

    #[cfg(feature = "events")]
    {
        v.push(("events list", ucal::cmd_events_list().unwrap()));
        v.push((
            "events show",
            ucal::cmd_events_show("recombination").unwrap(),
        ));
        // The one that carries UCAL-W0006, so the warning role is exercised.
        v.push(("events show inflation", ucal::cmd_events_show("inflation").unwrap()));
        v.push((
            "timeline",
            ucal::cmd_timeline(ucal::parse_tier("drift").unwrap()).unwrap(),
        ));
    }

    #[cfg(all(feature = "body", feature = "civil"))]
    {
        v.push(("cal list", ucal::cmd_cal_list().unwrap()));
        v.push(("cal show", ucal::cmd_cal_show("earth-d", T).unwrap()));
        v.push(("cal anchor", ucal::cmd_cal_anchor("earth-d").unwrap()));
        v.push((
            "show",
            ucal::cmd_show(T, &["earth-d".into(), "mars-d".into(), "earth-civil".into()]).unwrap(),
        ));
    }

    #[cfg(feature = "cosmo")]
    {
        v.push(("cosmo model", ucal::cmd_cosmo_model().unwrap()));
        v.push(("cosmo age", ucal::cmd_cosmo_age("1100", 6, 8).unwrap()));
    }

    v
}

/// Every style that can reach a terminal.
fn styles() -> Vec<(&'static str, Style)> {
    vec![
        ("plain", Style::PLAIN),
        ("colored", Style::colored()),
        ("default", Style::default()),
    ]
}

#[test]
fn colour_never_changes_a_character() {
    // Held at every separator setting, not only the default. The claim is that
    // *colour* adds no character, so the comparison is against the same document
    // rendered plainly with the same separator — widening it to ignore the
    // separator too would make the invariant vacuous.
    for (name, doc) in documents() {
        for sep in [None, Some('_'), Some(' ')] {
            let plain = doc.render(&Render::PLAIN.group(sep));
            for (sname, style) in styles() {
                let painted = doc.render(&Render::styled(style).group(sep));
                assert_eq!(
                    strip_ansi(&painted),
                    plain,
                    "`{name}` style `{sname}` sep {sep:?}: colour altered the text"
                );
            }
        }
    }
}

#[test]
fn the_separator_changes_grouping_and_layout_and_nothing_else() {
    // Weaker than the first draft, and deliberately. A separator widens a cell,
    // so a table's column widths and padding move with it — that is the layout
    // doing its job, not a defect. What must not move is the content: with
    // separators and whitespace removed, the two renderings are the same
    // characters in the same order, so no digit was altered, dropped or added.
    for (name, doc) in documents() {
        let strip = |s: &str, sep: char| -> String {
            s.chars()
                .filter(|c| *c != sep && !c.is_whitespace())
                .collect()
        };
        for sep in ['_', '·'] {
            let bare = strip(&doc.to_text(), sep);
            let grouped = strip(&doc.render(&Render::PLAIN.group(Some(sep))), sep);
            assert_eq!(
                grouped, bare,
                "`{name}`: separator `{sep}` changed content, not only grouping"
            );
        }
    }
}

#[test]
fn a_grouped_number_still_reassembles() {
    // The narrow property the one above gave up, checked where it actually
    // holds: on the value itself rather than on a laid-out document.
    for sep in ['_', ' ', '·'] {
        let r = Render::PLAIN.group(Some(sep));
        let n = "8070205189123984864657505252035637180530466139316558837890625";
        assert_eq!(group_decimal(&r, n).replace(sep, ""), n);
    }
}

#[test]
fn grouping_preserves_the_integer() {
    let r = Render::PLAIN.group(Some('_'));
    for n in [
        "0",
        "5",
        "42",
        "999",
        "1000",
        "-1000",
        "8070205189123984864657505252035637180530466139316558837890625",
        "-318856914364362819469533860683441162109375",
    ] {
        let g = group_decimal(&r, n);
        assert_eq!(g.replace('_', ""), n, "grouping changed {n}");
    }
    // Under three digits there is nothing to group and no separator appears.
    assert_eq!(group_decimal(&r, "42"), "42");
    assert_eq!(group_decimal(&r, "999"), "999");
    assert_eq!(group_decimal(&r, "1000"), "1_000");
    // The leading group is the short one, so groups align from the right.
    assert_eq!(group_decimal(&r, "12345"), "12_345");
    assert_eq!(group_decimal(&r, "123456"), "123_456");
    assert_eq!(group_decimal(&r, "1234567"), "1_234_567");
    assert_eq!(group_decimal(&r, "-1234567"), "-1_234_567");
}

#[test]
fn grouping_declines_what_it_does_not_understand() {
    // A renderer that guesses at a format is how an exact integer stops being
    // one. Anything that is not a plain signed integer passes through whole.
    let r = Render::PLAIN.group(Some('_'));
    for s in ["", "-", "12.34", "1e9", "0x1f", "12 34", "[1000, 2000]", "abc"] {
        assert_eq!(group_decimal(&r, s), s, "mangled {s:?}");
    }
}

#[test]
fn the_default_render_inserts_no_separator() {
    // The default has to stay paste-safe: a tick count copied out of this output
    // must still be an integer.
    let out = Render::PLAIN;
    assert_eq!(out.group, None);
    assert_eq!(
        group_decimal(&out, "8070205189123984864657505252035637180530466139316558837890625"),
        "8070205189123984864657505252035637180530466139316558837890625"
    );
}

#[test]
fn the_plain_style_emits_no_escape_sequences_at_all() {
    // Stripping equality would also hold for a style that emitted a sequence and
    // then a matching reset around nothing. The plain path must be byte-identical
    // to what it was before colour existed, not merely equivalent after stripping.
    for (name, doc) in documents() {
        let plain = doc.render(&Render::PLAIN);
        assert!(
            !plain.contains('\u{1b}'),
            "`{name}`: the plain style emitted an escape sequence"
        );
        assert_eq!(plain, doc.to_text(), "`{name}`: to_text is not the plain case");
    }
}

#[test]
fn the_coloured_rendering_actually_differs() {
    // Without this, every assertion above would pass against a `colored()` that
    // silently did nothing, and the whole layer would be untested.
    let differed = documents()
        .iter()
        .filter(|(_, d)| d.to_ansi(&Style::colored()) != d.to_text())
        .count();
    assert_eq!(
        differed,
        documents().len(),
        "some documents render identically coloured and plain"
    );
}

#[test]
fn json_is_never_coloured() {
    // §19.1 makes --json a stable contract for a program. An SGR sequence in it
    // is a defect regardless of what was asked for or what is attached.
    for (name, doc) in documents() {
        assert!(
            !doc.to_json().contains('\u{1b}'),
            "`{name}`: escape sequence in JSON"
        );
    }
    for choice in [ColorChoice::Auto, ColorChoice::Always, ColorChoice::Never] {
        assert!(
            ucal::style::resolve_for_output(choice, true).is_plain(),
            "{choice:?} produced colour for JSON output"
        );
    }
}

#[test]
fn alignment_is_computed_before_painting() {
    // The classic defect: padding an already-painted string pads to the width of
    // the escape sequences, so a coloured column no longer lines up. Stripping
    // both and comparing line for line is what catches it.
    let doc = Doc::new()
        .title("t")
        .field("a", Value::text("1"))
        .field("longer-key", Value::number("2"))
        .field("mid", Value::Bool(true));
    let plain: Vec<String> = doc.to_text().lines().map(str::to_string).collect();
    let painted: Vec<String> = doc
        .to_ansi(&Style::colored())
        .lines()
        .map(|l| strip_ansi(l))
        .collect();
    assert_eq!(plain, painted);
    // And the columns really are aligned, so the test above is testing something.
    assert!(plain.iter().any(|l| l.starts_with("a           1"))); // ucal-lint-allow(no-indent-in-literal): the padding is the assertion
}

#[test]
fn diagnostic_codes_take_their_own_roles() {
    let warn = Doc::new().note("UCAL-W0006: inside the claim half-width");
    let err = Doc::new().note("UCAL-E0071: tolerance unreachable");
    let plain_note = Doc::new().note("the beat is the universe second");
    let s = Style::colored();

    let w = warn.to_ansi(&s);
    let e = err.to_ansi(&s);
    let n = plain_note.to_ansi(&s);
    assert!(w.contains(&s.get(Role::Warning).to_string()), "warning uncoloured");
    assert!(e.contains(&s.get(Role::Error).to_string()), "error uncoloured");
    assert!(
        !n.contains(&s.get(Role::Warning).to_string()),
        "an ordinary note was coloured as a warning"
    );
    // All three still strip back.
    for d in [&warn, &err, &plain_note] {
        assert_eq!(strip_ansi(&d.to_ansi(&s)), d.to_text());
    }
}

// ------------------------------------------------------------------- forms

#[test]
fn a_form_keeps_every_character() {
    let r = Render::styled(Style::colored());
    for f in [
        "UC1 0031·0687·2481·3000·2434·1316:0750·0016",
        "UC1/5 00000.00000.00111.10222",
        "0000000000050PM6K45P2JZZTJ587Q9TBQSDGZFKF0T83MAJ9FJ1",
        "— (outside 2^256, UCAL-E0031)",
        "",
    ] {
        assert_eq!(strip_ansi(&paint_form(&r, f)), f, "form altered: {f:?}");
    }
}

#[test]
fn the_leading_zero_run_is_one_region_not_one_per_group() {
    // 27 dimmed groups separated by 26 dots is 53 alternations if a separator
    // inside the run takes its own role. That reads as stripes and emits a
    // sequence pair per group, so the run has to paint as one region.
    //
    // Asserted on the painted substring rather than by counting sequences: the
    // shipped scheme gives `Padding` and `Separator` the same appearance, so a
    // count cannot tell which role produced what.
    let r = Render::styled(Style::colored());
    let f = "UC1/5 00000.00000.00000.00000.00111.10222";
    let painted = paint_form(&r, f);
    let dim = Style::colored().get(Role::Padding);
    let run = format!("{dim}00000.00000.00000.00000.00{dim:#}");
    assert!(
        painted.contains(&run),
        "the leading run did not paint as a single region:\n{painted:?}"
    );
    assert_eq!(strip_ansi(&painted), f);
}

#[test]
fn the_run_ends_at_the_first_significant_digit_not_at_a_group_boundary() {
    // `00111` is two leading zeros and three digits. Rounding the boundary out
    // to the group would dim a `1`, which is a measured digit.
    let r = Render::styled(Style::colored());
    let painted = paint_form(&r, "UC1/5 00000.00111");
    let dim = Style::colored().get(Role::Padding);
    assert!(painted.contains(&format!("{dim}00000.00{dim:#}")));
    assert!(!painted.contains(&format!("{dim}00000.00111{dim:#}")));
}

#[test]
fn a_form_with_no_leading_zeros_dims_nothing() {
    // A UCID has no separators, so nothing but `Padding` can produce a dim
    // sequence here — which is what makes the assertion mean what it says.
    let r = Render::styled(Style::colored());
    let painted = paint_form(&r, "50PM6K45P2JZZTJ587Q9TBQSDGZFKF0T83MAJ9FJ1");
    let dim = Style::colored().get(Role::Padding).to_string();
    assert!(!painted.contains(&dim), "dimmed a value with no leading zeros");
}

#[test]
fn a_ucids_leading_zeros_are_one_dimmed_region() {
    // The real shape: a UCID at the present epoch spends its first ten
    // characters on domain nobody has reached.
    let r = Render::styled(Style::colored());
    let f = "0000000000050PM6K45P2JZZTJ587Q9TBQSDGZFKF0T83MAJ9FJ1";
    let painted = paint_form(&r, f);
    let dim = Style::colored().get(Role::Padding);
    assert!(painted.contains(&format!("{dim}00000000000{dim:#}")));
    assert_eq!(painted.matches(&dim.to_string()).count(), 1);
    assert_eq!(strip_ansi(&painted), f);
}

#[test]
fn an_all_zero_form_is_entirely_padding() {
    // Tick 0 itself: every digit is a leading zero and none is a measurement.
    let r = Render::styled(Style::colored());
    let f = "UC1/5 00000.00000";
    let painted = paint_form(&r, f);
    assert_eq!(strip_ansi(&painted), f);
    let dim = Style::colored().get(Role::Padding);
    assert!(painted.contains(&format!("{dim}00000.00000{dim:#}")));
}

#[test]
fn form_values_are_plain_strings_in_json() {
    // The variant exists so the text renderer can see structure. If it changed
    // the JSON it would be a breaking change to `ucal-json/1`, which is exactly
    // what U2 and U3 were scoped to avoid.
    let t = Doc::new().field("x", Value::text("UC1 0000·0001"));
    let f = Doc::new().field("x", Value::form("UC1 0000·0001"));
    assert_eq!(t.to_json(), f.to_json());
}
