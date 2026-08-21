//! §15.1's *other* file — the anchor — loaded, in the binary and only there.
//!
//! # Why anchors are a separate file
//!
//! §15.1 requires body files and anchor files to be separate and to version
//! independently, and gives the reason: **parameters change with better
//! measurement, anchors with re-determination.** Those are different events on
//! different schedules. A new rotation period does not invalidate a phase, and a
//! re-determined phase does not change a rotation period. One file would tie
//! them together and force a revision of one to look like a revision of both.
//!
//! `Anchor` carries a `revision` for exactly this reason, so the two-file split
//! is not a filing convention: it is what makes the revision number mean
//! something.
//!
//! # What a file cannot be used to do
//!
//! **Invent a phase.** This is the whole risk of the feature, and it is why
//! [`X1.3`] listed it as a kill criterion rather than a detail:
//!
//! > Loading must not become a way to invent an anchor. GE-3's kill criterion
//! > forbids narrowing a window by assumption, and a file is a much easier place
//! > to do it than a Rust constant.
//!
//! Rule J makes phase **empirical**: determined and cited, never derived and
//! never borrowed. Two of twelve shipped calendars have an anchor, and
//! [`D5-titan-anchor.md`] recorded what it costs to establish a third honestly —
//! Titan's rotational elements are published and its mean-solar-time convention
//! is not, so the honest answer was no anchor.
//!
//! The defence is that this loader adds no check of its own. Every obligation is
//! `Anchor::new`'s, which the compiled-in anchors go through too:
//!
//! - the phase must be a **physical event of the body** — `UCAL-E0063` if the
//!   definition names a foreign epoch, clock or calendar (Rule J.1);
//! - the uncertainty window must **contain the anchor's own tick** —
//!   `UCAL-E0062`, because a window excluding its own best estimate is a
//!   contradiction rather than an uncertainty (Rule J.2);
//! - the determination must state a **method**, a **citation** and what
//!   **dominates the uncertainty**, because `Determination`'s fields are not
//!   optional.
//!
//! So a file cannot reach a state a Rust constant could not. What it can do is
//! reach that state without editing this crate, which is the point.
//!
//! # The one thing a file must do that a constant need not
//!
//! State the window explicitly. A compiled-in anchor derives its window from a
//! stated uncertainty in the same expression that builds the tick; a file states
//! both, and nothing in the format computes one from the other. That is
//! deliberate — a loader that widened or narrowed a window by any rule of its own
//! would be doing the thing GE-3 forbids.
//!
//! [`X1.3`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/X1-authoring-local-calendars.md
//! [`D5-titan-anchor.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/D5-titan-anchor.md

use serde::Deserialize;
use ucal_body::anchor::{Anchor, Determination, Meridian, PhaseDefinition};
use ucal_core::backend::TickInt;
use ucal_core::{Citation, Code, Instant, Ticks, TimeError, Window, UC1};

type Result<T> = core::result::Result<T, TimeError>;

use crate::body_file::leak;

/// How the phase is defined (Rule J.1).
///
/// The named variants cannot express a foreign reference at all, which is the
/// structural half of Rule J.1's defence. `custom` can, and is word-screened by
/// `PhaseDefinition::check_is_a_body_event` — a partial defence that this crate
/// describes as one.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PhaseFile {
    /// `mean_solar_midnight`, `northward_equinox`, `southward_equinox`,
    /// `perihelion`, or `custom`.
    kind: String,
    /// The meridian, for `mean_solar_midnight`. Required for that kind only.
    #[serde(default)]
    meridian: Option<String>,
    /// Where the meridian's definition comes from.
    #[serde(default)]
    meridian_citation: Option<String>,
    /// What the event is, for `custom`.
    #[serde(default)]
    description: Option<String>,
    /// Where a custom definition comes from. D-15 requires it.
    #[serde(default)]
    citation: Option<String>,
    #[serde(default)]
    locator: Option<String>,
}

/// How the instant was obtained (Rule J.3).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeterminationFile {
    /// How the instant was obtained, in a sentence.
    method: String,
    /// The source of the observation or ephemeris.
    citation: String,
    #[serde(default)]
    locator: Option<String>,
    /// What dominates the uncertainty, and why the window is the width it is.
    uncertainty_note: String,
}

