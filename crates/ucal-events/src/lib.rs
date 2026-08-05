//! # ucal-events — cited, interval-valued milestones
//!
//! §17: every entry carries an id, a label, a `Window<UC1>`, a source citation,
//! and a note where the window falls inside `BIG_BANG_CLAIM` (§10.6).
//!
//! # Why every entry is a window
//!
//! Not one of these is known to a tick. Some are not known to a gigayear. A
//! catalogue of point values would be a catalogue of false precision, and the
//! whole apparatus of Rule T and Rule U exists so that it does not have to be
//! one. `Window` is the honest type, and the widths below are the published
//! uncertainties rather than a house style.
//!
//! # Why this is a separate crate
//!
//! D-7: "Citations get revised more often than the library." A new measurement of
//! recombination should bump this crate and nothing else.
//!
//! # The first twenty million years
//!
//! §10.6 requires `UCAL-W0006` for any statement about the first
//! `BIG_BANG_CLAIM` half-width of absolute time. That is 1.1707×10⁵⁸ ticks —
//! about 20 Myr, or 141 drifts. Inside it, the datum's own physical
//! identification is comparable to or larger than the quantity being discussed,
//! so an event "at 380 000 years" is being placed relative to a zero that is
//! itself uncertain by sixty times that interval.
//!
//! The arithmetic is untouched by this. `BIG_BANG_CLAIM` remains a non-operand
//! (Rule Q.3); the warning is emitted alongside, and nothing consumes it.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::vec::Vec;

use ucal_core::backend::TickInt;
use ucal_core::{Citation, Code, Instant, Profile, Ticks, TimeError, Warning, Window, UC1};

/// The citation set this catalogue is versioned by (D-7).
///
/// Bumped when a source is revised, independently of the library's own version.
pub const CITATION_SET: &str = "ucal-events/2026-07 (Planck 2018; IUGS 2023; \
                               Bouvier & Wadhwa 2010; Betts et al. 2018)";

/// Which side of the datum an event's window is stated from.
///
/// Both are absolute tick windows in the end. The distinction is recorded because
/// it is how the *sources* state them, and Rule Y's principle — keep the
/// published form — applies to a catalogue as much as to a parameter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum StatedAs {
    /// Published as a time *after* the FLRW t→0 limit, e.g. "380 000 years".
    AfterDatum,
    /// Published as a time *before* the present, e.g. "66 Ma ago". Converted
    /// against `SI_EPOCH`, which is a fixed reference; "ago" from a moving now
    /// would make the catalogue non-reproducible.
    BeforeBridgeEpoch,
}

/// One catalogued milestone (§17).
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Event {
    /// Stable id, e.g. `"recombination"`.
    pub id: &'static str,
    /// English label. Localisation is a locale-table concern (Rule N).
    pub label: &'static str,
    /// One sentence on what it is.
    pub description: &'static str,
    /// The interval, in absolute time.
    pub window: Window<UC1>,
    /// How the source states it.
    pub stated_as: StatedAs,
    /// The published figure, verbatim.
    pub as_published: &'static str,
    /// Where it comes from.
    pub citation: Citation,
}

impl Event {
    /// Whether this event's window reaches into the `BIG_BANG_CLAIM` half-width,
    /// and so requires `UCAL-W0006` (§10.6).
    pub fn touches_claim_window(&self) -> bool {
        let half = UC1::big_bang_claim()
            .hi()
            .magnitude()
            .ticks()
            .clone();
        // The warning is about the *statement*, so it applies if any part of the
        // interval lies inside the half-width.
        self.window.lo().ticks() < &half
    }

    /// The warning that accompanies this entry, if any (§10.6).
    pub fn warning(&self) -> Option<Warning> {
        if self.touches_claim_window() {
            Some(Warning::W0006)
        } else {
            None
        }
    }

    /// The width of the window, in ticks.
    pub fn uncertainty(&self) -> ucal_core::Delta {
        self.window.width()
    }
}

fn at(decimal: &str) -> Instant<UC1> {
    Instant::from_ticks(
        <Ticks as TickInt>::from_dec_str(decimal).expect("catalogue value within the domain"),
    )
    .expect("catalogue value within the domain")
}

