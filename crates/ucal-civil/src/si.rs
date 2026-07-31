//! The SI bridge (§8, §14). **TT is the only pivot** (Rule L).
//!
//! # The exactness claim, and why it holds
//!
//! §8.2: converting *into* absolute time is `ORIGIN_OFFSET + s x SECOND`, and for
//! any `s` whose denominator divides `10^30` the product is an exact integer with
//! no rounding. Input finer than `10^-30` s is rejected with `UCAL-E0043`, never
//! rounded.
//!
//! That works because of D-3's choice of `SECOND = 18 548 584 399 861 x 10^30`
//! ticks. A sub-second fraction of `d <= 30` decimal digits contributes
//!
//! ```text
//! value x SECOND / 10^d  =  value x 18 548 584 399 861 x 10^(30 - d)
//! ```
//!
//! which is an integer for every `d <= 30`. The exponent `30` is not decoration:
//! it is precisely the number of decimal places the bridge can carry exactly, and
//! `UCAL-E0043` is what happens at digit 31.
//!
//! The reverse direction is *not* exact, and the specification says so. A tick is
//! about `5.39 x 10^-44` s, so almost no tick count is a terminating decimal
//! number of seconds. `to_civil` therefore rounds under an explicit [`Rounding`]
//! and reports the loss (Rule R, `UCAL-W0001`). Exactly one direction rounds, and
//! it is the rendering direction — which is Rule R's whole content.
//!
//! # Nothing precedes the datum
//!
//! A civil label far enough in the past would name an instant before tick 0.
//! There is no such instant: the domain is unsigned (Rule Z, N12). Such a
//! conversion fails with `UCAL-E0020` rather than producing a negative or wrapped
//! value, and `no_civil_label_can_precede_the_datum` pins that at the bridge
//! rather than relying on the type alone.
//!
//! # SI units are not calendar units
//!
//! §8.3: [`day_si`] is 86400 SI seconds, **not** one rotation of Earth. The
//! latter is a `Body` parameter and is not exactly 86400 s. Conflating them is
//! the error this split exists to prevent, which is why there is no `to_years`
//! or `to_months` here — such a function has no meaning without an explicit
//! definition parameter, and those live in `ucal-body`.

#[cfg(feature = "alloc")]
use alloc::string::ToString;

use ucal_core::backend::TickInt;
use ucal_core::num::Ratio;
use ucal_core::{Code, Instant, Rounding, Ticks, TimeError, Warning, UC1};
use ucal_core::Profile;

use crate::calendar::{
    check_date, check_time, civil_from_days, days_from_civil, CivilCalendar,
};
use crate::leap::{
    has_leap_second, tai_minus_utc, table_covers, utc_from_tai_seconds, TT_MINUS_TAI_MILLIS,
};
use crate::rubber;

type Result<T> = core::result::Result<T, TimeError>;

// ---------------------------------------------------------------------------
// §8.3 — SI duration units, in ticks
// ---------------------------------------------------------------------------

/// The bridge constant: one SI second in ticks (Rule A.3).
///
/// The **only** door between absolute time and a foreign unit system.
pub fn second() -> Ticks {
    UC1::bridge().ticks
}

/// The decimal mantissa of [`second`]: `SECOND = MANTISSA x 10^30`.
const SECOND_MANTISSA: u64 = 18_548_584_399_861;

/// The decimal scale of [`second`], and so the number of decimal places the
/// bridge carries exactly (D-3).
pub const SECOND_DECIMAL_DIGITS: u8 = 30;

fn pow10(e: u32) -> Result<Ticks> {
    let ten = <Ticks as TickInt>::from_u64(10);
    let mut acc = <Ticks as TickInt>::one();
    for _ in 0..e {
        acc = acc
            .try_mul(&ten)
            .ok_or(TimeError::new(Code::E0021))?;
    }
    Ok(acc)
}

fn scaled(n: u64) -> Result<Ticks> {
    second()
        .try_mul(&<Ticks as TickInt>::from_u64(n))
        .ok_or(TimeError::new(Code::E0021))
}

/// One nanosecond in ticks. Exact: `SECOND` is divisible by `10^9`.
pub fn nanosecond() -> Ticks {
    let (q, r) = second().quot_rem(&<Ticks as TickInt>::from_u64(1_000_000_000));
    debug_assert!(r.is_zero_ticks(), "SECOND must divide exactly by 10^9");
    q
}

/// One minute: 60 SI seconds.
pub fn minute() -> Ticks {
    scaled(60).expect("within domain")
}

/// One hour: 3600 SI seconds.
pub fn hour() -> Ticks {
    scaled(3_600).expect("within domain")
}

/// 86400 SI seconds.
///
/// **Not** one rotation of Earth (§8.3). Earth's rotation is a `Body` parameter,
/// is not exactly 86400 s, and lengthens by roughly 1.8 ms per century.
pub fn day_si() -> Ticks {
    scaled(86_400).expect("within domain")
}

/// 604800 SI seconds. The seven-day week has no astronomical period behind it
/// (§8.6); this is a duration, not a calendar unit.
pub fn week_si() -> Ticks {
    scaled(604_800).expect("within domain")
}

/// The Julian year: 31 557 600 SI seconds, exact by definition.
pub fn year_julian() -> Ticks {
    scaled(31_557_600).expect("within domain")
}

/// The mean Gregorian year: 31 556 952 SI seconds.
pub fn year_gregorian_mean() -> Ticks {
    scaled(31_556_952).expect("within domain")
}

// ---------------------------------------------------------------------------
// Time scales (Rule L)
// ---------------------------------------------------------------------------

/// Which time scale a civil label is expressed in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Scale {
    /// Terrestrial Time — the pivot. Uniform, no leap seconds (Rule L).
    #[default]
    Tt,
    /// International Atomic Time. `TT = TAI + 32.184 s`, exactly.
    Tai,
    /// Coordinated Universal Time. Has leap seconds, and its labels are
    /// **not unique** across one: `23:59:60` and the following `00:00:00` are
    /// distinct instants, while a naive reader sees two labels for what looks
    /// like the same moment. Absolute time has no such ambiguity.
    Utc,
}

impl Scale {
    /// Whether this scale is subject to leap seconds.
    pub const fn uses_leap_seconds(self) -> bool {
        matches!(self, Scale::Utc)
    }
}

