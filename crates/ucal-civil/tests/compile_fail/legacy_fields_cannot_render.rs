//! §6.6: "Emitting a local calendar rendering without this qualifier is
//! `UCAL-E0007`."
//!
//! The rule is enforced by construction rather than by discipline: `LegacyFields`
//! has no `Display`, so the only route to a string is `LegacyCalendar::render`,
//! which returns a `Qualified`. If this ever compiles, an unqualified local
//! rendering has become expressible and the guarantee is gone.

use ucal_civil::legacy::{Gregorian, LegacyCalendar};
use ucal_civil::si::{Scale, SubSecond};
use ucal_civil::from_civil;
use ucal_civil::CivilCalendar;
use ucal_core::Rounding;

fn main() {
    let t = from_civil(
        2026, 7, 29, 0, 0, 0,
        SubSecond::zero(), Scale::Tt, CivilCalendar::Gregorian,
    )
    .unwrap();
    let fields = Gregorian.fields(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
    let _unqualified: String = fields.to_string();
}