fn window(lo: &str, hi: &str) -> Window<UC1> {
    Window::new(at(lo), at(hi)).expect("catalogue windows are ordered")
}

const PLANCK_2018: Citation = Citation::new(
        "Planck 2018 results VI: Cosmological parameters, A&A 641, A6 (2020)",
        Some("doi:10.1051/0004-6361/201833910"),
    );
const GUTH_1981: Citation = Citation::new(
        "Guth, A. (1981), Inflationary universe: a possible solution to the \
             horizon and flatness problems, Phys. Rev. D 23, 347",
        Some("doi:10.1103/PhysRevD.23.347"),
    );
const BROMM_2011: Citation = Citation::new(
        "Bromm, V. and Yoshida, N. (2011), The first galaxies, \
             Ann. Rev. Astron. Astrophys. 49, 373",
        Some("doi:10.1146/annurev-astro-081710-102608"),
    );
const BOUVIER_2010: Citation = Citation::new(
        "Bouvier, A. and Wadhwa, M. (2010), The age of the Solar System \
             redefined by the oldest Pb-Pb age of a meteoritic inclusion, \
             Nature Geoscience 3, 637",
        Some("doi:10.1038/ngeo941"),
    );
const BETTS_2018: Citation = Citation::new(
        "Betts, H. C. et al. (2018), Integrated genomic and fossil evidence \
             illuminates life's early evolution and eukaryote origin, \
             Nature Ecology & Evolution 2, 1556",
        Some("doi:10.1038/s41559-018-0644-x"),
    );
const IUGS: Citation = Citation::new(
        "International Commission on Stratigraphy, International \
             Chronostratigraphic Chart v2023/09",
        Some("https://stratigraphy.org/chart"),
    );