/// `TT - TAI` in ticks. Exactly 32.184 s.
///
/// Exact because 32184 ms divides into ticks without remainder: `SECOND` is
/// divisible by 1000.
pub fn tt_minus_tai() -> Ticks {
    let (q, r) = second().quot_rem(&<Ticks as TickInt>::from_u64(1_000));
    debug_assert!(r.is_zero_ticks(), "SECOND must divide exactly by 1000");
    q.try_mul(&<Ticks as TickInt>::from_u64(TT_MINUS_TAI_MILLIS as u64))
        .expect("within domain")
}

/// `TAI - UTC` in ticks for a UTC label, across the whole history of UTC.
///
/// Three regimes, and the boundaries between them are part of the definition
/// rather than an implementation detail:
///
/// | period | offset |
/// |---|---|
/// | before 1961-01-01 | UTC does not exist — `UCAL-E0041` |
/// | 1961-01-01 to 1972-01-01 | piecewise-linear and fractional (see [`crate::rubber`]) |
/// | from 1972-01-01 | a whole number of seconds, stepped by leap seconds |
///
/// Every regime yields an exact tick count; none of them rounds.
fn utc_offset_ticks(
    days: i64,
    second_of_day: i64,
    year: i64,
    month: u8,
    day: u8,
    is_leap_label: bool,
) -> Result<Ticks> {
    if rubber::precedes_utc(days) {
        return Err(TimeError::with_context(
            Code::E0041,
            "UTC does not exist before 1961-01-01; use the TT or TAI scale",
        ));
    }
    if rubber::is_rubber_era(days) {
        if is_leap_label {
            return Err(TimeError::with_context(
                Code::E0042,
                "leap seconds began in 1972; the 1961-1972 era steered UTC by rate \
                 offsets and fractional steps instead",
            ));
        }
        return rubber::offset_ticks(days, second_of_day, &second());
    }
    let whole = tai_minus_utc(year, month, day, is_leap_label)?;
    second()
        .try_mul(&<Ticks as TickInt>::from_u64(whole.unsigned_abs()))
        .ok_or(TimeError::new(Code::E0021))
}

// ---------------------------------------------------------------------------
// SubSecond (§14.1)
// ---------------------------------------------------------------------------

/// An exact decimal fraction of a second, of at most thirty digits (§14.1).
///
/// Construction is exact or it fails; there is no rounding path into absolute
/// time (Rule R). Thirty is not arbitrary — it is the decimal scale of the bridge
/// constant, so it is precisely the point at which exactness runs out.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Default)]
pub struct SubSecond {
    /// The numerator, strictly less than `10^digits`.
    value: u128,
    /// Number of decimal places.
    digits: u8,
}

impl SubSecond {
    /// The most decimal places the bridge can represent exactly (D-3).
    pub const MAX_DIGITS: u8 = SECOND_DECIMAL_DIGITS;

    /// Zero.
    pub fn zero() -> SubSecond {
        SubSecond {
            value: 0,
            digits: 0,
        }
    }

    /// Construct from a numerator and a digit count.
    ///
    /// `UCAL-E0043` beyond thirty digits — the bridge cannot represent it, and
    /// Rule R forbids rounding on the way *in*.
    pub fn new(value: u128, digits: u8) -> Result<SubSecond> {
        if digits > Self::MAX_DIGITS {
            return Err(TimeError::with_context(
                Code::E0043,
                "sub-second input finer than 10^-30 s cannot be represented exactly",
            ));
        }
        let limit = 10u128
            .checked_pow(digits as u32)
            .ok_or(TimeError::new(Code::E0043))?;
        if value >= limit {
            return Err(TimeError::with_context(
                Code::E0041,
                "sub-second numerator must be below 10^digits",
            ));
        }
        Ok(SubSecond { value, digits })
    }

    /// Parse a fractional part such as `"5"`, `"000001"` or `".25"`.
    ///
    /// The digit count is taken from the string, so `"5"` is 0.5 s and `"05"` is
    /// 0.05 s. Trailing zeros are significant to the digit count but not to the
    /// value, which is why they round-trip.
    pub fn parse(s: &str) -> Result<SubSecond> {
        let s = s.strip_prefix('.').unwrap_or(s);
        if s.is_empty() {
            return Ok(SubSecond::zero());
        }
        if !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err(TimeError::with_context(
                Code::E0041,
                "sub-second must be decimal digits",
            ));
        }
        if s.len() > Self::MAX_DIGITS as usize {
            return Err(TimeError::with_context(
                Code::E0043,
                "sub-second input finer than 10^-30 s cannot be represented exactly",
            ));
        }
        let value: u128 = s
            .parse()
            .map_err(|_| TimeError::new(Code::E0041))?;
        SubSecond::new(value, s.len() as u8)
    }

    /// The numerator.
    pub fn value(&self) -> u128 {
        self.value
    }

    /// The number of decimal places.
    pub fn digits(&self) -> u8 {
        self.digits
    }

    /// Whether this is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.value == 0
    }

    /// The fraction in ticks. **Exact** — this is the claim §8.2 makes.
    ///
    /// `value x SECOND / 10^d = value x MANTISSA x 10^(30 - d)`, an integer for
    /// every `d <= 30`.
    pub fn ticks(&self) -> Result<Ticks> {
        if self.value == 0 {
            return Ok(<Ticks as TickInt>::zero());
        }
        let v = <Ticks as TickInt>::from_u128(self.value)
            .ok_or(TimeError::new(Code::E0021))?;
        let mantissa = <Ticks as TickInt>::from_u64(SECOND_MANTISSA);
        let scale = pow10((Self::MAX_DIGITS - self.digits) as u32)?;
        v.try_mul(&mantissa)
            .and_then(|x| x.try_mul(&scale))
            .ok_or(TimeError::new(Code::E0021))
    }

    /// The exact fraction as a rational.
    pub fn to_ratio(&self) -> Result<Ratio> {
        Ratio::new(
            <Ticks as TickInt>::from_u128(self.value).ok_or(TimeError::new(Code::E0021))?,
            pow10(self.digits as u32)?,
        )
    }

    /// Render as a decimal string of exactly `digits` places, without a leading
    /// point.
    #[cfg(feature = "alloc")]
    pub fn render(&self, digits: u8) -> alloc::string::String {
        use alloc::string::String;
        let mut s = String::new();
        let text = self.value.to_string();
        // Left-pad to the stored digit count, then adjust to the requested one.
        let padded = {
            let mut p = String::new();
            let width = self.digits as usize;
            if width > text.len() {
                for _ in 0..(width - text.len()) {
                    p.push('0');
                }
            }
            p.push_str(&text);
            p
        };
        for i in 0..digits as usize {
            s.push(padded.as_bytes().get(i).map(|b| *b as char).unwrap_or('0'));
        }
        s
    }
}

