//! Calendar-qualified renderings (§6.6, §13.4).
//!
//! # The qualifier is not optional, and not a convention
//!
//! §6.6: emitting a local calendar rendering without its id and kind is
//! `UCAL-E0007`. A rule of that shape can be satisfied two ways — by discipline,
//! or by construction — and only the second survives contact with a codebase.
//!
//! So there is no type here that renders a local calendar value on its own.
//! [`Qualified`] is the only thing that implements [`core::fmt::Display`] for one,
//! and it cannot be built without a [`CalendarQualifier`]. A caller who wants a
//! string must state which calendar produced it and whether that calendar was
//! *derived* (Rule K) or *legacy* (§8.6). §13.4 puts [`Kind`] in core for exactly
//! this reason: every rendering path, in every crate, has to route through it.
//!
//! # Why the distinction is load-bearing
//!
//! Failure mode F9 is "Earth becomes the template rather than an instance". A
//! legacy calendar is a declared table — irregular month lengths, a seven-day
//! week with no astronomical period behind it, an intercalation rule that is not
//! a continued-fraction convergent. A derived calendar is a consequence of a
//! body's periods and nothing else. Presenting the two without distinction would
//! let the first pass for the second, which is precisely the confusion Rule K
//! exists to prevent.

#[cfg(feature = "alloc")]
use alloc::string::String;
use core::fmt;

use crate::error::{Code, Result, TimeError};

/// Whether a calendar is a Rule K derivation or declared legacy data.
///
/// Lives in core so that no rendering path anywhere in the workspace can avoid
/// stating it (§13.4).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Kind {
    /// Produced by the single derivation mechanism of Rule K: units from the
    /// body's periods, intercalation from continued fractions, grouping from a
    /// declared satellite. Nothing about it is a table.
    Derived,
    /// Declared table data preserved for interoperation (§8.6). Outside Rule K,
    /// and marked as such in every output.
    Legacy,
}

impl Kind {
    /// The suffix convention of §6.6: `-d` marks a derivation.
    pub const fn marker(self) -> &'static str {
        match self {
            Kind::Derived => "-d",
            Kind::Legacy => "",
        }
    }

    /// Whether this kind is a Rule K derivation.
    pub const fn is_derived(self) -> bool {
        matches!(self, Kind::Derived)
    }

    /// The warning that accompanies a value from this kind of calendar, if any.
    ///
    /// A legacy value carries `UCAL-W0005` on request (§8.6).
    pub const fn warning(self) -> Option<crate::error::Warning> {
        match self {
            Kind::Derived => None,
            Kind::Legacy => Some(crate::error::Warning::W0005),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Derived => "derived",
            Kind::Legacy => "legacy",
        })
    }
}

/// Anything that can say which calendar it is and what kind.
///
/// Both trait objects and concrete calendars implement it, so a runtime check is
/// available where the type is erased — see [`require_derived`].
pub trait CalendarIdentity {
    /// The calendar's id, e.g. `"earth-civil"` or `"earth-d"`.
    fn id(&self) -> &str;
    /// Derived or legacy.
    fn kind(&self) -> Kind;
    /// The anchor revision, for a derived calendar (Rule J.5). Legacy calendars
    /// have no anchor and return `None`.
    fn revision(&self) -> Option<u32> {
        None
    }
}

/// Reject a legacy calendar where Rule K requires a derived one (`UCAL-E0065`).
///
/// The primary defence is the type system: `LegacyCalendar` and `BodyCalendar`
/// are distinct traits with no blanket conversion, so the mistake usually cannot
/// be written. This is the fallback for erased types, where the compiler no
/// longer knows which is which.
pub fn require_derived(c: &dyn CalendarIdentity) -> Result<()> {
    if c.kind().is_derived() {
        Ok(())
    } else {
        Err(TimeError::with_context(
            Code::E0065,
            "this operation requires a calendar derived under Rule K; a legacy \
             calendar is declared table data and cannot substitute for one",
        ))
    }
}

/// The `id`, `kind` and optional revision that every local rendering must carry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CalendarQualifier<'a> {
    id: &'a str,
    kind: Kind,
    revision: Option<u32>,
}

impl<'a> CalendarQualifier<'a> {
    /// A qualifier for a derived calendar, with the anchor revision that produced
    /// the value (Rule J.5 — renderings carry it so values from different
    /// revisions are never silently compared).
    pub const fn derived(id: &'a str, revision: u32) -> Self {
        CalendarQualifier {
            id,
            kind: Kind::Derived,
            revision: Some(revision),
        }
    }

    /// A qualifier for a legacy calendar. There is no revision, because there is
    /// no anchor — a legacy calendar is a table, not a determination.
    pub const fn legacy(id: &'a str) -> Self {
        CalendarQualifier {
            id,
            kind: Kind::Legacy,
            revision: None,
        }
    }

