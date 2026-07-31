//! # ucal-body — bodies, their parameters, and their provenance
//!
//! Rule K's mechanism operates on three periods and a list of satellites. This
//! crate carries them, with everything Rule C requires: an epoch, a validity
//! window, a citation, and the published value verbatim.
//!
//! ## What is deliberately absent
//!
//! - **Any dependency on `ucal-civil`.** §12 forbids it, because the derived path
//!   must not be able to reach the declared civil tables even by accident —
//!   failure mode F9. The lint asserts the absence.
//! - **Any body-specific code path.** Rule K.5: Earth is an ordinary instance.
//! - **Any phase on a `Body`.** An anchor is empirical (N15), belongs to a
//!   *calendar* rather than to a body, and lives in [`anchor`] and [`anchors`].
//! - **Any calendar.** Calendars are derived from bodies, not carried by them.
//!
//! ## What is stored, and in what
//!
//! Every parameter is an exact rational of **ticks**. A period kept in seconds
//! would put a foreign unit inside the one mechanism Rule K exists to keep
//! Earth-free. The published value is kept alongside, verbatim, because Rule Y.1
//! requires a conversion to be auditable rather than merely trusted.
#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod anchor;
pub mod anchors;
pub mod body;
pub mod calendar;
pub mod data;
pub mod derive;
pub mod param;

pub use anchor::{Anchor, Determination, Meridian, PhaseDefinition};
pub use body::{AngleParam, Body, Satellite};
pub use calendar::{BodyCalendar, CyclePosition, DerivedFields};
pub use derive::{derive_cycles, derive_leap_rule, Convergent, Cycle, DriftBound, LeapRule};
pub use param::{Measured, MeasuredUnit, RatedParam};
