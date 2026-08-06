//! Text codecs (§6, Appendix F) — the two forms of one value (Rule D).
//!
//! # What a printed form means
//!
//! §6.1 is explicit: *the last group's tier is the stated precision*. A rendering
//! therefore denotes the closed interval `[v, v + 5^e - 1]` of Rule T, and a
//! rendering that stops at T0 denotes a beat, not a tick.
//!
//! Appendix C reads the other way — it prints SI_EPOCH to T0 and annotates "all
//! tiers below T0 are zero", treating omission as exactness. The two readings
//! cannot both hold, and this implementation takes §6.1's, because the
//! alternative *is* failure mode F2: if trailing omission meant "exact", then
//! reading a T-5 form would zero-fill it to a tick, which is precision invented
//! by zero-filling a truncated timestamp. See `spec/SPEC-DELTAS.md` D-A8.
//!
//! The practical consequence: **tick-exact text runs down to T-12.** For an
//! instant at a whole SI second the last six groups are guaranteed zero by §2.4,
//! which is exactly what tempts one to drop them — but dropping them changes what
//! the string says.
//!
//! # How a form is anchored
//!
//! Neither form states which tier it starts at, so each needs an anchor, and they
//! use different ones:
//!
//! - **Human form** anchors at the *bottom of the whole part*: the group
//!   immediately before `:` is T0, and groups after it run T-1 downward (§6.4).
//!   With no `:`, the last group is T0. This is what makes
//!   `UC1 0031·0687·2481·2999·3108·2437` unambiguous.
//! - **Digit form** anchors at the *top*: it always begins at T32, the highest
//!   tier the domain holds. The group count then fixes the precision. This is
//!   what lets D-9 call the digit form canonical for parse and sort, and what
//!   satisfies Rule S — a fixed tier width is the only condition under which
//!   lexicographic order on text equals chronological order.

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};
#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use crate::error::{Code, Result, TimeError};
use crate::tier::Tier;
#[cfg(feature = "alloc")]
use crate::backend::{TickInt, Ticks};
#[cfg(feature = "alloc")]
use crate::tier::GROUP_BASE;
use crate::locale::LocaleId;
#[cfg(feature = "alloc")]
use crate::profile::Profile;
#[cfg(feature = "alloc")]
use crate::tier::{K_MAX, K_MIN, TIER_COUNT};
use crate::locale;

#[doc(inline)]
pub use crate::locale::LocaleId as LocaleIdAlias;
use crate::value::Precision;
#[cfg(feature = "alloc")]
use crate::value::{Instant, Window};

/// §13 names a distinct parse error type; it is the crate's error type, because
/// the Appendix E codes are the contract.
pub type ParseError = TimeError;

/// The default group separator, U+00B7 MIDDLE DOT (§6.3, D-10).
pub const SEP: char = '·';

/// Always accepted on input, for shell-hostile contexts (§6.3, D-10).
pub const ALT_SEP: char = '.';

/// Introduces the sub-beat part (§6.4).
pub const SUB_SEP: char = ':';

/// Decimal digits per group in the human form: `3124` is four characters.
pub const HUMAN_GROUP_WIDTH: usize = 4;

/// Base-5 digits per group in the digit form (Rule G).
pub const DIGIT5_GROUP_WIDTH: usize = 5;

/// Which of the two forms (Rule D), plus the parseable-but-not-canonical named
/// form of §6.5.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Form {
    /// Decimal group values, most significant tier first. Tagged `UC1`.
    #[default]
    HumanGroups,
    /// Base-5 digits, five per group. Tagged `UC1/5`. Canonical for parse and
    /// sort (D-9).
    Digit5,
    /// `31 deep, 687 drift, ...`. Parseable, not canonical (§6.5).
    Named,
}

/// Formatting context (§13).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Fmt {
    /// Which form to render.
    pub form: Form,
    /// Group separator. Must not be a decimal or base-5 digit (§6.3).
    pub sep: char,
    /// Sub-beat introducer.
    pub sub_sep: char,
    /// The tier to render down to. This *is* the stated precision.
    pub precision: Precision,
    /// Pad the high end to a fixed tier width, which is the only condition under
    /// which text sorts chronologically (Rule S).
    pub pad: bool,
    /// Locale for the named form.
    pub locale: LocaleId,
}

impl Default for Fmt {
    fn default() -> Self {
        Fmt {
            form: Form::HumanGroups,
            sep: SEP,
            sub_sep: SUB_SEP,
            precision: Precision::Tick,
            pad: false,
            locale: LocaleId::En,
        }
    }
}

impl Fmt {
    // ---------------------------------------------------------------- builder
    //
    // `Fmt` is `#[non_exhaustive]`, so a caller assembles one from `default()`
    // or a named preset rather than with a struct literal. The fields stay
    // public to *read*.
    //
    // The alternative was to leave the type open and accept that any new
    // rendering option is a breaking change. A caller writing
    // `Fmt { .., ..Fmt::default() }` would have survived that, but one writing
    // the exhaustive literal would not — and which of the two a caller wrote is
    // not something this crate can influence. The builder makes the safe form
    // the only form.

    /// Which text form to render.
    pub const fn with_form(mut self, form: Form) -> Fmt {
        self.form = form;
        self
    }

    /// Group separator. Must not be a decimal or base-5 digit (§6.3).
    pub const fn with_sep(mut self, sep: char) -> Fmt {
        self.sep = sep;
        self
    }

