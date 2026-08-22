//! `LegacyCalendar` must stay implementable from outside this crate.
//!
//! X3. §8.6 keeps legacy calendars for interoperation and the crate ships two;
//! a third — a regional reckoning, an ecclesiastical table — is the obvious
//! reason someone outside would implement this trait, and nothing stops them.
//!
//! Note what the fixture cannot do. `DeclaredTables` is `#[non_exhaustive]` with
//! no constructor, so an outsider cannot **build** one and must borrow a shipped
//! calendar's. That is a real constraint on what a third legacy calendar can be
//! and it does not seal the trait — the same shape as `Profile`, whose
//! implementors must delegate `bridge()` rather than construct a `Bridge`.
//!
//! It was nearly read as a seal there, and the lesson is recorded in
//! `Documentation/Proposals/V1-check-audit.md`: an unconstructible return type
//! narrows what an implementor can say, and only forbids implementation
//! entirely if nothing shipped returns one.

use ucal_civil::legacy::{DeclaredTables, LegacyCalendar, LegacyFields};
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

impl LegacyCalendar for Downstream {
    fn tables(&self) -> &'static DeclaredTables {
        Gregorian.tables()
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
}
