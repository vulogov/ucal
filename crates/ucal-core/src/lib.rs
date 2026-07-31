//! # ucal-core — absolute time in Planck ticks
//!
//! An unsigned integer count of Planck-time units since a stipulated datum, and a
//! positional calendar over that integer in base 5. Implements Part A of
//! RFC UCAL-1 for profile `UC-1`.
//!
//! ## What tick 0 is
//!
//! Tick 0 is a **stipulated** datum, conventionally identified with the FLRW t→0
//! limit. It is exact by declaration and unrevisable within a profile. It is not
//! a measurement, not a derivation, and not an observed event (Rule Q, N17).
//!
//! The stipulation is a necessity rather than a shortcut, for three independent
//! reasons, each sufficient:
//!
//! 1. Exactness cannot come from measurement. The published age carries
//!    ±0.020 Gyr — about 1.17×10⁵⁸ ticks, 0.145% of the span. A datum inheriting
//!    that error bar would make every timestamp uncertain relative to zero.
//! 2. The t→0 limit is not an observable event; it is where the FLRW
//!    extrapolation's coordinates degenerate, and classical time is undefined
//!    below roughly one tick.
//! 3. The extrapolation is model-dependent. Under inflation the FLRW t→0 limit is
//!    not a physical event at all.
//!
//! This puts the datum in ordinary company: TAI's 1958-01-01, the Julian Day
//! epoch, the Unix epoch. The parallel to the SI second is exact — 9 192 631 770
//! caesium cycles was *chosen* to match the ephemeris second, and the definition
//! does not inherit that provenance.
//!
//! What a profile does assert about physics is carried separately, as
//! [`profile::Profile::big_bang_claim`], and no arithmetic can consume it.
//!
//! ## What this crate does not contain
//!
//! Rule A.2: no Earth-derived quantity is referenced, named or defined here. The
//! Julian year, the day, the hour and every civil calendar live outside, in
//! `ucal-civil`. The single exception is the declared bridge constant
//! ([`profile::Bridge`]), which exists precisely so that the boundary is visible.
//!
//! Rule E: no floating-point type appears in any signature, field, constant or
//! intermediate. A CI lint enforces it.
//!
//! ## Layout
//!
//! - [`backend`] — the integer backend and the [`backend::TickInt`] surface.
//!   Rule W keeps the domain `[0, 2^512)` on both backends.
//! - [`error`] — Appendix E diagnostics.
//! - [`tier`] — the `5^(5k)` grid (Rule G) and tier naming (Rule N).
//! - [`value`] — [`value::Instant`], [`value::Delta`], [`value::Window`] and the
//!   inert [`value::SignedWindow`].
//! - [`codec`] — the two text forms (§6, Rule D) and the Appendix F group codec.
//! - [`ident`] — canonical binary (§7.1) and UCID (§7.2), and the Rule S
//!   ordering property both depend on.
//! - [`locale`] — tier-name tables (Appendix D). Names are display-only
//!   (Rule N), so adding one cannot change what a value means.
//! - [`num`] — Appendix H: widening `mul_div`, directed `isqrt`, exact rationals,
//!   interval arithmetic, continued fractions.
//! - [`profile`] — [`profile::UC1`], the datum, and provenance.
//! - [`qualified`] — [`qualified::Kind`] and the calendar qualifier every local
//!   rendering must carry (§6.6, §13.4).
//!
//! Rule S in one sentence: the binary form and UCID sort lexicographically in
//! chronological order; the text forms do not, unless zero-padded to a fixed
//! tier width.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
// Rule E: enforced structurally here, and by `cargo run -p xtask -- lint` across
// the workspace. There is no float type in this crate to deny.
#![deny(clippy::float_arithmetic)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod backend;
pub mod codec;
pub mod error;
pub mod ident;
pub mod locale;
pub mod num;
pub mod profile;
pub mod qualified;
pub mod tier;
pub mod value;

pub use backend::{TickInt, Ticks, CANONICAL_BYTES, DOMAIN_BITS};
pub use codec::{Fmt, Form};
/// GE-5: text forms are radix formatting, so they need an allocator. Without
/// one the type system, the arithmetic and the binary codec all remain — only
/// the human-readable rendering goes.
#[cfg(feature = "alloc")]
pub use codec::{parse, render};
pub use error::{Code, TimeError, Warning};
pub use ident::{Ucid, CROCKFORD, UCID_BITS, UCID_LEN};
pub use locale::LocaleId;
pub use num::{isqrt_ceil, isqrt_floor, mul_div, RatInterval, Ratio};
/// Continued fractions return a `Vec` of terms, so they too need an allocator.
#[cfg(feature = "alloc")]
pub use num::{cf_expand, convergents};
pub use profile::{Bridge, Citation, Frame, MeasuredValue, Profile, Provenance, UC1};
pub use qualified::{CalendarIdentity, CalendarQualifier, Kind, Qualified};
pub use tier::{Tier, TierName, TierTable, GROUP_BASE};
pub use value::{
    Delta, Instant, IntervalOrdering, Precision, Rounding, Sign, Signed, SignedWindow, Span,
    Stated, Window,
};

/// The RFC revision this crate implements.
pub const RFC: &str = "UCAL-1 final draft, 2026-07-29";

/// Deltas applied against the RFC text; see `spec/SPEC-DELTAS.md`.
pub const SPEC_DELTAS: &[&str] = &[
    "D-A2: ORIGIN_OFFSET has 61 trailing base-5 zeros, not 62 (editorial)",
    "D-A3: Appendix B's seconds column is imprecise; the table is generated (editorial)",
    "D-A4: Appendix C's human forms are truncated at T-5, not tick-exact (correction)",
    "D-A5: grouping cycles are declared per body, not admitted by a global bound (amendment)",
    "D-A6: Earth body parameters are chosen to reproduce Appendix I (editorial)",
    "D-A7: full-width encode is 45 divmod steps, not 44 (correction)",
    "D-A8: precision is the last group's tier; forms are anchored per-form (amendment)",
    "D-A9: §6.6 needs a calendar-id grammar to disambiguate qualifier from body (amendment)",
    "D-A10: Appendix A's implied age is the unrounded input, not the quotient (editorial)",
    "D-A11: obliquity is an angle and cannot be a RatedParam under Rule C (correction)",
    "D-A12: §9.6's synodic formula contradicts Appendix I.2; the year-relative form is correct (correction)",
    "D-A13: a drift bound is a rate in local units, not a Delta (correction)",
];