// ---------------------------------------------------------------------------
// Exact rational seconds since SI_EPOCH (§14)
// ---------------------------------------------------------------------------

/// An exact, signed count of TT seconds relative to `SI_EPOCH`.
///
/// Signed because `SI_EPOCH` is 31.22 deeps *after* the datum, so instants on
/// either side of it are perfectly ordinary. The sign here is an offset from an
/// Earth epoch — it says nothing about absolute time, which remains unsigned
/// (Rule Z, N12).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SiSeconds {
    negative: bool,
    magnitude: Ratio,
}

impl SiSeconds {
    /// Construct from a sign and an exact magnitude.
    pub fn new(negative: bool, magnitude: Ratio) -> SiSeconds {
        if magnitude.is_zero() {
            SiSeconds {
                negative: false,
                magnitude,
            }
        } else {
            SiSeconds {
                negative,
                magnitude,
            }
        }
    }

    /// Whether the offset is before `SI_EPOCH`.
    pub fn is_negative(&self) -> bool {
        self.negative
    }

    /// The exact magnitude in seconds.
    pub fn magnitude(&self) -> &Ratio {
        &self.magnitude
    }
}

/// Absolute time to exact TT seconds since `SI_EPOCH` (§14). Always exact.
pub fn to_si_seconds(t: &Instant<UC1>) -> Result<SiSeconds> {
    let origin = UC1::origin_offset();
    let (negative, delta) = if t.ticks() >= &origin {
        (false, t.ticks().try_sub(&origin).expect("t >= origin"))
    } else {
        (true, origin.try_sub(t.ticks()).expect("origin > t"))
    };
    Ok(SiSeconds::new(negative, Ratio::new(delta, second())?))
}

/// Exact TT seconds since `SI_EPOCH` to absolute time (§14).
///
/// `UCAL-E0043` when the denominator does not divide `SECOND` — that is input
/// finer than the bridge can represent, and Rule R forbids rounding it in.
/// `UCAL-E0020` when the result would precede the datum.
pub fn from_si_seconds(s: &SiSeconds) -> Result<Instant<UC1>> {
    let sec = second();
    // magnitude = num/den seconds; ticks = num * SECOND / den, exact iff den | num*SECOND.
    let (q, r) = sec.quot_rem(s.magnitude.denom());
    if !r.is_zero_ticks() {
        // Fall back to the general check: the product may still be exact.
        let (prod_q, prod_r) =
            ucal_core::num::mul_div(s.magnitude.numer(), &sec, s.magnitude.denom())?;
        if !prod_r.is_zero_ticks() {
            return Err(TimeError::with_context(
                Code::E0043,
                "denominator does not divide the bridge constant; input is finer \
                 than 10^-30 s and cannot be converted exactly",
            ));
        }
        return offset_from_origin(s.negative, prod_q);
    }
    let ticks = q
        .try_mul(s.magnitude.numer())
        .ok_or(TimeError::new(Code::E0021))?;
    offset_from_origin(s.negative, ticks)
}

fn offset_from_origin(negative: bool, magnitude: Ticks) -> Result<Instant<UC1>> {
    let origin = UC1::origin_offset();
    let ticks = if negative {
        origin.try_sub(&magnitude).ok_or(TimeError::with_context(
            Code::E0020,
            "no time exists before the datum",
        ))?
    } else {
        origin
            .try_add(&magnitude)
            .ok_or(TimeError::new(Code::E0021))?
    };
    Instant::from_ticks(ticks)
}

// ---------------------------------------------------------------------------
// Civil conversion (§14, §14.3)
// ---------------------------------------------------------------------------

/// Lowest civil year the bridge will render (§14.3).
///
/// Bounded by `i64` second arithmetic, not by the profile: the absolute domain
/// reaches 2.29x10^103 years, far past anything a civil calendar can label.
/// Exceeding this is `UCAL-E0040`, never a panic.
pub const CIVIL_YEAR_MIN: i64 = -100_000_000_000;

/// Highest civil year the bridge will render (§14.3).
pub const CIVIL_YEAR_MAX: i64 = 100_000_000_000;

/// A civil label, with everything needed to interpret it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CivilFields {
    /// Astronomical year numbering: `0` is 1 BC (§2.5).
    pub year: i64,
    /// Month, 1-12.
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Hour, 0-23.
    pub hour: u8,
    /// Minute, 0-59.
    pub minute: u8,
    /// Second, 0-60. **60 during a leap second** — §14.2 requires it be reported
    /// rather than normalised away.
    pub second: u8,
    /// The fractional second, as rendered.
    pub sub: SubSecond,
    /// Which scale this label is in.
    pub scale: Scale,
    /// Which calendar this label is in.
    pub calendar: CivilCalendar,
    /// Set when the rendering discarded sub-tick-level detail (Rule R).
    pub lossy: bool,
    /// Any warning that accompanies the value.
    pub warning: Option<Warning>,
}