/// The catalogue (§17).
pub fn all() -> Vec<Event> {
    alloc::vec![
        Event {
            id: "inflation",
            label: "inflationary epoch",
            description: "the hypothesised exponential expansion; under inflation \
                          the FLRW t→0 limit is not a physical event at all, which \
                          is one of Rule Q.2's three reasons the datum must be \
                          stipulated",
            window: window("18548584", "185485843998"),
            stated_as: StatedAs::AfterDatum,
            as_published: "10^-36 to 10^-32 s",
            citation: GUTH_1981,
        },
        Event {
            id: "recombination",
            label: "recombination",
            description: "electrons and protons combine and the universe becomes \
                          transparent. A process, not an instant: it spans roughly \
                          z = 1400 to z = 1000. Planck 2018 quotes last scattering \
                          at z_* = 1089.92, t_* = 372.6 kyr; the classic textbook \
                          figure of 380 kyr names the same era less precisely",
            window: window(
                "140483713693692838464000000000000000000000000000000000000",
                "251699987034533002248000000000000000000000000000000000000",
            ),
            stated_as: StatedAs::AfterDatum,
            as_published: "240 to 430 kyr (z = 1400 to z = 1000)",
            citation: PLANCK_2018,
        },
        Event {
            id: "first-stars",
            label: "first stars",
            description: "Population III star formation begins",
            window: window(
                "58534880705705349360000000000000000000000000000000000000000",
                "234139522822821397440000000000000000000000000000000000000000",
            ),
            stated_as: StatedAs::AfterDatum,
            as_published: "100 to 400 Myr",
            citation: BROMM_2011,
        },
        Event {
            id: "reionization",
            label: "reionization",
            description: "the intergalactic medium is reionised by the first \
                          luminous sources",
            window: window(
                "87802321058558024040000000000000000000000000000000000000000",
                "585348807057053493600000000000000000000000000000000000000000",
            ),
            stated_as: StatedAs::AfterDatum,
            as_published: "150 Myr to 1 Gyr",
            citation: PLANCK_2018,
        },
        Event {
            id: "galaxy-formation",
            label: "first galaxies",
            description: "the earliest galaxies assemble",
            window: window(
                "234139522822821397440000000000000000000000000000000000000000",
                "585348807057053493600000000000000000000000000000000000000000",
            ),
            stated_as: StatedAs::AfterDatum,
            as_published: "400 Myr to 1 Gyr",
            citation: BROMM_2011,
        },
        Event {
            id: "solar-system",
            label: "Solar System formation",
            description: "condensation of the first calcium-aluminium-rich \
                          inclusions, the oldest dated Solar System solids",
            window: window(
                "5394574605837804996698743085635637180530466139316558837890625",
                "5396916001066033210673143085635637180530466139316558837890625",
            ),
            stated_as: StatedAs::BeforeBridgeEpoch,
            as_published: "4567 to 4571 Ma ago",
            citation: BOUVIER_2010,
        },
        Event {
            id: "luca",
            label: "last universal common ancestor",
            description: "the most recent organism from which all present life \
                          descends",
            window: window(
                "5845878536078793240264343085635637180530466139316558837890625",
                "6021483178195909288344343085635637180530466139316558837890625",
            ),
            stated_as: StatedAs::BeforeBridgeEpoch,
            as_published: "3500 to 3800 Ma ago",
            citation: BETTS_2018,
        },
        Event {
            id: "cambrian",
            label: "base of the Cambrian",
            description: "the beginning of the Cambrian period, and of abundant \
                          animal fossils",
            window: window(
                "7754700995891844682893943085635637180530466139316558837890625",
                "7754818065653256093592663085635637180530466139316558837890625",
            ),
            stated_as: StatedAs::BeforeBridgeEpoch,
            as_published: "538.8 ± 0.1 Ma ago",
            citation: IUGS,
        },
        Event {
            id: "k-pg",
            label: "Cretaceous-Palaeogene boundary",
            description: "the impact and mass extinction that ends the Cretaceous",
            window: window(
                "8031512446749125280017383085635637180530466139316558837890625",
                "8031570981629830985366743085635637180530466139316558837890625",
            ),
            stated_as: StatedAs::BeforeBridgeEpoch,
            as_published: "66.0 to 66.1 Ma ago",
            citation: IUGS,
        },
        Event {
            id: "hominin-divergence",
            label: "hominin-chimpanzee divergence",
            description: "the split between the lineages leading to humans and to \
                          chimpanzees",
            window: window(
                "8066106561246197141489143085635637180530466139316558837890625",
                "8066691910053254194982743085635637180530466139316558837890625",
            ),
            stated_as: StatedAs::BeforeBridgeEpoch,
            as_published: "6 to 7 Ma ago",
            citation: BETTS_2018,
        },
        Event {
            id: "bridge-epoch",
            label: "the bridge epoch",
            description: "SI_EPOCH: year 0 of the proleptic Gregorian calendar. \
                          Exact by declaration, and the only entry here that is — \
                          it is a definition, not a measurement",
            window: window(
                "8070204002895596515944343085635637180530466139316558837890625",
                "8070204002895596515944343085635637180530466139316558837890625",
            ),
            stated_as: StatedAs::AfterDatum,
            as_published: "0000-01-01T00:00:00 TT",
            citation: PLANCK_2018,
        },
    ]
}

/// One event by id.
pub fn by_id(id: &str) -> Result<Event, TimeError> {
    all()
        .into_iter()
        .find(|e| e.id == id)
        .ok_or(TimeError::with_context(
            Code::E0012,
            "no such event in the catalogue",
        ))
}

