//! Diagnostics (Appendix E).
//!
//! Every error carries its `UCAL-Ennnn` code, because the codes are the stable
//! contract: §22's conformance classes and §21.3's required assertions are
//! written in terms of them, not in terms of Rust type names.

use core::fmt;

/// A diagnostic code from Appendix E.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Code {
    // --- text and structure ---
    /// Malformed timestamp.
    E0001,
    /// Unknown profile tag.
    E0002,
    /// Mixed text forms in one string (Rule D).
    E0003,
    /// Group value out of range (> 3124).
    E0004,
    /// Invalid base-5 digit.
    E0005,
    /// Non-contiguous tier sequence.
    E0006,
    /// Calendar rendering without a kind/id qualifier (§6.6).
    E0007,
    /// Locale table load failure.
    E0010,
    /// Duplicate name in the active locale table (Rule N).
    E0011,
    /// Unknown key in an HJSON data file.
    E0012,
    /// Profile lacks a `datum_provenance` record (Rule Q.4).
    E0013,
    /// Name not found in the active locale table (Rule N).
    ///
    /// Distinct from [`Code::E0011`], which Rule N pins to a *collision* —
    /// two entries claiming one name. Until 0.9.0 a lookup miss also reported
    /// E0011, so the diagnostic read *"duplicate name in the active locale
    /// table (unknown tier name)"*, which states the opposite of what happened.
    /// See `spec/SPEC-DELTAS.md` D-A17.
    E0014,

    // --- domain ---
    /// Result precedes the datum (Rule Z).
    E0020,
    /// Result exceeds DOMAIN (Rules O, W).
    E0021,
    /// Window inversion, `lo > hi` (Rule U).
    E0022,
    /// Comparison indeterminate at the stated precision (Rule T).
    E0023,
    /// Lossy rendering requested without a rounding mode (Rule R).
    E0024,
    /// `BIG_BANG_CLAIM` or its half-width used as a computational operand
    /// (Rule Q.3). Reaching this at runtime is an internal invariant violation;
    /// the type system is supposed to make it unreachable, and §21.3-3 requires
    /// a compile-fail test proving so.
    E0025,

    // --- binary and identifiers ---
    /// Binary form is not 64 bytes (Rule B).
    E0030,
    /// Instant outside UCID range (Rule I).
    E0031,
    /// Invalid Crockford base-32.
    E0032,

    // --- civil bridge (§8, §14) ---
    /// Civil date outside the renderable range (§14.3). Never a panic.
    E0040,
    /// Invalid civil date for the stated calendar.
    E0041,
    /// `second = 60` outside a leap-second instant.
    E0042,
    /// Foreign-unit input finer than the bridge constant permits (Rules A, R, Y).
    E0043,

    // --- profile ---
    /// Profile mismatch (Rule P).
    E0050,

    // --- calendars (§9, §8.6) ---
    /// Body parameter missing required provenance or as-measured value
    /// (Rules C, Y).
    E0060,
    /// Leap-rule derivation cannot meet the requested drift bound.
    E0061,
    /// Calendar has no anchor; local fields cannot be produced (Rule J.3).
    E0062,
    /// Anchor phase definition not evaluable for this body's parameters
    /// (Rule J.4).
    E0063,
    /// Grouping cycle requested but none derivable from any satellite (§9.6).
    E0064,
    /// Legacy calendar supplied where a derived calendar is required (Rule K.6).
    E0065,

    // --- numerics and cosmology (Appendix H, Rule X) ---
    /// Division by zero, or by an interval containing zero (Appendix H.3);
    /// also a cosmology inversion that failed to bracket.
    E0070,
    /// Requested enclosure width unreachable at the permitted depth (Rule X).
    E0071,

    // --- tier grid ---
    /// Tier index outside the profile's grid.
    E0080,
}