    /// Sub-beat introducer.
    pub const fn with_sub_sep(mut self, sub_sep: char) -> Fmt {
        self.sub_sep = sub_sep;
        self
    }

    /// The tier to render down to. This *is* the stated precision (Rule T).
    pub const fn with_precision(mut self, precision: Precision) -> Fmt {
        self.precision = precision;
        self
    }

    /// Pad the high end to a fixed tier width — the only condition under which
    /// text sorts chronologically (Rule S).
    pub const fn with_pad(mut self, pad: bool) -> Fmt {
        self.pad = pad;
        self
    }

    /// Locale for the named form. Display only (Rule N).
    pub const fn with_locale(mut self, locale: LocaleId) -> Fmt {
        self.locale = locale;
        self
    }

    /// Human form at tick precision.
    pub fn human() -> Fmt {
        Fmt::default()
    }

    /// Human form truncated to a tier. The result denotes a window (Rule T).
    pub fn human_at(tier: Tier) -> Fmt {
        Fmt {
            precision: Precision::Tier(tier),
            ..Fmt::default()
        }
    }

    /// Canonical digit form: fixed width from T32, tick precision, sortable.
    ///
    /// Uses `.` rather than the `·` default, matching §6.2's printed example.
    /// Both separators parse either form (§6.3), so this is presentation only.
    pub fn digit5() -> Fmt {
        Fmt {
            form: Form::Digit5,
            sep: ALT_SEP,
            pad: true,
            ..Fmt::default()
        }
    }

    /// Named form.
    pub fn named() -> Fmt {
        Fmt {
            form: Form::Named,
            ..Fmt::default()
        }
    }

    /// Validate the separator choices (§6.3).
    ///
    /// A separator that is also a digit would make the notation ambiguous, so
    /// this is checked rather than assumed.
    pub fn validate(&self) -> Result<()> {
        for (c, what) in [(self.sep, "separator"), (self.sub_sep, "sub-separator")] {
            if c.is_ascii_digit() {
                return Err(TimeError::with_context(
                    Code::E0001,
                    "separator must not be a decimal or base-5 digit (§6.3)",
                ));
            }
            let _ = what;
        }
        if self.sep == self.sub_sep {
            return Err(TimeError::with_context(
                Code::E0001,
                "separator and sub-separator must differ",
            ));
        }
        Ok(())
    }

    #[cfg(feature = "alloc")]
    fn low_tier(&self) -> Tier {
        self.precision.tier()
    }
}

// ---------------------------------------------------------------------------
// Appendix F — the group codec
// ---------------------------------------------------------------------------

/// Decimal group values for tiers `k_hi` down to `k_lo`, most significant first.
///
/// Appendix F: repeated `divmod` by `5^5 = 3125`, one group per step. Digit-by-
/// digit division by 5 MUST NOT be used — it would take 221 steps at full width
/// instead of 45.
#[cfg(feature = "alloc")]
pub fn encode_groups(t: &Ticks, k_hi: Tier, k_lo: Tier) -> Result<Vec<u16>> {
    if k_hi < k_lo {
        return Err(TimeError::with_context(
            Code::E0006,
            "group range must descend",
        ));
    }
    let gb = <Ticks as TickInt>::from_u64(GROUP_BASE as u64);
    let (mut x, _) = t.quot_rem(&k_lo.ticks());
    let n = (k_hi.index() as isize - k_lo.index() as isize + 1) as usize;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        let (q, r) = x.quot_rem(&gb);
        out.push(group_to_u16(&r));
        x = q;
    }
    out.reverse();
    Ok(out)
}

/// Reassemble a tick count from group values (Appendix F).
#[cfg(feature = "alloc")]
pub fn decode_groups(groups: &[u16], k_lo: Tier) -> Result<Ticks> {
    let gb = <Ticks as TickInt>::from_u64(GROUP_BASE as u64);
    let mut acc = <Ticks as TickInt>::zero();
    for g in groups {
        if *g >= GROUP_BASE {
            return Err(TimeError::new(Code::E0004));
        }
        acc = acc
            .try_mul(&gb)
            .and_then(|v| v.try_add(&<Ticks as TickInt>::from_u64(*g as u64)))
            .ok_or(TimeError::new(Code::E0021))?;
    }
    acc.try_mul(&k_lo.ticks())
        .ok_or(TimeError::new(Code::E0021))
}

/// A group value is always below 3125, so it fits `u16`. Converted through the
/// canonical bytes so no backend-specific cast is needed.
#[cfg(feature = "alloc")]
fn group_to_u16(r: &Ticks) -> u16 {
    let b = r.to_canonical_bytes();
    u16::from_be_bytes([b[b.len() - 2], b[b.len() - 1]])
}

/// The five base-5 digits of a group, most significant first (Appendix F).
pub fn digit5_of_group(g: u16) -> [u8; DIGIT5_GROUP_WIDTH] {
    let mut out = [b'0'; DIGIT5_GROUP_WIDTH];
    let mut v = g;
    for i in (0..DIGIT5_GROUP_WIDTH).rev() {
        out[i] = b'0' + (v % 5) as u8;
        v /= 5;
    }
    out
}