/// Civil label to absolute time. **Exact** for any input the bridge accepts.
#[allow(clippy::too_many_arguments)]
pub fn from_civil(
    year: i64,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    sec: u8,
    sub: SubSecond,
    scale: Scale,
    cal: CivilCalendar,
) -> Result<Instant<UC1>> {
    if !(CIVIL_YEAR_MIN..=CIVIL_YEAR_MAX).contains(&year) {
        return Err(TimeError::with_context(
            Code::E0040,
            "civil year outside the renderable range",
        ));
    }
    check_date(year, month, day, cal)?;
    check_time(hour, minute, sec)?;

    // A 60th second is legal only where the table says one was inserted (Rule L).
    let is_leap_label = sec == 60;
    if is_leap_label {
        if scale != Scale::Utc {
            return Err(TimeError::with_context(
                Code::E0042,
                "second = 60 exists only in UTC; TT and TAI are uniform scales",
            ));
        }
        if hour != 23 || minute != 59 {
            return Err(TimeError::with_context(
                Code::E0042,
                "a leap second occurs only at 23:59:60",
            ));
        }
        if !has_leap_second(year, month, day) {
            return Err(TimeError::with_context(
                Code::E0042,
                "no leap second was inserted on this date",
            ));
        }
    }

    // Label-linear seconds since 0000-01-01 in the label's own calendar.
    let days = days_from_civil(year, month, day, cal);
    let linear = days
        .checked_mul(86_400)
        .and_then(|d| d.checked_add(hour as i64 * 3_600 + minute as i64 * 60 + sec as i64))
        .ok_or(TimeError::new(Code::E0040))?;

    // The label's own linear position, before any scale offset.
    let sec_ticks = second();
    let magnitude = <Ticks as TickInt>::from_u64(linear.unsigned_abs())
        .try_mul(&sec_ticks)
        .ok_or(TimeError::new(Code::E0021))?;
    let base = offset_from_origin(linear < 0, magnitude)?;

    // Lift to TT, in ticks rather than in whole seconds — the 1961-1972 era's
    // offset is fractional, so a seconds-valued offset could not represent it.
    // This is the only place leap seconds, or any UTC steering, are consulted
    // (Rule L).
    let base = match scale {
        Scale::Tt => base,
        Scale::Tai => base.checked_add(&ucal_core::Delta::from_ticks(tt_minus_tai()))?,
        Scale::Utc => {
            let sod = hour as i64 * 3_600 + minute as i64 * 60 + sec as i64;
            let offset = utc_offset_ticks(days, sod, year, month, day, is_leap_label)?;
            base.checked_add(&ucal_core::Delta::from_ticks(offset))?
                .checked_add(&ucal_core::Delta::from_ticks(tt_minus_tai()))?
        }
    };

    // The sub-second fraction is added forward, exactly.
    let sub_ticks = sub.ticks()?;
    base.checked_add(&ucal_core::Delta::from_ticks(sub_ticks))
}

/// Absolute time to a civil label.
///
/// Rounds only at the requested digit count, under an explicit mode, and reports
/// the loss (Rule R). `UCAL-E0040` outside the renderable range.
pub fn to_civil(
    t: &Instant<UC1>,
    scale: Scale,
    digits: u8,
    rounding: Rounding,
    cal: CivilCalendar,
) -> Result<CivilFields> {
    if digits > SubSecond::MAX_DIGITS {
        return Err(TimeError::with_context(
            Code::E0043,
            "cannot render more than thirty decimal places exactly",
        ));
    }

    // Step back to the scale's own zero point.
    let shifted = match scale {
        Scale::Tt => t.clone(),
        Scale::Tai | Scale::Utc => {
            t.checked_sub(&ucal_core::Delta::from_ticks(tt_minus_tai()))?
        }
    };

    let origin = UC1::origin_offset();
    let sec_ticks = second();
    let (negative, delta) = if shifted.ticks() >= &origin {
        (false, shifted.ticks().try_sub(&origin).expect("ge"))
    } else {
        (true, origin.try_sub(shifted.ticks()).expect("lt"))
    };
    let (whole, frac) = delta.quot_rem(&sec_ticks);
    let whole_u64: u64 = whole
        .to_dec_string()
        .parse()
        .map_err(|_| TimeError::with_context(Code::E0040, "instant outside the civil range"))?;

    // For a negative offset the fractional part runs forward from the second
    // *below*, so borrow one second when it is non-zero.
    let (signed_seconds, frac) = if negative {
        if frac.is_zero_ticks() {
            (-(whole_u64 as i64), frac)
        } else {
            (
                -(whole_u64 as i64) - 1,
                sec_ticks.try_sub(&frac).expect("frac < SECOND"),
            )
        }
    } else {
        (whole_u64 as i64, frac)
    };

    // Drop from TT to the label's scale (Rule L: only here).
    let mut warning = None;
    let mut frac = frac;
    let (linear, leap) = match scale {
        Scale::Tt | Scale::Tai => (signed_seconds, false),
        Scale::Utc => {
            // Which regime? Keyed on TAI, because the 1961-1972 era contains step
            // adjustments and the UTC label is ambiguous across one.
            let from_origin = if negative {
                None
            } else {
                shifted.ticks().try_sub(&origin)
            };
            let in_era = match &from_origin {
                Some(d) => rubber::covers_tai(d, &sec_ticks)?,
                None => false,
            };
            if in_era {
                let d = from_origin.as_ref().expect("checked above");
                let u = rubber::utc_linear_seconds_from_tai(d, &sec_ticks)?;
                let whole = u.floor();
                let whole_i: i64 = whole
                    .to_dec_string()
                    .parse()
                    .map_err(|_| TimeError::new(Code::E0040))?;
                // The fractional remainder becomes the sub-second field. It need
                // not be a whole tick — a rubber-era label is a projection of an
                // exact instant onto a scale that ran at a different rate — so
                // any remainder is rendered under the caller's mode (Rule R).
                let f = u.frac();
                let (q, r) =
                    ucal_core::num::mul_div(f.numer(), &sec_ticks, f.denom())?;
                if !r.is_zero_ticks() {
                    warning = Some(Warning::W0001);
                }
                frac = q;
                (whole_i, false)
            } else {
                let r = utc_from_tai_seconds(signed_seconds)?;
                (r.linear_seconds, r.in_leap_second)
            }
        }
    };

    let days = linear.div_euclid(86_400);
    let mut rem = linear.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days, cal);
    if !(CIVIL_YEAR_MIN..=CIVIL_YEAR_MAX).contains(&year) {
        return Err(TimeError::with_context(
            Code::E0040,
            "civil year outside the renderable range",
        ));
    }
    let hour = (rem / 3_600) as u8;
    rem %= 3_600;
    let minute = (rem / 60) as u8;
    let mut sec = (rem % 60) as u8;

    // §14.2: report the 60th second rather than normalising it.
    let (year, month, day, hour, minute) = if leap {
        let prev = civil_from_days(days - 1, cal);
        sec = 60;
        (prev.0, prev.1, prev.2, 23, 59)
    } else {
        (year, month, day, hour, minute)
    };

    if scale == Scale::Utc && !table_covers(year, month, day) {
        warning = Some(Warning::W0002);
    }

    // Render the fraction, rounding only here (Rule R).
    let (sub, lossy) = render_fraction(&frac, digits, rounding)?;
    if lossy && warning.is_none() {
        warning = Some(Warning::W0001);
    }

    Ok(CivilFields {
        year,
        month,
        day,
        hour,
        minute,
        second: sec,
        sub,
        scale,
        calendar: cal,
        lossy,
        warning,
    })
}

