//! # ucal-civil — the SI bridge and legacy civil interop
//!
//! Rule A.3 permits conversion to a foreign unit system only through a constant
//! explicitly declared a **bridge constant**. Profile `UC-1` declares exactly one,
//! `SECOND`, and this crate is the only place it is used.
//!
//! Rule L: **TT is the only pivot.** Leap seconds exist solely at the UTC
//! parse/format boundary and never appear in absolute-time arithmetic.
//!
//! Everything here is Earth metrology, quarantined from `ucal-core` by the
//! dependency graph itself (§12).
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "hifitime")]
pub mod bridge;
pub mod calendar;
pub mod legacy;
pub mod jd;
pub mod leap;
pub mod rubber;
pub mod si;

pub use calendar::CivilCalendar;
pub use legacy::{Gregorian, Julian, LegacyCalendar};
pub use leap::leap_table_version;
pub use si::{
    from_civil, from_si_seconds, to_civil, to_si_seconds, CivilFields, Scale, SiSeconds, SubSecond,
};