impl Code {
    /// The wire-stable code string, e.g. `"UCAL-E0021"`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Code::E0001 => "UCAL-E0001",
            Code::E0002 => "UCAL-E0002",
            Code::E0003 => "UCAL-E0003",
            Code::E0004 => "UCAL-E0004",
            Code::E0005 => "UCAL-E0005",
            Code::E0006 => "UCAL-E0006",
            Code::E0007 => "UCAL-E0007",
            Code::E0010 => "UCAL-E0010",
            Code::E0011 => "UCAL-E0011",
            Code::E0012 => "UCAL-E0012",
            Code::E0013 => "UCAL-E0013",
            Code::E0014 => "UCAL-E0014",
            Code::E0020 => "UCAL-E0020",
            Code::E0021 => "UCAL-E0021",
            Code::E0022 => "UCAL-E0022",
            Code::E0023 => "UCAL-E0023",
            Code::E0024 => "UCAL-E0024",
            Code::E0025 => "UCAL-E0025",
            Code::E0030 => "UCAL-E0030",
            Code::E0031 => "UCAL-E0031",
            Code::E0032 => "UCAL-E0032",
            Code::E0040 => "UCAL-E0040",
            Code::E0041 => "UCAL-E0041",
            Code::E0042 => "UCAL-E0042",
            Code::E0043 => "UCAL-E0043",
            Code::E0050 => "UCAL-E0050",
            Code::E0060 => "UCAL-E0060",
            Code::E0061 => "UCAL-E0061",
            Code::E0062 => "UCAL-E0062",
            Code::E0063 => "UCAL-E0063",
            Code::E0064 => "UCAL-E0064",
            Code::E0065 => "UCAL-E0065",
            Code::E0070 => "UCAL-E0070",
            Code::E0071 => "UCAL-E0071",
            Code::E0080 => "UCAL-E0080",
        }
    }

    /// One-line description, matching Appendix E.
    pub const fn describe(self) -> &'static str {
        match self {
            Code::E0001 => "malformed timestamp",
            Code::E0002 => "unknown profile tag",
            Code::E0003 => "mixed text forms in one string",
            Code::E0004 => "group value out of range (> 3124)",
            Code::E0005 => "invalid base-5 digit",
            Code::E0006 => "non-contiguous tier sequence",
            Code::E0007 => "calendar rendering without a kind/id qualifier",
            Code::E0010 => "locale table load failure",
            Code::E0011 => "duplicate name in the active locale table",
            Code::E0012 => "unknown key in HJSON data file",
            Code::E0013 => "profile lacks a datum_provenance record",
            Code::E0014 => "name not found in the active locale table",
            Code::E0020 => "result precedes the datum",
            Code::E0021 => "result exceeds DOMAIN",
            Code::E0022 => "window inversion, lo > hi",
            Code::E0023 => "comparison indeterminate at stated precision",
            Code::E0024 => "lossy rendering requested without a rounding mode",
            Code::E0025 => "BIG_BANG_CLAIM used as a computational operand",
            Code::E0030 => "binary form is not 64 bytes",
            Code::E0031 => "instant outside UCID range",
            Code::E0032 => "invalid Crockford base-32",
            Code::E0040 => "civil date outside renderable range",
            Code::E0041 => "invalid civil date for the stated calendar",
            Code::E0042 => "second = 60 outside a leap-second instant",
            Code::E0043 => "foreign-unit input finer than the bridge constant permits",
            Code::E0050 => "profile mismatch",
            Code::E0060 => "body parameter missing required provenance or as-measured value",
            Code::E0061 => "leap-rule derivation cannot meet the requested drift bound",
            Code::E0062 => "calendar has no anchor; local fields cannot be produced",
            Code::E0063 => "anchor phase definition not evaluable for this body",
            Code::E0064 => "grouping cycle requested but none derivable from any satellite",
            Code::E0065 => "legacy calendar supplied where a derived calendar is required",
            Code::E0070 => "division by zero or by an interval containing zero",
            Code::E0071 => "requested enclosure width unreachable at the permitted depth",
            Code::E0080 => "tier index outside the profile grid",
        }
    }

    /// CLI exit code for this diagnostic (§19.5).
    pub const fn exit_code(self) -> u8 {
        match self {
            Code::E0001
            | Code::E0002
            | Code::E0003
            | Code::E0004
            | Code::E0005
            | Code::E0006
            | Code::E0007
            | Code::E0032 => 2,
            Code::E0020 | Code::E0021 | Code::E0022 | Code::E0030 | Code::E0031
            | Code::E0080 => 3,
            Code::E0023 | Code::E0024 | Code::E0043 => 4,
            Code::E0040 | Code::E0041 | Code::E0042 => 2,
            Code::E0050 => 5,
            Code::E0060 | Code::E0061 | Code::E0062 | Code::E0063 | Code::E0064
            | Code::E0065 => 7,
            Code::E0070 | Code::E0071 => 8,
            Code::E0010 | Code::E0011 | Code::E0012 | Code::E0013 | Code::E0014 => 6,
            Code::E0025 => 9,
        }
    }
}