/// Inverse of [`digit5_of_group`]. `UCAL-E0005` on a digit outside `0..=4`.
pub fn group_of_digit5(s: &str) -> Result<u16> {
    if s.len() != DIGIT5_GROUP_WIDTH {
        return Err(TimeError::with_context(
            Code::E0005,
            "a base-5 group is exactly five digits",
        ));
    }
    let mut v: u16 = 0;
    for c in s.bytes() {
        if !(b'0'..=b'4').contains(&c) {
            return Err(TimeError::new(Code::E0005));
        }
        v = v * 5 + (c - b'0') as u16;
    }
    Ok(v)
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

/// Render an instant (§6, Rule D).
#[cfg(feature = "alloc")]
pub fn render<P: Profile>(v: &Instant<P>, f: &Fmt) -> Result<String> {
    f.validate()?;
    let k_lo = f.low_tier();
    match f.form {
        Form::HumanGroups => render_human::<P>(v, f, k_lo),
        Form::Digit5 => render_digit5::<P>(v, f, k_lo),
        Form::Named => render_named::<P>(v, f, k_lo),
    }
}

/// The highest tier that must be shown: the highest non-zero one, but never
/// below T0, because the human form's anchor is the group before the `:`.
#[cfg(feature = "alloc")]
fn human_high_tier<P: Profile>(v: &Instant<P>, pad: bool) -> Tier {
    if pad {
        return Tier::new(K_MAX).expect("K_MAX is on the grid");
    }
    let mut hi = Tier::BEAT;
    for k in (1..=K_MAX).rev() {
        let t = Tier::new(k).expect("in range");
        if v.tier_value(t) != 0 {
            hi = t;
            break;
        }
    }
    hi
}

#[cfg(feature = "alloc")]
fn render_human<P: Profile>(v: &Instant<P>, f: &Fmt, k_lo: Tier) -> Result<String> {
    // The human form's only anchor is §6.4's sub-beat separator, which fixes the
    // group before it as T0. A rendering that stopped above T0 would therefore be
    // read back as though its last group *were* T0, silently changing the value
    // by whole tiers. Rather than invent syntax the RFC does not define, the
    // human form covers T0 and finer; the digit form (anchored at T32) and the
    // named form (self-describing) cover the whole grid. See D-A8.
    if k_lo.index() > 0 {
        return Err(TimeError::with_context(
            Code::E0006,
            "the human form anchors at T0 and cannot state a coarser precision; \
             use the digit form or the named form",
        ));
    }
    let k_hi = human_high_tier(v, f.pad);
    let whole_lo = Tier::BEAT;
    let mut s = String::new();
    s.push_str(P::TAG);
    s.push(' ');

    let whole = encode_groups(v.ticks(), k_hi, whole_lo)?;
    push_joined(&mut s, &whole, f.sep, |g| {
        let mut b = [b'0'; HUMAN_GROUP_WIDTH];
        write_dec4(g, &mut b);
        b
    });

    if k_lo.index() < 0 {
        s.push(f.sub_sep);
        let sub = encode_groups(
            v.ticks(),
            Tier::new(-1).expect("on the grid"),
            k_lo,
        )?;
        push_joined(&mut s, &sub, f.sep, |g| {
            let mut b = [b'0'; HUMAN_GROUP_WIDTH];
            write_dec4(g, &mut b);
            b
        });
    }
    Ok(s)
}

#[cfg(feature = "alloc")]
fn render_digit5<P: Profile>(v: &Instant<P>, f: &Fmt, k_lo: Tier) -> Result<String> {
    // Anchored at the top: always begins at T32, so the group count fixes the
    // precision and the width is constant for a given precision (Rule S).
    let k_hi = Tier::new(K_MAX).expect("K_MAX is on the grid");
    let groups = encode_groups(v.ticks(), k_hi, k_lo)?;
    let mut s = String::new();
    s.push_str(P::TAG);
    s.push_str("/5 ");
    push_joined(&mut s, &groups, f.sep, digit5_of_group);
    Ok(s)
}

#[cfg(feature = "alloc")]
fn render_named<P: Profile>(v: &Instant<P>, f: &Fmt, k_lo: Tier) -> Result<String> {
    // When the requested precision is coarser than the value's own magnitude the
    // value floors to zero at that tier, and the honest rendering is `0 drift` —
    // not an empty string. Clamping the high end up to the precision keeps the
    // stated tier present, which is what carries the precision (§6.1).
    let k_hi = {
        let h = human_high_tier(v, f.pad);
        if h.index() < k_lo.index() {
            k_lo
        } else {
            h
        }
    };
    let mut parts: Vec<String> = Vec::new();
    let mut k = k_hi.index();
    while k >= k_lo.index() {
        let t = Tier::new(k)?;
        let g = v.tier_value(t);
        // Zero groups are omitted except the last, which carries the precision.
        if g != 0 || k == k_lo.index() {
            // Rule N: a named tier prints its locale name, an unnamed one its
            // T[k] form, which is accepted wherever a name is.
            let mut p = g.to_string();
            p.push(' ');
            p.push_str(&locale::display(f.locale, t));
            parts.push(p);
        }
        k -= 1;
    }
    let _ = f;
    Ok(parts.join(", "))
}

#[cfg(feature = "alloc")]
fn push_joined<F, const N: usize>(s: &mut String, groups: &[u16], sep: char, render_one: F)
where
    F: Fn(u16) -> [u8; N],
{
    for (i, g) in groups.iter().enumerate() {
        if i > 0 {
            s.push(sep);
        }
        let b = render_one(*g);
        s.push_str(core::str::from_utf8(&b).expect("ascii digits"));
    }
}

#[cfg(feature = "alloc")]
fn write_dec4(g: u16, out: &mut [u8; HUMAN_GROUP_WIDTH]) {
    let mut v = g;
    for i in (0..HUMAN_GROUP_WIDTH).rev() {
        out[i] = b'0' + (v % 10) as u8;
        v /= 10;
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// Parse either text form, returning the value **and its stated precision**.
///
/// Rule T: a truncated notation never yields a bare tick-precision instant. The
/// returned [`Precision`] is the tier of the last group, and
/// [`parse_window`] turns it into the interval the notation actually denotes.
#[cfg(feature = "alloc")]
pub fn parse<P: Profile>(s: &str, ctx: &Fmt) -> core::result::Result<(Instant<P>, Precision), ParseError> {
    let s = s.trim();
    let (tag, rest) = split_tag(s)?;

    let digit_tag = {
        let mut t = String::from(P::TAG);
        t.push_str("/5");
        t
    };
    if tag == digit_tag {
        parse_digit5::<P>(rest, ctx)
    } else if tag == P::TAG {
        // The named form has no digits-only groups; detect it by its separator.
        if rest.contains(',') || rest.bytes().any(|b| b.is_ascii_alphabetic()) {
            parse_named::<P>(rest, ctx)
        } else {
            parse_human::<P>(rest, ctx)
        }
    } else {
        Err(TimeError::new(Code::E0002))
    }
}

/// Parse and immediately materialise the interval the notation denotes (Rule T).
#[cfg(feature = "alloc")]
pub fn parse_window<P: Profile>(s: &str, ctx: &Fmt) -> Result<Window<P>> {
    let (v, p) = parse::<P>(s, ctx)?;
    v.window_at(p)
}

#[cfg(feature = "alloc")]
fn split_tag(s: &str) -> core::result::Result<(&str, &str), ParseError> {
    match s.split_once(char::is_whitespace) {
        None => Err(TimeError::with_context(
            Code::E0001,
            "missing profile tag; every serialised form carries one (Rule P)",
        )),
        Some((tag, rest)) => Ok((tag, rest.trim())),
    }
}

#[cfg(feature = "alloc")]
fn is_sep(c: char, ctx: &Fmt) -> bool {
    c == ctx.sep || c == ALT_SEP || c == SEP
}

#[cfg(feature = "alloc")]
fn split_groups<'a>(s: &'a str, ctx: &Fmt) -> Vec<&'a str> {
    s.split(|c| is_sep(c, ctx))
        .filter(|p| !p.is_empty())
        .collect()
}

#[cfg(feature = "alloc")]
fn parse_human<P: Profile>(
    rest: &str,
    ctx: &Fmt,
) -> core::result::Result<(Instant<P>, Precision), ParseError> {
    let (whole_str, sub_str) = match rest.split_once(ctx.sub_sep) {
        None => (rest, None),
        Some((w, s)) => (w, Some(s)),
    };

    let whole_parts = split_groups(whole_str, ctx);
    if whole_parts.is_empty() {
        return Err(TimeError::new(Code::E0001));
    }
    let mut groups: Vec<u16> = Vec::new();
    for p in &whole_parts {
        groups.push(parse_human_group(p)?);
    }
    // Anchor: the last whole group is T0.
    let k_hi = whole_parts.len() as i32 - 1;
    if k_hi > K_MAX as i32 {
        return Err(TimeError::with_context(
            Code::E0006,
            "more whole groups than the tier grid holds",
        ));
    }

    let mut k_lo = 0i32;
    if let Some(sub) = sub_str {
        let sub_parts = split_groups(sub, ctx);
        if sub_parts.is_empty() {
            return Err(TimeError::with_context(
                Code::E0001,
                "sub-beat separator with no groups after it",
            ));
        }
        for p in &sub_parts {
            groups.push(parse_human_group(p)?);
        }
        k_lo = -(sub_parts.len() as i32);
        if k_lo < K_MIN as i32 {
            return Err(TimeError::with_context(
                Code::E0006,
                "more sub-beat groups than the tier grid holds",
            ));
        }
    }

    let low = Tier::new(k_lo as i8)?;
    let ticks = decode_groups(&groups, low)?;
    let v = Instant::<P>::from_ticks(ticks)?;
    Ok((v, precision_of(low)))
}

#[cfg(feature = "alloc")]
fn parse_human_group(p: &str) -> core::result::Result<u16, ParseError> {
    // A five-character group in a decimal-tagged string is the digit form:
    // the two forms must not be mixed (Rule D, UCAL-E0003).
    if p.len() == DIGIT5_GROUP_WIDTH && p.bytes().all(|b| (b'0'..=b'4').contains(&b)) {
        return Err(TimeError::with_context(
            Code::E0003,
            "base-5 group in a decimal-tagged string",
        ));
    }
    if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
        return Err(TimeError::new(Code::E0001));
    }
    // Parse before checking the width, so that an out-of-range group reports
    // E0004 ("group value out of range") rather than the vaguer E0001. Extra
    // leading zeros are tolerated on input; §6.1's four-digit padding is a
    // rendering rule, not a parsing one.
    let v: u32 = p.parse().map_err(|_| TimeError::new(Code::E0004))?;
    if v >= GROUP_BASE as u32 {
        return Err(TimeError::new(Code::E0004));
    }
    Ok(v as u16)
}

