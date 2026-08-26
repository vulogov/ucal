//! §20 UC-P15: **one-screen demo; every entry cited and `Window`-valued;
//! `UCAL-W0006` where applicable.**

use ucal::{cmd_events_list, cmd_events_show, cmd_ruler, cmd_timeline};
use ucal_core::Tier;

const SI_EPOCH: &str = "8070204002895596515944343085635637180530466139316558837890625";
const NOW_ISH: &str = "8070205189123984864657505252035637180530466139316558837890625";

#[test]
fn the_timeline_spans_the_whole_of_time() {
    // §20 calls this the one-screen demo, and it was one screen when the
    // catalogue held eleven events. 1.4.0 doubled it, and twenty-two events with
    // their published figures and warnings is around a hundred and twenty lines.
    //
    // The fields were not trimmed to get back under a line count: `as_published`
    // is a `ucal-json/1` path and promise 4 forbids removing one because a
    // *terminal* rendering grew. So the demo is no longer one screen, and the
    // claim is what changed — `ucal events show <id>` is the per-event view and
    // `--tier` coarsens the placement.
    //
    // What the test asserts instead is the thing that actually mattered about
    // "one screen": that the whole of time is present in one document, from the
    // first entry to the last.
    let doc = cmd_timeline(Tier::DEEP).unwrap();
    let text = doc.to_text();

    assert!(text.contains("inflationary epoch"));
    assert!(text.contains("bridge epoch"));
    assert!(text.contains("Cretaceous"));
    // And, since 1.4.0, past the present: the upper half of the ladder held
    // nothing at all when every event was inside the first ten tiers.
    assert!(text.contains("the last black holes evaporate"));
    assert!(text.contains("big bang nucleosynthesis"));

    // Still bounded. A timeline that grows without limit is a list.
    let lines = text.lines().count();
    assert!(lines < 200, "the timeline has stopped being a timeline; {lines} lines");
}

#[test]
fn every_timeline_entry_renders_at_every_tier() {
    // The regression this guards: the human form cannot state a precision coarser
    // than T0 (D-A8), and a silent `unwrap_or_default()` turned that into a blank
    // column that read as a value.
    for tier in [Tier::DEEP, Tier::DRIFT, Tier::SPAN, Tier::SWEEP, Tier::BEAT] {
        let text = cmd_timeline(tier).unwrap().to_text();
        assert!(
            !text.contains("at                   \n"),  // ucal-lint-allow(no-indent-in-literal): a column header, aligned on purpose
            "empty position at {tier}"
        );
        assert!(!text.contains("<unrenderable"), "unrenderable at {tier}");
    }
}

#[test]
fn every_entry_is_cited_and_interval_valued() {
    let doc = cmd_events_list().unwrap();
    // `events` renders as a table, so it is a `Rows`. Read through the accessor
    // that does not care which: the choice is a rendering one.
    let Some(events) = doc.rows("events") else {
        panic!("expected an events section");
    };
    assert!(events.len() >= 10, "the catalogue must be substantial");
    for (id, v) in events {
        let Some(fields) = v.as_rows() else {
            panic!("{id} is not a section")
        };
        let keys: Vec<&str> = fields.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"citation"), "{id} has no citation");
        assert!(keys.contains(&"window_ticks"), "{id} is not window-valued");
        assert!(keys.contains(&"as_published"), "{id} has no published form");
    }
}

#[test]
fn w0006_appears_exactly_where_it_should() {
    // §10.6: inside the claim half-width, and nowhere else.
    let warned = cmd_events_show("recombination").unwrap().to_text();
    assert!(warned.contains("UCAL-W0006"));
    assert!(warned.contains("never an operand"), "Rule Q.3 must be restated");

    let inflation = cmd_events_show("inflation").unwrap().to_text();
    assert!(inflation.contains("UCAL-W0006"));

    for id in ["k-pg", "solar-system", "first-stars", "bridge-epoch"] {
        let text = cmd_events_show(id).unwrap().to_text();
        assert!(!text.contains("UCAL-W0006"), "{id} should not warn");
    }
}