    /// The calendar id.
    pub const fn id(&self) -> &'a str {
        self.id
    }

    /// Derived or legacy.
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// The anchor revision, if this is a derived calendar.
    pub const fn revision(&self) -> Option<u32> {
        self.revision
    }

    /// Attach a value, producing something that can be rendered.
    pub const fn attach<T>(self, value: T) -> Qualified<'a, T> {
        Qualified {
            qualifier: self,
            value,
        }
    }
}

impl fmt::Display for CalendarQualifier<'_> {
    /// `earth-d/1` or `earth-civil` (§6.6).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id)?;
        if let Some(r) = self.revision {
            write!(f, "/{r}")?;
        }
        Ok(())
    }
}

/// A local calendar value together with the qualifier §6.6 requires.
///
/// This is the only way to render one. There is deliberately no `Display` on the
/// field structs themselves, and no `From<Fields> for String`, so an unqualified
/// rendering cannot be produced by accident — see `tests/compile_fail/`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Qualified<'a, T> {
    qualifier: CalendarQualifier<'a>,
    value: T,
}

impl<'a, T> Qualified<'a, T> {
    /// The qualifier.
    pub const fn qualifier(&self) -> &CalendarQualifier<'a> {
        &self.qualifier
    }

    /// The underlying value.
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consume, yielding the value. Named so that discarding the qualifier is a
    /// visible act rather than a coercion.
    pub fn into_unqualified(self) -> T {
        self.value
    }

    /// The warning this rendering carries, if any (`UCAL-W0005` for legacy).
    pub const fn warning(&self) -> Option<crate::error::Warning> {
        self.qualifier.kind.warning()
    }
}

impl<T: fmt::Display> fmt::Display for Qualified<'_, T> {
    /// `earth-civil: 2026-07-29T00:00:00Z`, `earth-d/1: 2026-208.4137` (§6.6).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.qualifier, self.value)
    }
}