#[cfg(feature = "alloc")]
fn parse_digit5<P: Profile>(
    rest: &str,
    ctx: &Fmt,
) -> core::result::Result<(Instant<P>, Precision), ParseError> {
    if rest.contains(ctx.sub_sep) {
        return Err(TimeError::with_context(
            Code::E0003,
            "the digit form is anchored at T32 and takes no sub-beat separator",
        ));
    }
    let parts = split_groups(rest, ctx);
    if parts.is_empty() {
        return Err(TimeError::new(Code::E0001));
    }
    if parts.len() > TIER_COUNT {
        return Err(TimeError::with_context(
            Code::E0006,
            "more groups than the tier grid holds",
        ));
    }
    let mut groups = Vec::with_capacity(parts.len());
    for p in &parts {
        if p.len() == HUMAN_GROUP_WIDTH {
            return Err(TimeError::with_context(
                Code::E0003,
                "decimal group in a base-5-tagged string",
            ));
        }
        groups.push(group_of_digit5(p)?);
    }
    // Anchor: the first group is T32, so the count fixes the precision.
    let k_lo = K_MAX as i32 - (parts.len() as i32 - 1);
    let low = Tier::new(k_lo as i8)?;
    let ticks = decode_groups(&groups, low)?;
    let v = Instant::<P>::from_ticks(ticks)?;
    Ok((v, precision_of(low)))
}