#[test]
fn appendix_cs_recombination_fixture_lies_inside_the_catalogue_window() {
    // Appendix C places recombination at 380 kyr. The catalogue spans the whole
    // era, 240-430 kyr, because recombination is a process — so the RFC's own
    // fixture must fall inside it rather than equal it.
    let doc = cmd_events_show("recombination").unwrap();
    let text = doc.to_text();
    assert!(text.contains("240 to 430 kyr"));
    let appendix_c = ucal_core::Instant::<ucal_core::UC1>::from_ticks(
        <ucal_core::Ticks as ucal_core::TickInt>::from_dec_str(
            "222432546681680327568000000000000000000000000000000000000",
        )
        .unwrap(),
    )
    .unwrap();
    let e = ucal_events::by_id("recombination").unwrap();
    assert!(
        e.window.contains(&appendix_c),
        "Appendix C's 380 kyr fixture must lie inside the catalogue era"
    );
}

#[test]
fn a_window_is_reported_as_a_window_not_a_point() {
    // Rule U: no silent collapse. The midpoint is offered, and labelled as a
    // rendering choice rather than a measurement.
    let text = cmd_events_show("luca").unwrap().to_text();
    assert!(text.contains("width_ticks"));
    assert!(text.contains("rendering choice, not a measurement"));
}

#[test]
fn the_ruler_marks_the_grid() {
    let doc = cmd_ruler(SI_EPOCH, NOW_ISH, Tier::SPAN).unwrap();
    let text = doc.to_text();
    assert!(text.contains("whole_steps"));
    // Marks are on the tier grid, and named so any tier renders.
    assert!(text.contains("span"));
}

#[test]
fn the_ruler_reports_what_it_truncated() {
    // No silent caps: a bound that drops output must say so.
    let doc = cmd_ruler(SI_EPOCH, NOW_ISH, Tier::BEAT).unwrap();
    let text = doc.to_text();
    assert!(
        text.contains("the first 64 are shown"),
        "a truncated ruler must say what it dropped"
    );
    assert!(text.contains("coarser tier"));
}

#[test]
fn an_inverted_ruler_is_refused() {
    let e = cmd_ruler(NOW_ISH, SI_EPOCH, Tier::SPAN).unwrap_err();
    assert_eq!(e.code, ucal_core::Code::E0022);
}

#[test]
fn an_unknown_event_is_an_error() {
    assert!(cmd_events_show("nonexistent").is_err());
}

/// G10 — an event id used where an instant belongs says what it is.
///
/// The catalogue's ids are words no accepted instant form could be confused
/// with, and `ucal between recombination <B>` reported *malformed timestamp*
/// about a name this program knows perfectly well.
///
/// **It is a refusal, not a conversion.** An event is an interval:
/// `recombination`'s window is hundreds of thousands of years wide, and quietly
/// taking one end would be the substitution Rule U refuses while taking the
/// midpoint would be a rendering choice presented as a measurement.
#[test]
fn an_event_id_is_not_an_instant_and_says_so() {
    let e = ucal::parse_instant("recombination").expect_err("an event is an interval");
    assert_eq!(e.code, ucal_core::Code::E0023);
    let msg = e.to_string();
    assert!(msg.contains("an interval rather than an instant"), "{msg}");
    // And it names where to go instead.
    assert!(msg.contains("events show recombination"), "{msg}");
    assert!(msg.contains("wallclock --since"), "{msg}");
}

/// Every id in the catalogue is refused the same way, rather than a handful
/// being special-cased.
#[test]
fn the_whole_catalogue_is_refused_as_an_instant() {
    for e in ucal_events::all() {
        let err = ucal::parse_instant(e.id)
            .err()
            .unwrap_or_else(|| panic!("`{}` was accepted as an instant", e.id));
        assert_eq!(err.code, ucal_core::Code::E0023, "{}", e.id);
    }
}

/// **`wallclock --since` still takes one**, which is the point of the message.
///
/// That is the one place the collapse to a point is checked before it is made:
/// an event whose window is wider than the finest hand on the face is refused,
/// and `bridge-epoch` is exact by declaration.
#[test]
#[cfg(feature = "tui")]
fn the_one_command_that_resolves_an_event_still_does() {
    let (_, label) = ucal::wallclock_origin("bridge-epoch").expect("exact by declaration");
    assert!(label.contains("bridge-epoch"), "{label}");
    assert!(ucal::wallclock_origin("recombination").is_err());
}