/// Render sub-second ticks to `digits` decimal places under an explicit mode.
///
/// Returns the fraction and whether anything was discarded. A tick is about
/// `5.39 x 10^-44` s, so a fraction is exactly representable in `d <= 30` decimal
/// places only when the tick count is divisible by `MANTISSA x 10^(30-d)`; almost
/// none are, and saying so is the point.
fn render_fraction(frac: &Ticks, digits: u8, rounding: Rounding) -> Result<(SubSecond, bool)> {
    if frac.is_zero_ticks() {
        return Ok((SubSecond::new(0, digits)?, false));
    }
    let sec_ticks = second();
    let scale = pow10(digits as u32)?;
    // value = round(frac * 10^digits / SECOND)
    let (q, r) = ucal_core::num::mul_div(frac, &scale, &sec_ticks)?;
    let up = match rounding {
        Rounding::Trunc => false,
        Rounding::Ceil => !r.is_zero_ticks(),
        Rounding::HalfUp | Rounding::HalfEven => {
            let twice = r.try_add(&r).ok_or(TimeError::new(Code::E0021))?;
            match twice.cmp(&sec_ticks) {
                core::cmp::Ordering::Greater => true,
                core::cmp::Ordering::Less => false,
                core::cmp::Ordering::Equal => match rounding {
                    Rounding::HalfUp => true,
                    _ => q.is_odd(),
                },
            }
        }
    };
    let mut value: u128 = q
        .to_dec_string()
        .parse()
        .map_err(|_| TimeError::new(Code::E0040))?;
    if up {
        value += 1;
    }
    let limit = 10u128
        .checked_pow(digits as u32)
        .ok_or(TimeError::new(Code::E0043))?;
    // Rounding up can carry into the whole second; clamp rather than corrupt the
    // label, and report the loss.
    let carried = value >= limit;
    if carried {
        value = limit - 1;
    }
    Ok((SubSecond::new(value, digits)?, !r.is_zero_ticks()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::backend::TickInt;

    fn tt(y: i64, mo: u8, d: u8, h: u8, mi: u8, s: u8) -> Instant<UC1> {
        from_civil(
            y,
            mo,
            d,
            h,
            mi,
            s,
            SubSecond::zero(),
            Scale::Tt,
            CivilCalendar::Gregorian,
        )
        .unwrap()
    }

    // ---- §8.3 units ----

    #[test]
    fn si_units_are_exact_multiples_of_the_bridge_constant() {
        let s = second();
        assert_eq!(minute(), s.try_mul(&<Ticks as TickInt>::from_u64(60)).unwrap());
        assert_eq!(hour(), s.try_mul(&<Ticks as TickInt>::from_u64(3600)).unwrap());
        assert_eq!(day_si(), s.try_mul(&<Ticks as TickInt>::from_u64(86400)).unwrap());
        assert_eq!(week_si(), s.try_mul(&<Ticks as TickInt>::from_u64(604800)).unwrap());
        assert_eq!(
            year_julian(),
            s.try_mul(&<Ticks as TickInt>::from_u64(31_557_600)).unwrap()
        );
        // A nanosecond is exact because SECOND is divisible by 10^9 (D-3).
        assert_eq!(
            nanosecond()
                .try_mul(&<Ticks as TickInt>::from_u64(1_000_000_000))
                .unwrap(),
            s
        );
    }

    #[test]
    fn tt_minus_tai_is_exactly_32_184_seconds() {
        let d = tt_minus_tai();
        // 32.184 s x 1000 = 32184 x (SECOND/1000)
        let thousandth = second().quot_rem(&<Ticks as TickInt>::from_u64(1000)).0;
        assert_eq!(d, thousandth.try_mul(&<Ticks as TickInt>::from_u64(32_184)).unwrap());
        // ...and it is less than 33 seconds and more than 32.
        assert!(d > second().try_mul(&<Ticks as TickInt>::from_u64(32)).unwrap());
        assert!(d < second().try_mul(&<Ticks as TickInt>::from_u64(33)).unwrap());
    }

    // ---- Appendix C fixtures, through the bridge ----

    #[test]
    fn reproduces_appendix_c_fixtures() {
        let cases: [((i64, u8, u8, u8, u8, u8), &str); 6] = [
            ((0, 1, 1, 0, 0, 0), "8070204002895596515944343085635637180530466139316558837890625"),
            ((-43, 3, 15, 0, 0, 0), "8070203977843789392286957152835637180530466139316558837890625"),
            ((1969, 7, 20, 20, 17, 40), "8070205155746435292175415045495637180530466139316558837890625"),
            ((1970, 1, 1, 0, 0, 0), "8070205156009508751803579616835637180530466139316558837890625"),
            ((2000, 1, 1, 12, 0, 0), "8070205173569972963515184424835637180530466139316558837890625"),
            ((2026, 7, 29, 0, 0, 0), "8070205189123984864657505252035637180530466139316558837890625"),
        ];
        for ((y, mo, d, h, mi, s), want) in cases {
            let got = tt(y, mo, d, h, mi, s);
            assert_eq!(
                got.ticks().to_dec_string(),
                want,
                "fixture {y}-{mo}-{d}T{h}:{mi}:{s}"
            );
            // ...and back.
            let f = to_civil(&got, Scale::Tt, 0, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
            assert_eq!((f.year, f.month, f.day, f.hour, f.minute, f.second), (y, mo, d, h, mi, s));
        }
    }

    #[test]
    fn si_epoch_is_the_bridge_anchor() {
        assert_eq!(tt(0, 1, 1, 0, 0, 0).ticks(), &UC1::origin_offset());
    }

    // ---- Rule Z at the bridge ----

    #[test]
    fn no_civil_label_can_precede_the_datum() {
        // "No time exists before that." The datum is the floor of the domain, and
        // a civil label far enough back names no instant at all — it fails,
        // rather than producing a negative or wrapped value (Rule Z, N12).
        //
        // The datum lies about 13.787 Gyr before SI_EPOCH, so the boundary is
        // near year -13,787,000,000.
        let err = from_civil(
            -20_000_000_000,
            1,
            1,
            0,
            0,
            0,
            SubSecond::zero(),
            Scale::Tt,
            CivilCalendar::Gregorian,
        )
        .unwrap_err();
        assert_eq!(err.code, Code::E0020);

        // Just inside the domain still works, and is a large positive tick count.
        let ok = from_civil(
            -13_000_000_000,
            1,
            1,
            0,
            0,
            0,
            SubSecond::zero(),
            Scale::Tt,
            CivilCalendar::Gregorian,
        )
        .unwrap();
        assert!(ok.ticks() > &<Ticks as TickInt>::zero());
        assert!(ok < tt(0, 1, 1, 0, 0, 0));

        // And every ordinary historical date is far above the floor.
        for y in [-4712i64, -43, 0, 1970, 2026] {
            let v = tt(y, 1, 1, 0, 0, 0);
            assert!(v.ticks() > &<Ticks as TickInt>::zero(), "year {y}");
        }
    }

    // ---- §14.1 SubSecond, and §8.2 exactness ----

    #[test]
    fn sub_second_is_exact_to_thirty_digits() {
        for d in 0..=30u8 {
            let value = if d == 0 { 0 } else { 1 };
            let ss = SubSecond::new(value, d).unwrap();
            let ticks = ss.ticks().unwrap();
            // value/10^d seconds, exactly: ticks * 10^d == value * SECOND
            let lhs = ticks.try_mul(&pow10(d as u32).unwrap()).unwrap();
            let rhs = <Ticks as TickInt>::from_u128(value)
                .unwrap()
                .try_mul(&second())
                .unwrap();
            assert_eq!(lhs, rhs, "digit count {d} was not exact");
        }
    }

    #[test]
    fn thirty_one_digits_is_e0043() {
        // The bridge carries exactly thirty decimal places; the thirty-first is
        // where Rule R's prohibition on rounding *in* becomes visible.
        assert_eq!(SubSecond::new(1, 31).unwrap_err().code, Code::E0043);
        assert_eq!(
            SubSecond::parse("1234567890123456789012345678901").unwrap_err().code,
            Code::E0043
        );
        // Thirty is fine.
        assert!(SubSecond::parse("123456789012345678901234567890").is_ok());
        assert_eq!(SubSecond::MAX_DIGITS, 30);
    }

    #[test]
    fn sub_second_parses_and_keeps_significance() {
        assert_eq!(SubSecond::parse("5").unwrap().to_ratio().unwrap(), Ratio::new(<Ticks as TickInt>::from_u64(1), <Ticks as TickInt>::from_u64(2)).unwrap());
        assert_eq!(SubSecond::parse("05").unwrap().digits(), 2);
        assert_eq!(SubSecond::parse(".25").unwrap().value(), 25);
        assert!(SubSecond::parse("").unwrap().is_zero());
        assert!(SubSecond::parse("abc").is_err());
    }

    #[test]
    fn a_whole_nanosecond_keeps_twenty_one_base5_zeros() {
        // §2.4, verified through the bridge rather than in the abstract.
        let base = tt(2026, 7, 29, 0, 0, 0);
        for n in 1..8u128 {
            let ss = SubSecond::new(n, 9).unwrap(); // n nanoseconds
            let v = base.checked_add(&ucal_core::Delta::from_ticks(ss.ticks().unwrap())).unwrap();
            let val = ucal_core::profile::base5_valuation(v.ticks());
            assert!(val >= 21, "nanosecond {n} gave v5 = {val}");
        }
    }

    // ---- Rule L: leap seconds only at the boundary ----

    #[test]
    fn leap_second_labels_round_trip() {
        // §14.2: to_civil must report sec = 60 rather than normalising.
        let t = from_civil(
            2016, 12, 31, 23, 59, 60,
            SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian,
        )
        .unwrap();
        let f = to_civil(&t, Scale::Utc, 0, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
        assert_eq!(
            (f.year, f.month, f.day, f.hour, f.minute, f.second),
            (2016, 12, 31, 23, 59, 60)
        );

        // The following instant is 2017-01-01T00:00:00 UTC, exactly one second on.
        let next = t.checked_add(&ucal_core::Delta::from_ticks(second())).unwrap();
        let f2 = to_civil(&next, Scale::Utc, 0, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
        assert_eq!(
            (f2.year, f2.month, f2.day, f2.hour, f2.minute, f2.second),
            (2017, 1, 1, 0, 0, 0)
        );
    }

    #[test]
    fn utc_labels_are_not_unique_but_absolute_time_is() {
        // Rule L requires this be documented; here it is measured. The two labels
        // 23:59:60 and 00:00:00 look adjacent, and are one second apart — but a
        // reader who ignores the leap second would compute zero.
        let leap = from_civil(2016, 12, 31, 23, 59, 60, SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian).unwrap();
        let midnight = from_civil(2017, 1, 1, 0, 0, 0, SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian).unwrap();
        assert_ne!(leap, midnight);
        assert_eq!(midnight.since(&leap).unwrap().ticks(), &second());
        // The naive label-linear difference is zero, which is the trap.
        let naive = days_from_civil(2017, 1, 1, CivilCalendar::Gregorian) * 86_400
            - (days_from_civil(2016, 12, 31, CivilCalendar::Gregorian) * 86_400 + 23 * 3600 + 59 * 60 + 60);
        assert_eq!(naive, 0);
    }

    #[test]
    fn a_sixtieth_second_is_rejected_where_none_was_inserted() {
        for (y, mo, d) in [(2024, 12, 31), (2020, 6, 30)] {
            let e = from_civil(y, mo, d, 23, 59, 60, SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian)
                .unwrap_err();
            assert_eq!(e.code, Code::E0042, "{y}-{mo}-{d}");
        }
        // ...and never in a uniform scale (Rule L).
        let e = from_civil(2016, 12, 31, 23, 59, 60, SubSecond::zero(), Scale::Tt, CivilCalendar::Gregorian)
            .unwrap_err();
        assert_eq!(e.code, Code::E0042);
    }

    #[test]
    fn leap_seconds_never_enter_absolute_arithmetic() {
        // Rule L's central claim: elapsed absolute time across a leap second is
        // the true elapsed time, and the scales differ only in their labels.
        let a = from_civil(2016, 12, 31, 0, 0, 0, SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian).unwrap();
        let b = from_civil(2017, 1, 1, 0, 0, 0, SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian).unwrap();
        let elapsed = b.since(&a).unwrap();
        // 86401 seconds: a day plus the inserted second.
        let expect = second().try_mul(&<Ticks as TickInt>::from_u64(86_401)).unwrap();
        assert_eq!(elapsed.ticks(), &expect);
        // The same two labels in TT are exactly 86400 s apart, because TT has no
        // leap seconds at all.
        let a_tt = tt(2016, 12, 31, 0, 0, 0);
        let b_tt = tt(2017, 1, 1, 0, 0, 0);
        assert_eq!(
            b_tt.since(&a_tt).unwrap().ticks(),
            &second().try_mul(&<Ticks as TickInt>::from_u64(86_400)).unwrap()
        );
    }

    #[test]
    fn tai_is_32_184_seconds_behind_tt() {
        let tai = from_civil(2000, 1, 1, 12, 0, 0, SubSecond::zero(), Scale::Tai, CivilCalendar::Gregorian).unwrap();
        let t_tt = tt(2000, 1, 1, 12, 0, 0);
        assert_eq!(tai.since(&t_tt).unwrap().ticks(), &tt_minus_tai());
    }

    // ---- §14.3 range ----

    #[test]
    fn out_of_range_years_are_e0040_not_a_panic() {
        let e = from_civil(
            CIVIL_YEAR_MAX + 1, 1, 1, 0, 0, 0,
            SubSecond::zero(), Scale::Tt, CivilCalendar::Gregorian,
        )
        .unwrap_err();
        assert_eq!(e.code, Code::E0040);
        let e = from_civil(
            CIVIL_YEAR_MIN - 1, 1, 1, 0, 0, 0,
            SubSecond::zero(), Scale::Tt, CivilCalendar::Gregorian,
        )
        .unwrap_err();
        assert_eq!(e.code, Code::E0040);
    }

    // ---- Rule R: rounding only on the way out ----

    #[test]
    fn rendering_rounds_and_reports_but_construction_never_does() {
        // One tick past a whole second: not representable in thirty decimal
        // places, so rendering must round and say so.
        let base = tt(2026, 7, 29, 0, 0, 0);
        let off = base.checked_add(&ucal_core::Delta::one_tick()).unwrap();
        let f = to_civil(&off, Scale::Tt, 30, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
        assert!(f.lossy, "one tick is far finer than 10^-30 s");
        assert_eq!(f.warning, Some(Warning::W0001));
        // Truncation gives all zeros; ceiling gives the last place.
        assert_eq!(f.sub.value(), 0);
        let f_up = to_civil(&off, Scale::Tt, 30, Rounding::Ceil, CivilCalendar::Gregorian).unwrap();
        assert_eq!(f_up.sub.value(), 1);

        // An exactly representable fraction is not lossy.
        let half = from_civil(
            2026, 7, 29, 0, 0, 0,
            SubSecond::parse("5").unwrap(), Scale::Tt, CivilCalendar::Gregorian,
        )
        .unwrap();
        let f = to_civil(&half, Scale::Tt, 1, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
        assert!(!f.lossy);
        assert_eq!(f.sub.value(), 5);
        assert_eq!(f.warning, None);
    }

    #[test]
    fn asking_for_more_than_thirty_digits_is_e0043() {
        let t = tt(2026, 7, 29, 0, 0, 0);
        assert_eq!(
            to_civil(&t, Scale::Tt, 31, Rounding::Trunc, CivilCalendar::Gregorian)
                .unwrap_err()
                .code,
            Code::E0043
        );
    }

    // ---- exact rational seconds ----

    #[test]
    fn si_seconds_round_trip_exactly() {
        for (y, mo, d, h, mi, s) in [
            (2026, 7, 29, 0, 0, 0),
            (0, 1, 1, 0, 0, 0),
            (-43, 3, 15, 0, 0, 0),
            (1969, 7, 20, 20, 17, 40),
        ] {
            let t = tt(y, mo, d, h, mi, s);
            let si = to_si_seconds(&t).unwrap();
            assert_eq!(from_si_seconds(&si).unwrap(), t, "{y}-{mo}-{d}");
        }
        // SI_EPOCH is exactly zero seconds.
        let si = to_si_seconds(&tt(0, 1, 1, 0, 0, 0)).unwrap();
        assert!(si.magnitude().is_zero());
        assert!(!si.is_negative());
        // Dates before SI_EPOCH give a negative *offset*, not a negative instant.
        let si = to_si_seconds(&tt(-43, 3, 15, 0, 0, 0)).unwrap();
        assert!(si.is_negative());
    }

    #[test]
    fn a_denominator_that_does_not_divide_the_bridge_is_e0043() {
        // 1/3 s is not representable: 3 does not divide 10^30.
        let third = Ratio::new(
            <Ticks as TickInt>::from_u64(1),
            <Ticks as TickInt>::from_u64(3),
        )
        .unwrap();
        let e = from_si_seconds(&SiSeconds::new(false, third)).unwrap_err();
        assert_eq!(e.code, Code::E0043);

        // A power-of-ten denominator up to 10^30 is fine.
        let tenth = Ratio::new(
            <Ticks as TickInt>::from_u64(1),
            <Ticks as TickInt>::from_u64(10),
        )
        .unwrap();
        assert!(from_si_seconds(&SiSeconds::new(false, tenth)).is_ok());
    }

    // ---- the Julian input calendar (§8.5) ----

    #[test]
    fn julian_input_differs_by_two_days_at_the_epoch() {
        let g = tt(0, 1, 1, 0, 0, 0);
        let j = from_civil(
            0, 1, 1, 0, 0, 0,
            SubSecond::zero(), Scale::Tt, CivilCalendar::Julian,
        )
        .unwrap();
        let two_days = day_si().try_mul(&<Ticks as TickInt>::from_u64(2)).unwrap();
        assert_eq!(g.since(&j).unwrap().ticks(), &two_days);

        // The historical Ides of March is a Julian date.
        let ides = from_civil(
            -43, 3, 15, 0, 0, 0,
            SubSecond::zero(), Scale::Tt, CivilCalendar::Julian,
        )
        .unwrap();
        let greg_same_label = tt(-43, 3, 15, 0, 0, 0);
        assert_eq!(greg_same_label.since(&ides).unwrap().ticks(), &two_days);
    }
}

#[cfg(test)]
mod rubber_era_tests {
    use super::*;
    use crate::calendar::days_from_gregorian;
    use ucal_core::backend::TickInt;

    fn utc(y: i64, mo: u8, d: u8, h: u8, mi: u8, s: u8) -> Result<Instant<UC1>> {
        from_civil(y, mo, d, h, mi, s, SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian)
    }

    #[test]
    fn pre_1972_utc_now_works() {
        // The question this era was added to answer: dates in 1961-1972 are
        // computable in UTC, not merely in TT and TAI.
        for (y, mo, d) in [(1961, 1, 1), (1963, 11, 1), (1965, 6, 15), (1968, 2, 1), (1971, 12, 31)] {
            let t = utc(y, mo, d, 12, 0, 0).unwrap_or_else(|e| panic!("{y}-{mo}-{d}: {e}"));
            assert!(t.ticks() > &<Ticks as TickInt>::zero());
        }
    }

    #[test]
    fn pre_1961_utc_is_still_refused() {
        // UTC did not exist. This is a statement about the scale, not a limit of
        // the arithmetic — the same instants are reachable in TT and TAI.
        let e = utc(1960, 12, 31, 0, 0, 0).unwrap_err();
        assert_eq!(e.code, Code::E0041);
        assert!(from_civil(
            1960, 12, 31, 0, 0, 0,
            SubSecond::zero(), Scale::Tt, CivilCalendar::Gregorian
        )
        .is_ok());
        assert!(from_civil(
            1960, 12, 31, 0, 0, 0,
            SubSecond::zero(), Scale::Tai, CivilCalendar::Gregorian
        )
        .is_ok());
    }

    #[test]
    fn the_era_round_trips() {
        for (y, mo, d, h, mi, s) in [
            (1961, 1, 1, 0, 0, 0),
            (1962, 7, 4, 6, 30, 15),
            (1964, 9, 1, 23, 59, 59),
            (1966, 1, 1, 0, 0, 1),
            (1969, 7, 20, 20, 17, 40),
            (1971, 12, 31, 23, 59, 59),
        ] {
            let t = utc(y, mo, d, h, mi, s).unwrap();
            let f = to_civil(&t, Scale::Utc, 9, Rounding::HalfEven, CivilCalendar::Gregorian)
                .unwrap_or_else(|e| panic!("{y}-{mo}-{d}: {e}"));
            assert_eq!(
                (f.year, f.month, f.day, f.hour, f.minute, f.second),
                (y, mo, d, h, mi, s),
                "round trip failed for {y}-{mo}-{d}T{h}:{mi}:{s}"
            );
        }
    }

    #[test]
    fn the_offset_is_fractional_and_grows_within_a_day() {
        // What made these seconds "rubber": the offset drifts continuously, so
        // two instants a day apart differ by slightly more than 86400 s.
        let a = utc(1965, 6, 15, 0, 0, 0).unwrap();
        let b = utc(1965, 6, 16, 0, 0, 0).unwrap();
        let elapsed = b.since(&a).unwrap();
        let day = second().try_mul(&<Ticks as TickInt>::from_u64(86_400)).unwrap();
        assert!(
            elapsed.ticks() > &day,
            "a rubber-era UTC day is longer than 86400 SI seconds"
        );
        // The excess is the daily rate, 0.001296 s.
        let excess = elapsed.ticks().try_sub(&day).unwrap();
        let (want, r) = ucal_core::num::mul_div(
            &<Ticks as TickInt>::from_u64(1_296),
            &second(),
            &<Ticks as TickInt>::from_u64(1_000_000),
        )
        .unwrap();
        assert!(r.is_zero_ticks());
        assert_eq!(excess, want, "the daily drift must be exactly 0.001296 s");
    }

    #[test]
    fn the_1972_discontinuity_is_visible_and_exact() {
        // The last rubber instant and the first modern one are 0.107758 s closer
        // together than their labels suggest, because the definition stepped.
        let last = utc(1971, 12, 31, 23, 59, 59).unwrap();
        let first = utc(1972, 1, 1, 0, 0, 0).unwrap();
        let elapsed = first.since(&last).unwrap();
        let one = second();
        let excess = elapsed.ticks().try_sub(&one).unwrap();

        // Two different quantities live near this boundary, and they differ:
        //
        //   0.107758    s  the discontinuity in the *definition*, at the boundary
        //                  instant (asserted in `rubber::the_famous_step_...`)
        //   0.10775803  s  the gap actually observed from 23:59:59, which adds
        //                  the rate term's drift over that second,
        //                  0.002592 / 86400 = 3e-8 s
        //
        // Both are whole numbers of ticks. Conflating them is easy and wrong, so
        // the observable one is pinned here explicitly.
        assert_eq!(
            excess.to_dec_string(),
            "1998758914217753633830000000000000000000000",
            "observed excess over one labelled second"
        );

        // The drift accounts for exactly the difference between the two.
        let boundary_step = <Ticks as TickInt>::from_dec_str(
            "1998758357760221638000000000000000000000000",
        )
        .unwrap();
        let drift = excess.try_sub(&boundary_step).unwrap();
        let (want, r) = ucal_core::num::mul_div(
            &<Ticks as TickInt>::from_u64(3),
            &second(),
            &<Ticks as TickInt>::from_u64(100_000_000),
        )
        .unwrap();
        assert!(r.is_zero_ticks());
        assert_eq!(drift, want, "drift over one second must be 3e-8 s");
    }

    #[test]
    fn era_boundaries_select_the_right_regime() {
        // 1972-01-01 is modern UTC: the offset is exactly 10 s.
        let t = utc(1972, 1, 1, 0, 0, 0).unwrap();
        let tt = from_civil(
            1972, 1, 1, 0, 0, 0,
            SubSecond::zero(), Scale::Tai, CivilCalendar::Gregorian,
        )
        .unwrap();
        let ten = second().try_mul(&<Ticks as TickInt>::from_u64(10)).unwrap();
        assert_eq!(t.since(&tt).unwrap().ticks(), &ten);

        // 1961-01-01 is the era's first instant: exactly 1.4228180 s.
        let t = utc(1961, 1, 1, 0, 0, 0).unwrap();
        let tai = from_civil(
            1961, 1, 1, 0, 0, 0,
            SubSecond::zero(), Scale::Tai, CivilCalendar::Gregorian,
        )
        .unwrap();
        assert_eq!(
            t.since(&tai).unwrap().ticks().to_dec_string(),
            "26391259758641428298000000000000000000000000"
        );
    }

    #[test]
    fn leap_labels_are_refused_in_the_rubber_era() {
        // Leap seconds began in 1972; the earlier era steered UTC differently.
        let days = days_from_gregorian(1965, 6, 30);
        assert!(crate::rubber::is_rubber_era(days));
        let e = from_civil(
            1965, 6, 30, 23, 59, 60,
            SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian,
        )
        .unwrap_err();
        assert_eq!(e.code, Code::E0042);
    }
}