#[cfg(feature = "alloc")]
fn parse_named<P: Profile>(
    rest: &str,
    _ctx: &Fmt,
) -> core::result::Result<(Instant<P>, Precision), ParseError> {
    let mut acc = <Ticks as TickInt>::zero();
    let mut lowest: Option<Tier> = None;
    let mut prev: Option<i8> = None;
    for term in rest.split(',') {
        let term = term.trim();
        if term.is_empty() {
            continue;
        }
        let (n_str, name) = term
            .split_once(char::is_whitespace)
            .ok_or(TimeError::with_context(Code::E0001, "expected `<count> <tier>`"))?;
        let count: u64 = n_str
            .trim()
            .parse()
            .map_err(|_| TimeError::new(Code::E0001))?;
        if count >= GROUP_BASE as u64 {
            return Err(TimeError::new(Code::E0004));
        }
        let tier = resolve_tier_name(name.trim())?;
        // Descending order is required so that the precision is the last term.
        if let Some(p) = prev {
            if tier.index() >= p {
                return Err(TimeError::with_context(
                    Code::E0006,
                    "named terms must descend",
                ));
            }
        }
        prev = Some(tier.index());
        lowest = Some(tier);
        let add = tier
            .ticks()
            .try_mul(&<Ticks as TickInt>::from_u64(count))
            .ok_or(TimeError::new(Code::E0021))?;
        acc = acc.try_add(&add).ok_or(TimeError::new(Code::E0021))?;
    }
    let low = lowest.ok_or(TimeError::new(Code::E0001))?;
    let v = Instant::<P>::from_ticks(acc)?;
    Ok((v, precision_of(low)))
}

/// Resolve a tier by name in the default locale, `T<k>`, or `5^e` (Rule N).
pub fn resolve_tier_name(s: &str) -> Result<Tier> {
    locale::resolve(LocaleId::default(), s)
}

/// Resolve a tier by name in a stated locale (Rule N).
pub fn resolve_tier_name_in(loc: LocaleId, s: &str) -> Result<Tier> {
    locale::resolve(loc, s)
}

