//! `LegacyCalendar` must stay implementable from outside this crate.
//!
//! X3. §8.6 keeps legacy calendars for interoperation and the crate ships two;
//! a third — a regional reckoning, an ecclesiastical table — is the obvious
//! reason someone outside would implement this trait, and nothing stops them.
//!
//! **Updated in 1.8.0.** It used to borrow `Gregorian.tables()`, because
//! `DeclaredTables` was `#[non_exhaustive]` with no constructor and no outsider
//! could build one. A1 gave it one — along with `DeclaredLeapRule` and
//! `Discontinuity`, without which the first was decorative — so this fixture now
//! declares its **own** tables, which is what §8.6's "preserved for
//! interoperation" was supposed to mean all along.

use std::sync::OnceLock;
use ucal_civil::legacy::{
    DeclaredLeapRule, DeclaredTables, Discontinuity, LegacyCalendar, LegacyFields,
};
use ucal_civil::{CivilCalendar, Gregorian, Scale};
use ucal_core::qualified::CalendarIdentity;
use ucal_core::{Citation, Instant, Kind, Rounding, UC1};

pub struct Downstream;

impl CalendarIdentity for Downstream {
    fn id(&self) -> &str {
        "downstream-legacy"
    }
    fn kind(&self) -> Kind {
        Kind::Legacy
    }
}

/// Tables this crate has never seen: a thirteen-month year is not expressible
/// (the type fixes twelve), but the lengths, the cycle and the reform are the
/// caller's own.
static DOWNSTREAM_TABLES: OnceLock<DeclaredTables> = OnceLock::new();

fn tables() -> &'static DeclaredTables {
    DOWNSTREAM_TABLES.get_or_init(|| {
        DeclaredTables::new(
            [30, 30, 31, 30, 31, 30, 31, 30, 31, 30, 31, 30],
            10,
            DeclaredLeapRule::new(8, 33, true).expect("33 years is a cycle"),
            Some(
                Discontinuity::new(
                    "a reform this crate has never heard of",
                    (1700, 2, 18),
                    (1700, 3, 1),
                    11,
                )
                .expect("eleven days is a skip"),
            ),
            &["every table in this calendar was chosen by its author"],
        )
        .expect("the months sum to 365 and the arbitrariness is declared")
    })
}

impl LegacyCalendar for Downstream {
    fn tables(&self) -> &'static DeclaredTables {
        tables()
    }
    fn citation(&self) -> Citation {
        Citation::new("a downstream table, cited by its author", None)
    }
    fn civil(&self) -> CivilCalendar {
        CivilCalendar::Gregorian
    }
    fn fields(
        &self,
        t: &Instant<UC1>,
        scale: Scale,
        digits: u8,
        rounding: Rounding,
    ) -> ucal_core::error::Result<LegacyFields> {
        Gregorian.fields(t, scale, digits, rounding)
    }
    fn instant(&self, f: &LegacyFields) -> ucal_core::error::Result<Instant<UC1>> {
        Gregorian.instant(f)
    }
}

fn main() {
    assert_eq!(Downstream.id(), "downstream-legacy");
    assert!(!Downstream.kind().is_derived());
    // Its own tables, not a shipped calendar's.
    assert_eq!(Downstream.tables().week_length, 10);
    assert_ne!(Downstream.tables().month_lengths, Gregorian.tables().month_lengths);
}