/// Whether a string is a well-formed calendar id.
///
/// §6.6 gives examples — `earth-d`, `earth-civil`, `mars-d` — but does not define
/// the grammar, and without one the notation is ambiguous: the *body* of a
/// rendering may contain colons (`earth-civil: 2026-07-29T00:00:00Z`), so a naive
/// split at the first colon happily produces a "calendar id" of `2026-07-29T00`.
///
/// The grammar adopted here is the narrowest that admits every id the RFC uses:
/// a lowercase letter, then lowercase letters, digits and hyphens. Requiring a
/// leading letter is what disambiguates a qualifier from a date, since every date
/// form in this specification begins with a digit. See `spec/SPEC-DELTAS.md`
/// D-A9.
pub fn is_valid_calendar_id(id: &str) -> bool {
    let mut bytes = id.bytes();
    match bytes.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    bytes.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// Split a qualified rendering into its qualifier and body.
///
/// `UCAL-E0007` when the qualifier is absent or malformed — the parse-side half
/// of §6.6's requirement. The kind is inferred from the id: `-d` marks a Rule K
/// derivation, anything else is legacy.
#[cfg(feature = "alloc")]
pub fn split_qualified(s: &str) -> Result<(String, Option<u32>, Kind, &str)> {
    use alloc::string::ToString;
    let Some((head, body)) = s.split_once(':') else {
        return Err(TimeError::with_context(
            Code::E0007,
            "a local calendar rendering must carry its calendar id and kind (§6.6)",
        ));
    };
    let head = head.trim();
    if head.is_empty() {
        return Err(TimeError::with_context(Code::E0007, "empty calendar id"));
    }
    let (id, revision) = match head.split_once('/') {
        None => (head, None),
        Some((i, r)) => {
            let n: u32 = r
                .parse()
                .map_err(|_| TimeError::with_context(Code::E0007, "malformed anchor revision"))?;
            (i, Some(n))
        }
    };
    if !is_valid_calendar_id(id) {
        return Err(TimeError::with_context(
            Code::E0007,
            "malformed calendar id: expected a lowercase letter followed by \
             lowercase letters, digits or hyphens (§6.6). A rendering whose body \
             contains a colon must still be qualified.",
        ));
    }
    let kind = if id.ends_with("-d") {
        Kind::Derived
    } else {
        Kind::Legacy
    };
    if kind.is_derived() && revision.is_none() {
        return Err(TimeError::with_context(
            Code::E0007,
            "a derived calendar rendering must state its anchor revision (Rule J.5)",
        ));
    }
    Ok((id.to_string(), revision, kind, body.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Warning;

    struct FakeLegacy;
    impl CalendarIdentity for FakeLegacy {
        fn id(&self) -> &str {
            "earth-civil"
        }
        fn kind(&self) -> Kind {
            Kind::Legacy
        }
    }

    struct FakeDerived;
    impl CalendarIdentity for FakeDerived {
        fn id(&self) -> &str {
            "earth-d"
        }
        fn kind(&self) -> Kind {
            Kind::Derived
        }
        fn revision(&self) -> Option<u32> {
            Some(1)
        }
    }

    #[test]
    fn rendering_always_states_the_calendar() {
        let q = CalendarQualifier::legacy("earth-civil").attach("2026-07-29T00:00:00Z");
        assert_eq!(q.to_string(), "earth-civil: 2026-07-29T00:00:00Z");

        let q = CalendarQualifier::derived("earth-d", 1).attach("2026-208.4137");
        assert_eq!(q.to_string(), "earth-d/1: 2026-208.4137");

        let q = CalendarQualifier::derived("mars-d", 1).attach("0212-334.0918");
        assert_eq!(q.to_string(), "mars-d/1: 0212-334.0918");
    }

    #[test]
    fn legacy_renderings_carry_w0005() {
        let q = CalendarQualifier::legacy("earth-civil").attach("x");
        assert_eq!(q.warning(), Some(Warning::W0005));
        let q = CalendarQualifier::derived("earth-d", 1).attach("x");
        assert_eq!(q.warning(), None);
        assert_eq!(Kind::Legacy.warning(), Some(Warning::W0005));
        assert_eq!(Kind::Derived.warning(), None);
    }

    #[test]
    fn require_derived_rejects_legacy() {
        // UCAL-E0065, the runtime half of the guarantee.
        assert!(require_derived(&FakeDerived).is_ok());
        let e = require_derived(&FakeLegacy).unwrap_err();
        assert_eq!(e.code, Code::E0065);
        assert_eq!(e.code.exit_code(), 7);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn unqualified_input_is_e0007() {
        for bad in [
            // No qualifier at all.
            "2026-208.4137",
            ": something",
            "  : x",
            // The subtle one: a bare civil timestamp *contains* colons, so a
            // naive split finds a "qualifier" of `2026-07-29T00`. Requiring the
            // id to begin with a lowercase letter is what rejects it.
            "2026-07-29T00:00:00Z",
            "12:34:56",
            // Ids may not contain uppercase, underscores or spaces.
            "Earth-Civil: x",
            "earth_civil: x",
            "earth civil: x",
        ] {
            let e = split_qualified(bad).unwrap_err();
            assert_eq!(e.code, Code::E0007, "input {bad:?} should be rejected");
        }
    }

    #[test]
    fn calendar_id_grammar() {
        for good in ["earth-d", "earth-civil", "mars-d", "titan-d", "a", "x1-y2"] {
            assert!(is_valid_calendar_id(good), "{good} should be valid");
        }
        for bad in ["", "2026", "2026-07-29T00", "Earth", "earth_civil", "-d", "earth d"] {
            assert!(!is_valid_calendar_id(bad), "{bad} should be invalid");
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn a_body_containing_colons_still_parses() {
        // The whole point of the grammar: the body is free to contain colons.
        let (id, _, kind, body) = split_qualified("earth-civil: 2026-07-29T00:00:00Z").unwrap();
        assert_eq!(id, "earth-civil");
        assert_eq!(kind, Kind::Legacy);
        assert_eq!(body, "2026-07-29T00:00:00Z");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn qualified_input_round_trips() {
        let (id, rev, kind, body) = split_qualified("earth-civil: 2026-07-29T00:00:00Z").unwrap();
        assert_eq!(id, "earth-civil");
        assert_eq!(rev, None);
        assert_eq!(kind, Kind::Legacy);
        assert_eq!(body, "2026-07-29T00:00:00Z");

        let (id, rev, kind, body) = split_qualified("earth-d/1: 2026-208.4137").unwrap();
        assert_eq!(id, "earth-d");
        assert_eq!(rev, Some(1));
        assert_eq!(kind, Kind::Derived);
        assert_eq!(body, "2026-208.4137");
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn a_derived_rendering_must_state_its_anchor_revision() {
        // Rule J.5: revisions are carried so values from different determinations
        // are never silently compared.
        let e = split_qualified("earth-d: 2026-208.4137").unwrap_err();
        assert_eq!(e.code, Code::E0007);
        assert!(split_qualified("earth-d/3: x").is_ok());
        // A legacy calendar has no anchor, so none is required.
        assert!(split_qualified("earth-civil: x").is_ok());
    }

    #[test]
    fn discarding_the_qualifier_is_explicit() {
        let q = CalendarQualifier::legacy("earth-civil").attach(42);
        assert_eq!(*q.value(), 42);
        assert_eq!(q.qualifier().id(), "earth-civil");
        assert_eq!(q.qualifier().kind(), Kind::Legacy);
        // Named, not a Deref or an Into.
        assert_eq!(q.into_unqualified(), 42);
    }
}