#[cfg(feature = "alloc")]
fn precision_of(t: Tier) -> Precision {
    if t.is_tick() {
        Precision::Tick
    } else {
        Precision::Tier(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::UC1;
    use crate::value::Rounding;
    use alloc::vec;

    type I = Instant<UC1>;

    fn at(n: u64) -> I {
        I::from_u64(n).unwrap()
    }

    /// A deterministic xorshift, so the "full domain" sweep is reproducible and
    /// needs no float and no rand dependency.
    struct Rng(u64);
    impl Rng {
        fn next_u64(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }
        /// A value spread across the whole 512-bit domain.
        fn next_ticks(&mut self) -> Ticks {
            let mut bytes = [0u8; 64];
            for chunk in bytes.chunks_mut(8) {
                chunk.copy_from_slice(&self.next_u64().to_be_bytes());
            }
            <Ticks as TickInt>::from_canonical_bytes(&bytes).unwrap()
        }
    }

    fn sample_values() -> Vec<Ticks> {
        let mut v = vec![
            <Ticks as TickInt>::zero(),
            <Ticks as TickInt>::one(),
            <Ticks as TickInt>::domain_max(),
            UC1::origin_offset(),
            UC1::beat(),
        ];
        // Every tier boundary, and one tick either side of it.
        for t in Tier::all_ascending() {
            let b = t.ticks();
            v.push(b.clone());
            if let Some(x) = b.try_sub(&<Ticks as TickInt>::one()) {
                v.push(x);
            }
            if let Some(x) = b.try_add(&<Ticks as TickInt>::one()) {
                v.push(x);
            }
        }
        // And a spread across the domain.
        let mut rng = Rng(0x5EED_1234_9ABC_DEF0);
        for _ in 0..256 {
            v.push(rng.next_ticks());
        }
        v
    }

    // ---- Appendix F ----

    #[test]
    fn full_width_encode_takes_45_steps_not_44() {
        // Appendix F and §13.1 both say 44. 2^512-1 has 221 base-5 digits, and
        // T32's group of domain_max is non-zero, so the 45th group is needed.
        // See spec/SPEC-DELTAS.md D-A7.
        let m = <Ticks as TickInt>::domain_max();
        let gs = encode_groups(&m, Tier::new(K_MAX).unwrap(), Tier::TICK).unwrap();
        assert_eq!(gs.len(), 45);
        assert_eq!(gs.len(), TIER_COUNT);
        assert_ne!(gs[0], 0, "the top group must be load-bearing");
        assert_eq!(gs[0], 2);
        // 221 base-5 digits over 5 per group is 44.2, i.e. 45 groups.
        assert_eq!(m.to_radix_string(5).len(), 221);
    }

    #[test]
    fn group_codec_round_trips() {
        for v in sample_values() {
            let gs = encode_groups(&v, Tier::new(K_MAX).unwrap(), Tier::TICK).unwrap();
            assert_eq!(gs.len(), TIER_COUNT);
            assert!(gs.iter().all(|g| *g < GROUP_BASE));
            let back = decode_groups(&gs, Tier::TICK).unwrap();
            assert_eq!(back, v);
        }
    }

    #[test]
    fn digit5_group_codec_round_trips() {
        for g in 0..GROUP_BASE {
            let d = digit5_of_group(g);
            let s = core::str::from_utf8(&d).unwrap();
            assert_eq!(s.len(), 5);
            assert!(s.bytes().all(|b| (b'0'..=b'4').contains(&b)));
            assert_eq!(group_of_digit5(s).unwrap(), g);
        }
        assert_eq!(group_of_digit5("00005").unwrap_err().code, Code::E0005);
        assert_eq!(group_of_digit5("0000").unwrap_err().code, Code::E0005);
    }

    // ---- Rule D: both forms round-trip, over the whole domain ----

    #[test]
    fn human_form_round_trips_over_the_domain() {
        let f = Fmt::human();
        for v in sample_values() {
            let inst = I::from_ticks(v.clone()).unwrap();
            let s = render(&inst, &f).unwrap();
            let (back, p) = parse::<UC1>(&s, &f).unwrap();
            assert_eq!(back, inst, "round trip failed for {s}");
            assert_eq!(p, Precision::Tick, "tick-precision render must parse as tick");
        }
    }

    #[test]
    fn digit_form_round_trips_over_the_domain() {
        let f = Fmt::digit5();
        for v in sample_values() {
            let inst = I::from_ticks(v.clone()).unwrap();
            let s = render(&inst, &f).unwrap();
            let (back, p) = parse::<UC1>(&s, &f).unwrap();
            assert_eq!(back, inst, "round trip failed for {s}");
            assert_eq!(p, Precision::Tick);
        }
    }

    #[test]
    fn named_form_round_trips() {
        let f = Fmt::named();
        for v in sample_values().into_iter().take(64) {
            let inst = I::from_ticks(v).unwrap();
            let s = render(&inst, &f).unwrap();
            let tagged = {
                let mut t = String::from(UC1::TAG);
                t.push(' ');
                t.push_str(&s);
                t
            };
            let (back, _) = parse::<UC1>(&tagged, &f).unwrap();
            assert_eq!(back, inst, "named round trip failed for {s}");
        }
    }

    #[test]
    fn the_two_forms_denote_the_same_value() {
        // Rule D: one value, two tagged forms.
        for v in sample_values().into_iter().take(64) {
            let inst = I::from_ticks(v).unwrap();
            let h = render(&inst, &Fmt::human()).unwrap();
            let d = render(&inst, &Fmt::digit5()).unwrap();
            let (a, _) = parse::<UC1>(&h, &Fmt::human()).unwrap();
            let (b, _) = parse::<UC1>(&d, &Fmt::digit5()).unwrap();
            assert_eq!(a, b);
        }
    }

    // ---- Rule T: precision is the last group's tier ----

    #[test]
    fn truncated_notation_never_yields_tick_precision() {
        // Failure mode F2, stated as a test: no parse path may return
        // Precision::Tick from a string that stops above T-12.
        let inst = at(123_456_789);
        for k in (K_MIN + 1)..=0 {
            let t = Tier::new(k).unwrap();
            let s = render(&inst, &Fmt::human_at(t)).unwrap();
            let (v, p) = parse::<UC1>(&s, &Fmt::human()).unwrap();
            assert_eq!(p, Precision::Tier(t), "precision must be the last group's tier");
            assert_ne!(p, Precision::Tick);
            // The value is the floor, and the window contains the original.
            assert_eq!(v, inst.floor_to(t));
            let w = v.window_at(p).unwrap();
            assert!(w.contains(&inst));
            assert!(!w.is_exact());
        }
    }

    #[test]
    fn coarse_precision_needs_the_digit_or_named_form() {
        // The human form's T0 anchor is a real constraint, so it is reported
        // rather than silently mis-rendered.
        let inst = at(123_456_789);
        let e = render(&inst, &Fmt::human_at(Tier::DEEP)).unwrap_err();
        assert_eq!(e.code, Code::E0006);

        // The digit form is anchored at T32, so the group count states the
        // precision unambiguously at any tier.
        for k in [5i8, 3, 1, 0, -4, -12] {
            let t = Tier::new(k).unwrap();
            let f = Fmt {
                form: Form::Digit5,
                precision: Precision::Tier(t),
                ..Fmt::default()
            };
            let s = render(&inst, &f).unwrap();
            let (v, p) = parse::<UC1>(&s, &f).unwrap();
            assert_eq!(p, precision_of(t), "digit form must state T{k}");
            assert_eq!(v, inst.floor_to(t));
            let groups = s.split(f.sep).count();
            assert_eq!(groups, (K_MAX as i32 - k as i32 + 1) as usize);
        }

        // The named form carries its tiers explicitly, so it needs no anchor.
        let f = Fmt {
            form: Form::Named,
            precision: Precision::Tier(Tier::DRIFT),
            ..Fmt::default()
        };
        // A value far below the stated precision floors to zero there, and the
        // tier still has to appear, because it is what states the precision.
        let named = render(&inst, &f).unwrap();
        assert_eq!(named, "0 drift", "a sub-drift value must still state its tier");

        // A value above the precision renders its real groups.
        let big = I::from_ticks(UC1::origin_offset()).unwrap();
        let named = render(&big, &f).unwrap();
        assert!(named.starts_with("31 deep"), "got {named}");
        assert!(named.ends_with("687 drift"), "got {named}");
    }

    #[test]
    fn parse_window_materialises_the_interval() {
        let inst = at(999_999);
        let s = render(&inst, &Fmt::human_at(Tier::BEAT)).unwrap();
        let w = parse_window::<UC1>(&s, &Fmt::human()).unwrap();
        assert!(w.contains(&inst));
        assert_eq!(w.lo(), &inst.floor_to(Tier::BEAT));
        assert_eq!(
            w.width().ticks(),
            &Tier::BEAT
                .ticks()
                .try_sub(&<Ticks as TickInt>::one())
                .unwrap()
        );
    }

    #[test]
    fn tick_exact_text_runs_to_t_minus_12() {
        // D-A8: a whole SI second has 30 trailing base-5 zeros (§2.4), so its last
        // six groups are zero — and dropping them would change what the string
        // says, from a tick to a T-6 window.
        let bridge_unit = UC1::bridge().ticks;
        let inst = I::from_ticks(UC1::origin_offset().try_add(&bridge_unit).unwrap()).unwrap();
        let s = render(&inst, &Fmt::human()).unwrap();
        let sub = s.split(SUB_SEP).nth(1).unwrap();
        let groups: Vec<&str> = sub.split(SEP).collect();
        assert_eq!(groups.len(), 12, "T-1..T-12");
        assert!(
            groups[6..].iter().all(|g| *g == "0000"),
            "§2.4 guarantees the last six groups are zero"
        );
        // Parsing it back gives a tick, not a window.
        let (back, p) = parse::<UC1>(&s, &Fmt::human()).unwrap();
        assert_eq!(back, inst);
        assert_eq!(p, Precision::Tick);
        // Truncating to T-6 gives the same digits but a different meaning.
        let t6 = render(&inst, &Fmt::human_at(Tier::new(-6).unwrap())).unwrap();
        let (_, p6) = parse::<UC1>(&t6, &Fmt::human()).unwrap();
        assert_eq!(p6, Precision::Tier(Tier::new(-6).unwrap()));
    }

    // ---- Rule S: text sorts only when padded to a fixed tier width ----

    #[test]
    fn padded_digit_form_sorts_chronologically() {
        let mut rng = Rng(0xC0FFEE);
        let mut vals: Vec<I> = (0..128)
            .map(|_| I::from_ticks(rng.next_ticks()).unwrap())
            .collect();
        vals.push(I::zero());
        vals.push(I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap());
        vals.sort();

        let f = Fmt::digit5();
        let rendered: Vec<String> = vals.iter().map(|v| render(v, &f).unwrap()).collect();
        assert!(rendered[0].contains(ALT_SEP), "§6.2 prints the digit form with `.`");
        // Fixed width is the precondition Rule S names.
        let w = rendered[0].len();
        assert!(rendered.iter().all(|s| s.len() == w));
        let mut sorted = rendered.clone();
        sorted.sort();
        assert_eq!(sorted, rendered, "padded digit form must sort chronologically");
    }

    #[test]
    fn unpadded_human_form_is_not_sortable() {
        // Rule S is a real caveat, not a formality: without a fixed tier width the
        // lexicographic order genuinely differs from the chronological one, which
        // is why §6 forbids documenting text forms as sortable.
        let f = Fmt::human();
        let small = at(1);
        let large = I::from_ticks(Tier::DEEP.ticks()).unwrap();
        assert!(small < large);
        let (a, b) = (render(&small, &f).unwrap(), render(&large, &f).unwrap());
        // The larger value has more groups, so the shorter string sorts first only
        // by accident of its leading digits.
        assert_ne!(a.len(), b.len());
    }

    // ---- Rule D: forms must not be mixed ----

    #[test]
    fn mixed_forms_are_rejected() {
        // A base-5 group inside a decimal-tagged string...
        let e = parse::<UC1>("UC1 00111·10222", &Fmt::human()).unwrap_err();
        assert_eq!(e.code, Code::E0003);
        // ...and a decimal group inside a base-5-tagged string.
        let e = parse::<UC1>("UC1/5 0031·0687", &Fmt::digit5()).unwrap_err();
        assert_eq!(e.code, Code::E0003);
        // The digit form takes no sub-beat separator; it is anchored at the top.
        let e = parse::<UC1>("UC1/5 00111.10222:00000", &Fmt::digit5()).unwrap_err();
        assert_eq!(e.code, Code::E0003);
    }

    #[test]
    fn malformed_input_maps_to_the_right_code() {
        for (s, code) in [
            ("XX1 0001", Code::E0002),           // unknown profile tag
            ("0001·0002", Code::E0001),          // missing tag
            ("UC1 3125", Code::E0004),           // group out of range
            ("UC1 99999", Code::E0004),          // and well out of range
            ("UC1/5 00095", Code::E0005),        // invalid base-5 digit
            ("UC1 0001:", Code::E0001),          // sub separator, no groups
            ("UC1 12a4", Code::E0001),           // not a number
        ] {
            let e = parse::<UC1>(s, &Fmt::human()).unwrap_err();
            assert_eq!(e.code, code, "input {s:?}");
        }
    }

    #[test]
    fn alternate_separator_is_accepted_on_input() {
        // §6.3 / D-10: `.` must always be accepted, for shell-hostile contexts.
        let inst = at(987_654_321);
        let canonical = render(&inst, &Fmt::human()).unwrap();
        let dotted = canonical.replace(SEP, ".");
        let (a, _) = parse::<UC1>(&canonical, &Fmt::human()).unwrap();
        let (b, _) = parse::<UC1>(&dotted, &Fmt::human()).unwrap();
        assert_eq!(a, b);
        assert!(dotted.contains('.'));
    }

    #[test]
    fn separator_must_not_be_a_digit() {
        let bad = Fmt {
            sep: '5',
            ..Fmt::default()
        };
        assert_eq!(bad.validate().unwrap_err().code, Code::E0001);
        let same = Fmt {
            sep: ':',
            ..Fmt::default()
        };
        assert_eq!(same.validate().unwrap_err().code, Code::E0001);
        assert!(Fmt::default().validate().is_ok());
    }

    // ---- named form ----

    #[test]
    fn named_form_uses_keys_and_index_notation() {
        let inst = I::from_ticks(
            Tier::DEEP
                .ticks()
                .try_mul(&<Ticks as TickInt>::from_u64(31))
                .unwrap(),
        )
        .unwrap();
        let s = render(&inst, &Fmt::human_at(Tier::BEAT)).unwrap();
        assert!(s.starts_with("UC1 0031"));
        let named = render(&inst, &{
            Fmt {
                form: Form::Named,
                precision: Precision::Tier(Tier::BEAT),
                ..Fmt::default()
            }
        })
        .unwrap();
        assert!(named.starts_with("31 deep"), "got {named}");
        assert!(named.ends_with("0 beat"));
    }

    #[test]
    fn tier_names_resolve_by_key_index_and_exponent() {
        // Rule N: T[k] and 5^e must be accepted wherever a name is.
        assert_eq!(resolve_tier_name("deep").unwrap(), Tier::DEEP);
        assert_eq!(resolve_tier_name("T5").unwrap(), Tier::DEEP);
        assert_eq!(resolve_tier_name("5^85").unwrap(), Tier::DEEP);
        assert_eq!(resolve_tier_name("T-12").unwrap(), Tier::TICK);
        assert_eq!(resolve_tier_name("5^0").unwrap(), Tier::TICK);
        assert_eq!(resolve_tier_name("nope").unwrap_err().code, Code::E0014);
        // Locale names resolve too (Appendix D).
        assert_eq!(
            resolve_tier_name_in(LocaleId::Ru, "\u{431}\u{43e}\u{439}").unwrap(),
            Tier::BEAT
        );
        // An unnamed tier has no key but is still addressable.
        assert!(resolve_tier_name("T7").is_ok());
        assert_eq!(resolve_tier_name("5^61").unwrap_err().code, Code::E0080);
    }

    #[test]
    fn named_terms_must_descend() {
        let f = Fmt::named();
        assert!(parse::<UC1>("UC1 3 deep, 2 drift", &f).is_ok());
        let e = parse::<UC1>("UC1 2 drift, 3 deep", &f).unwrap_err();
        assert_eq!(e.code, Code::E0006);
    }

    // ---- Appendix C cross-check ----

    #[test]
    fn reproduces_appendix_c_beat_parts() {
        // The RFC's printed human forms are T0-precision windows under D-A8; the
        // digits themselves must still match exactly.
        let cases = [
            (UC1::origin_offset(), "UC1 0031·0687·2437·0454·2703·2885"),
            (
                <Ticks as TickInt>::from_dec_str(
                    "8070205189123984864657505252035637180530466139316558837890625",
                )
                .unwrap(),
                "UC1 0031·0687·2481·2999·3108·2437",
            ),
        ];
        for (ticks, want) in cases {
            let inst = I::from_ticks(ticks).unwrap();
            let s = render(&inst, &Fmt::human_at(Tier::BEAT)).unwrap();
            assert_eq!(s, want);
            // ...and it parses back as a beat window, not a tick.
            let (v, p) = parse::<UC1>(&s, &Fmt::human()).unwrap();
            assert_eq!(p, Precision::Tier(Tier::BEAT));
            assert_eq!(v, inst.floor_to(Tier::BEAT));
        }
    }

    #[test]
    fn rounding_to_a_tier_then_rendering_is_stable() {
        let inst = at(123_456_789_012);
        for mode in [
            Rounding::Trunc,
            Rounding::Ceil,
            Rounding::HalfEven,
            Rounding::HalfUp,
        ] {
            let r = inst.round_to(Tier::new(-8).unwrap(), mode).unwrap();
            let s = render(&r, &Fmt::human()).unwrap();
            let (back, p) = parse::<UC1>(&s, &Fmt::human()).unwrap();
            assert_eq!(back, r);
            assert_eq!(p, Precision::Tick);
        }
    }
}