impl fmt::Display for Code {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.as_str(), self.describe())
    }
}

/// An error from absolute-time arithmetic or representation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct TimeError {
    /// The Appendix E code.
    pub code: Code,
    /// Optional static context, e.g. which tier was out of range.
    pub context: Option<&'static str>,
}

impl TimeError {
    /// Construct from a code.
    pub const fn new(code: Code) -> Self {
        TimeError {
            code,
            context: None,
        }
    }
    /// Construct with static context.
    pub const fn with_context(code: Code, context: &'static str) -> Self {
        TimeError {
            code,
            context: Some(context),
        }
    }
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.context {
            None => write!(f, "{}", self.code),
            Some(c) => write!(f, "{} ({c})", self.code),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for TimeError {}

/// Shorthand for the crate's fallible operations.
pub type Result<T> = core::result::Result<T, TimeError>;

/// A diagnostic warning from Appendix E's W-series.
///
/// Warnings are separate from [`Code`] because they never abort an operation:
/// they accompany a value that was produced, and the caller decides what to do.
/// Rule R requires lossy renderings to be reported, and §8.4 requires a stale
/// leap-second table to warn with a bounded error rather than convert silently.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Warning {
    /// Precision loss in the requested rendering (Rule R).
    W0001,
    /// Leap-second table may be stale; the bounded error is reported with it.
    W0002,
    /// Body parameter evaluated outside its validity window (Rule C).
    W0003,
    /// Cosmology enclosure width exceeds one tick.
    W0004,
    /// Value produced by a legacy, non-derived calendar (§8.6).
    W0005,
    /// Quantity comparable to or smaller than `BIG_BANG_CLAIM`; the datum's
    /// physical identification is uncertain at this scale (§10.6).
    W0006,
}

impl Warning {
    /// The wire-stable code string, e.g. `"UCAL-W0001"`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Warning::W0001 => "UCAL-W0001",
            Warning::W0002 => "UCAL-W0002",
            Warning::W0003 => "UCAL-W0003",
            Warning::W0004 => "UCAL-W0004",
            Warning::W0005 => "UCAL-W0005",
            Warning::W0006 => "UCAL-W0006",
        }
    }

    /// One-line description, matching Appendix E.
    pub const fn describe(self) -> &'static str {
        match self {
            Warning::W0001 => "precision loss in the requested rendering",
            Warning::W0002 => "leap-second table may be stale; bounded error reported",
            Warning::W0003 => "body parameter evaluated outside its validity window",
            Warning::W0004 => "cosmology enclosure width exceeds one tick",
            Warning::W0005 => "value produced by a legacy (non-derived) calendar",
            Warning::W0006 => "quantity comparable to or smaller than BIG_BANG_CLAIM",
        }
    }
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.as_str(), self.describe())
    }
}