/// The catalogue in chronological order, by window lower bound.
pub fn chronological() -> Vec<Event> {
    let mut v = all();
    v.sort_by(|a, b| a.window.lo().cmp(b.window.lo()));
    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::num::Ratio;
    use ucal_core::Rounding;

    #[test]
    fn every_entry_is_cited_and_window_valued() {
        // §17's requirement, checked entry by entry.
        for e in all() {
            assert!(!e.id.is_empty());
            assert!(!e.label.is_empty());
            assert!(!e.description.is_empty(), "{}", e.id);
            assert!(!e.as_published.is_empty(), "{}", e.id);
            assert!(e.citation.source.len() > 20, "{} has no real citation", e.id);
            assert!(
                e.window.lo() <= e.window.hi(),
                "{} has an inverted window",
                e.id
            );
        }
    }

    #[test]
    fn only_the_declared_epoch_is_exact() {
        // Every measurement is an interval. The one point value in the catalogue
        // is a definition, and it says so.
        for e in all() {
            if e.id == "bridge-epoch" {
                assert!(e.window.is_exact(), "a declaration may be exact");
                assert!(e.description.contains("definition"));
            } else {
                assert!(
                    !e.window.is_exact(),
                    "{} is a measurement and must carry its uncertainty",
                    e.id
                );
            }
        }
    }

    #[test]
    fn the_first_twenty_million_years_carry_w0006() {
        // §10.6: any statement inside the claim half-width must surface the
        // warning, because there the datum's own identification is comparable to
        // or larger than the quantity being discussed.
        let expect_warned = ["inflation", "recombination"];
        for e in all() {
            let warned = e.warning() == Some(Warning::W0006);
            assert_eq!(
                warned,
                expect_warned.contains(&e.id),
                "{} warning state is wrong",
                e.id
            );
        }
    }

    #[test]
    fn the_claim_dwarfs_the_events_it_warns_about() {
        // The point of the warning, made numerically: recombination is placed at
        // 380 kyr relative to a zero that is itself uncertain by 20 Myr — fifty
        // times the interval being quoted.
        let rec = by_id("recombination").unwrap();
        let half = UC1::big_bang_claim().hi().magnitude().ticks().clone();
        let mid = rec.window.midpoint(Rounding::HalfEven).unwrap();
        let ratio = Ratio::new(half, mid.ticks().clone()).unwrap();
        let times = ratio.to_decimal_string(0, Rounding::HalfEven).unwrap();
        assert_eq!(
            times, "60",
            "the datum's uncertainty is ~60x recombination's own age"
        );
        assert_eq!(rec.warning(), Some(Warning::W0006));
    }

    #[test]
    fn events_after_the_claim_window_do_not_warn() {
        for id in ["first-stars", "solar-system", "k-pg", "bridge-epoch"] {
            assert_eq!(by_id(id).unwrap().warning(), None, "{id}");
        }
    }

    #[test]
    fn the_catalogue_is_chronological_and_spans_the_domain() {
        let c = chronological();
        for w in c.windows(2) {
            assert!(
                w[0].window.lo() <= w[1].window.lo(),
                "{} should precede {}",
                w[0].id,
                w[1].id
            );
        }
        assert_eq!(c.first().unwrap().id, "inflation");
        assert_eq!(c.last().unwrap().id, "bridge-epoch");
    }

    #[test]
    fn published_figures_round_trip_to_the_windows() {
        // A spot check that the tick values were not transcribed from a different
        // computation: recombination's window must be 240-430 kyr of Julian years
        // after the datum.
        let rec = by_id("recombination").unwrap();
        let year = UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
            .unwrap();
        let lo_years = Ratio::new(rec.window.lo().ticks().clone(), year.clone()).unwrap();
        let hi_years = Ratio::new(rec.window.hi().ticks().clone(), year).unwrap();
        assert_eq!(
            lo_years.to_decimal_string(0, Rounding::HalfEven).unwrap(),
            "240000"
        );
        assert_eq!(
            hi_years.to_decimal_string(0, Rounding::HalfEven).unwrap(),
            "430000"
        );
    }

    #[test]
    fn events_before_the_bridge_epoch_are_stated_that_way() {
        // "Ago" is converted against SI_EPOCH, which is fixed; a moving `now`
        // would make the catalogue non-reproducible.
        let epoch = UC1::origin_offset();
        for e in all() {
            match e.stated_as {
                StatedAs::BeforeBridgeEpoch => {
                    assert!(e.window.hi().ticks() < &epoch, "{} is not before", e.id)
                }
                StatedAs::AfterDatum => {}
            }
        }
    }

    #[test]
    fn the_citation_set_is_versioned_independently() {
        // D-7: revising a citation bumps this crate, not the library.
        assert!(CITATION_SET.contains("ucal-events/"));
        assert!(CITATION_SET.contains("Planck 2018"));
    }

    #[test]
    fn unknown_ids_are_an_error_not_a_default() {
        assert_eq!(by_id("nonexistent").unwrap_err().code, Code::E0012);
    }
}
