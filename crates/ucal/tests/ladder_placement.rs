//! Y1 — the ladder placement rows, and what holds them to the ladder.
//!
//! `cal show` prints where a body's own periods sit on the universal grid.
//! The number it prints is computed in the binary; the table W4 step 1 produced
//! is computed in `ucal-body`'s test. Two computations of the same quantity is
//! how a display drifts from what it displays, so this pins the first against
//! the second and against §4.3's published figure.

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

/// §4.3 publishes `1 d = 591.25 arc`. The row must say so.
///
/// Not 591.3 by coincidence of rounding: the row is rendered to one decimal and
/// the specification's figure is stated to two, so the test checks the render
/// the user sees against the digits the specification prints.
#[test]
fn earths_day_is_the_arc_count_the_spec_publishes() {
    let doc = ucal::cmd_cal_show("earth-d", T).expect("earth-d shows");
    let text = doc.to_text();
    let row = text
        .lines()
        .find(|l| l.contains("solar_day"))
        .unwrap_or_else(|| panic!("no solar_day row in:\n{text}"));
    assert!(
        row.contains("T1 arc"),
        "Earth's day should sit on T1, the arc: {row}"
    );
    assert!(
        row.contains("591.3"),
        "§4.3 publishes 1 d = 591.25 arc, which renders to 591.3: {row}"
    );
}

/// Earth and Mars land on the same rung, which is the finding worth printing.
///
/// A ladder whose steps are a factor of 3125 puts two independently measured
/// days — 591.3 arcs and 607.5 — inside one step of each other. If this ever
/// stops holding, the row has lost the reason it exists and should be removed
/// rather than quietly kept.
#[test]
fn earth_and_mars_print_the_same_rung() {
    let of = |id: &str| {
        let text = ucal::cmd_cal_show(id, T).expect("shows").to_text();
        text.lines()
            .find(|l| l.contains("solar_day"))
            .map(|l| l.to_string())
            .unwrap_or_else(|| panic!("no solar_day row for {id}"))
    };
    let earth = of("earth-d");
    let mars = of("mars-d");
    assert!(earth.contains("T1 arc") && mars.contains("T1 arc"), "{earth}\n{mars}");
    assert!(mars.contains("607.5"), "{mars}");
}

/// Every anchored calendar gets a placement for every period it has.
///
/// The failure this catches is a row that silently disappears: `ladder_row`
/// returns `None` on any arithmetic that will not close, and a `None` is a
/// missing line rather than an error. Nothing in the shipped catalogue may take
/// that path.
#[test]
fn every_anchored_calendar_places_its_day_and_year() {
    let mut seen = 0;
    for id in ucal_body::calendar::ids() {
        let Ok(doc) = ucal::cmd_cal_show(id, T) else {
            continue; // no anchor: `cal show` is UCAL-E0062, and that is Rule J.3
        };
        seen += 1;
        let json = doc.to_json();
        for unit in ["solar_day", "year"] {
            assert!(
                json.contains(&format!("\"{unit}\"")),
                "{id} has no {unit} placement:\n{json}"
            );
        }
        assert!(json.contains("ladder_placement"), "{id} has no placement rows");
        assert!(json.contains("above_rung"), "{id} has a rung with no residual");
    }
    assert!(seen >= 2, "only {seen} anchored calendars were reachable");
}