/// An anchor file (§15.1).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AnchorFile {
    /// Which calendar this anchors, e.g. `europa-d`.
    calendar_id: String,
    /// The anchor instant, in ticks.
    tick: String,
    /// The uncertainty window, in ticks, stated and never computed.
    window_lo: String,
    window_hi: String,
    phase: PhaseFile,
    determination: DeterminationFile,
    /// Anchors are versioned because they are observations.
    revision: u32,
}

fn malformed(what: &str) -> TimeError {
    TimeError::with_context(Code::E0017, leak(format!("the anchor file {what}")))
}

fn tick_of(s: &str, what: &str) -> Result<Instant<UC1>> {
    let t = <Ticks as TickInt>::from_dec_str(s.trim())
        .ok_or_else(|| malformed(&format!("has a {what} that is not a decimal tick count")))?;
    Instant::from_ticks(t)
}

impl PhaseFile {
    fn build(self) -> Result<PhaseDefinition> {
        let cite = |c: Option<String>, l: Option<String>, what: &str| -> Result<Citation> {
            let c = c.ok_or_else(|| malformed(&format!("needs a {what}")))?;
            Ok(Citation::new(leak(c), l.map(leak)))
        };
        match self.kind.as_str() {
            "mean_solar_midnight" => {
                let name = self
                    .meridian
                    .ok_or_else(|| malformed("needs a `meridian` for this phase kind"))?;
                Ok(PhaseDefinition::MeanSolarMidnight {
                    meridian: Meridian::new(
                        leak(name),
                        cite(
                            self.meridian_citation,
                            self.locator,
                            "meridian_citation: a prime meridian is a convention and has a source",
                        )?,
                    ),
                })
            }
            "northward_equinox" => Ok(PhaseDefinition::NorthwardEquinox),
            "southward_equinox" => Ok(PhaseDefinition::SouthwardEquinox),
            "perihelion" => Ok(PhaseDefinition::Perihelion),
            "custom" => {
                let description = self
                    .description
                    .ok_or_else(|| malformed("needs a `description` for a custom phase"))?;
                Ok(PhaseDefinition::Custom {
                    description: leak(description),
                    // D-15: a custom phase cannot be asserted without a source.
                    citation: cite(self.citation, self.locator, "citation for a custom phase")?,
                })
            }
            _ => Err(TimeError::with_context(
                Code::E0018,
                "a phase kind must be one of `mean_solar_midnight`, `northward_equinox`, \
                 `southward_equinox`, `perihelion` or `custom`",
            )),
        }
    }
}

/// Read an anchor file and build the [`Anchor`] it declares.
///
/// Every refusal below `E0017` comes from `Anchor::new`, not from here.
pub fn load(path: &std::path::Path) -> Result<Anchor> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        TimeError::with_context(
            Code::E0017,
            match e.kind() {
                std::io::ErrorKind::NotFound => "no such anchor file",
                _ => "the anchor file could not be read",
            },
        )
    })?;

    let file: AnchorFile = deser_hjson::from_str(&text).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("unknown field") {
            TimeError::with_context(
                Code::E0012,
                "unknown key in the anchor file; the accepted keys are calendar_id, tick, \
                 window_lo, window_hi, phase, determination and revision",
            )
        } else if msg.contains("missing field") {
            TimeError::with_context(
                Code::E0060,
                "an anchor file must give calendar_id, tick, window_lo, window_hi, revision, \
                 a phase and a determination; the determination needs a method, a citation \
                 and an uncertainty_note, because Rule J.3 makes all three obligations",
            )
        } else {
            TimeError::with_context(Code::E0017, "the anchor file is not well-formed HJSON")
        }
    })?;

    let window = Window::new(
        tick_of(&file.window_lo, "window_lo")?,
        tick_of(&file.window_hi, "window_hi")?,
    )?;

    // The anchor's citation is the determination's. They are separate arguments
    // to `Anchor::new` and the shipped anchors pass the same value to both — an
    // anchor is cited to the observation that determined it, and a file that
    // could name a different source for the anchor than for its determination
    // would be offering a way to cite something other than the work.
    let citation = Citation::new(
        leak(file.determination.citation),
        file.determination.locator.map(leak),
    );

    Anchor::new(
        leak(file.calendar_id),
        tick_of(&file.tick, "tick")?,
        file.phase.build()?,
        Determination::new(
            leak(file.determination.method),
            citation,
            leak(file.determination.uncertainty_note),
        ),
        window,
        citation,
        file.revision,
    )
}
