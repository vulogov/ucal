//! Locale tables for tier names (Appendix D, Rule N).
//!
//! # Names are display-only
//!
//! Rule N: a tier's canonical identity is its **exponent**. Everything here is a
//! display and parse alias, and nothing in the library decides behaviour from a
//! name. Adding a locale therefore cannot change what any value means — which is
//! what makes D-20's position tenable, that naming the unnamed tiers is a locale
//! change rather than a specification change.
//!
//! Rule N also requires `T[k]` and `5^e` notation to be accepted *wherever a name
//! is accepted*, so every resolver here falls back to them.
//!
//! # Why the names are what they are
//!
//! Appendix D records the criterion: short, concrete motion words with no
//! mythological, religious, national or numeric-prefix content. That rules out
//! the obvious candidates — no "aeon", no "epoch", no "kilo-" or "mega-" — and it
//! is why the ladder reads *deep, drift, span, sweep, arc, beat, flicker, glint,
//! spark* rather than anything more familiar. Familiarity here would be a defect:
//! the scale is not a second and should not sound like one.
//!
//! Calendar unit names — day, year, cycle — are deliberately **absent**. They
//! belong to a body's calendar and are declared with it (§9), not to the
//! universal ladder.

use crate::error::{Code, Result, TimeError};
use crate::tier::{Tier, TierName, NAMED};

/// A shipped locale (D-19).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
#[non_exhaustive]
pub enum LocaleId {
    /// English. The stable keys double as the English names.
    #[default]
    En,
    /// Russian.
    Ru,
}

impl LocaleId {
    /// The BCP-47-ish tag.
    pub const fn tag(self) -> &'static str {
        match self {
            LocaleId::En => "en",
            LocaleId::Ru => "ru",
        }
    }

    /// Every shipped locale.
    pub const ALL: &'static [LocaleId] = &[LocaleId::En, LocaleId::Ru];

    /// Resolve a locale tag.
    pub fn parse(tag: &str) -> Result<LocaleId> {
        LocaleId::ALL
            .iter()
            .copied()
            .find(|l| l.tag() == tag)
            .ok_or(TimeError::with_context(
                Code::E0010,
                "unknown locale; shipped locales are en and ru",
            ))
    }
}

/// A tier's names in one locale: singular and plural.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Names {
    /// One of them.
    pub singular: &'static str,
    /// More than one.
    pub plural: &'static str,
}

const fn n(singular: &'static str, plural: &'static str) -> Names {
    Names { singular, plural }
}

/// The English table. The stable keys and the display names coincide, which is
/// why `en` needs no aliasing.
const EN: &[(TierName, Names)] = &[
    (TierName::Deep, n("deep", "deeps")),
    (TierName::Drift, n("drift", "drifts")),
    (TierName::Span, n("span", "spans")),
    (TierName::Sweep, n("sweep", "sweeps")),
    (TierName::Arc, n("arc", "arcs")),
    (TierName::Beat, n("beat", "beats")),
    (TierName::Flicker, n("flicker", "flickers")),
    (TierName::Glint, n("glint", "glints")),
    (TierName::Spark, n("spark", "sparks")),
    (TierName::Tick, n("tick", "ticks")),
];

/// The Russian table (Appendix D, D-19).
const RU: &[(TierName, Names)] = &[
    (TierName::Deep, n("глубь", "глуби")),
    (TierName::Drift, n("дрейф", "дрейфы")),
    (TierName::Span, n("срок", "сроки")),
    (TierName::Sweep, n("обход", "обходы")),
    (TierName::Arc, n("дуга", "дуги")),
    (TierName::Beat, n("бой", "бои")),
    (TierName::Flicker, n("мерцание", "мерцания")),
    (TierName::Glint, n("блик", "блики")),
    (TierName::Spark, n("искра", "искры")),
    (TierName::Tick, n("тик", "тики")),
];

/// The table for a locale.
pub const fn table(locale: LocaleId) -> &'static [(TierName, Names)] {
    match locale {
        LocaleId::En => EN,
        LocaleId::Ru => RU,
    }
}

/// The names of a tier in a locale, if the tier is named at all.
///
/// Unnamed tiers return `None` and are addressed by index (D-20).
pub fn names_of(locale: LocaleId, tier: Tier) -> Option<Names> {
    let key = crate::tier::name_of(tier)?;
    table(locale)
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| *v)
}

/// A tier's display name in a locale, or its `T[k]` form if unnamed.
///
/// Always returns something printable, because Rule N guarantees the index
/// notation is valid wherever a name is.
#[cfg(feature = "alloc")]
pub fn display(locale: LocaleId, tier: Tier) -> alloc::string::String {
    use alloc::string::ToString;
    match names_of(locale, tier) {
        Some(n) => n.singular.to_string(),
        None => tier.to_string(),
    }
}

/// Resolve a tier from a locale name, a stable key, `T[k]`, or `5^e` (Rule N).
///
/// Singular and plural both resolve, and matching is case-insensitive for ASCII;
/// a user who types `Deeps` means the same tier as one who types `deep`.
pub fn resolve(locale: LocaleId, s: &str) -> Result<Tier> {
    let t = s.trim();

    // Index and exponent notation, accepted wherever a name is (Rule N).
    if let Some(k) = t.strip_prefix('T') {
        if let Ok(idx) = k.parse::<i8>() {
            return Tier::new(idx);
        }
    }
    if let Some(e) = t.strip_prefix("5^") {
        if let Ok(exp) = e.parse::<u32>() {
            return Tier::from_exponent(exp);
        }
    }

    // Locale names, then the stable keys, so a key always works in any locale.
    let lowered = ascii_lower(t);
    for (key, names) in table(locale) {
        if eq_fold(names.singular, &lowered) || eq_fold(names.plural, &lowered) {
            return tier_of_name(*key);
        }
    }
    for (k, key) in NAMED {
        if eq_fold(key.key(), &lowered) {
            return Tier::new(*k);
        }
    }
    Err(TimeError::with_context(
        Code::E0011,
        "unknown tier name; try a locale name, a stable key, T<k>, or 5^e",
    ))
}

fn tier_of_name(key: TierName) -> Result<Tier> {
    NAMED
        .iter()
        .find(|(_, k)| *k == key)
        .map(|(idx, _)| Tier::new(*idx))
        .unwrap_or(Err(TimeError::new(Code::E0011)))
}

/// Case-insensitive comparison for ASCII, exact for everything else.
///
/// Deliberately not a full Unicode case fold: Russian tier names are compared as
/// written. Case-folding Cyrillic correctly needs tables this crate has no reason
/// to carry, and getting it half-right would be worse than not doing it.
fn eq_fold(candidate: &str, lowered_input: &str) -> bool {
    if candidate.is_ascii() {
        candidate.eq_ignore_ascii_case(lowered_input)
    } else {
        candidate == lowered_input
    }
}

#[cfg(feature = "alloc")]
fn ascii_lower(s: &str) -> alloc::string::String {
    s.chars()
        .map(|c| if c.is_ascii() { c.to_ascii_lowercase() } else { c })
        .collect()
}

#[cfg(not(feature = "alloc"))]
fn ascii_lower(s: &str) -> &str {
    s
}

/// Validate a locale table: every name distinct, every named tier covered.
///
/// Rule N makes a collision within an active table `UCAL-E0011`. Checked rather
/// than assumed, because a collision would make a name ambiguous on input while
/// still looking fine on output.
pub fn validate(locale: LocaleId) -> Result<()> {
    let t = table(locale);
    // Every named tier must appear exactly once.
    for (_, key) in NAMED {
        let count = t.iter().filter(|(k, _)| k == key).count();
        if count != 1 {
            return Err(TimeError::with_context(
                Code::E0010,
                "locale table does not cover every named tier exactly once",
            ));
        }
    }
    // No two display names may collide, singular or plural.
    for (i, (_, a)) in t.iter().enumerate() {
        for (_, b) in t.iter().skip(i + 1) {
            if a.singular == b.singular
                || a.plural == b.plural
                || a.singular == b.plural
                || a.plural == b.singular
            {
                return Err(TimeError::with_context(
                    Code::E0011,
                    "duplicate name in the active locale table",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_locale_is_valid() {
        for l in LocaleId::ALL {
            validate(*l).unwrap_or_else(|e| panic!("locale {} is invalid: {e}", l.tag()));
        }
    }

    #[test]
    fn every_locale_covers_every_named_tier() {
        // §13.5: the tier table and the locale table come from one source, so a
        // tier cannot be named in one and missing from the other.
        for l in LocaleId::ALL {
            assert_eq!(table(*l).len(), NAMED.len(), "locale {}", l.tag());
            for (k, _) in NAMED {
                let tier = Tier::new(*k).unwrap();
                assert!(
                    names_of(*l, tier).is_some(),
                    "locale {} is missing T{k}",
                    l.tag()
                );
            }
        }
    }

    #[test]
    fn unnamed_tiers_stay_unnamed_in_every_locale() {
        // D-20: tiers above T5 and below T-3 are addressable by index only.
        for l in LocaleId::ALL {
            for k in [6i8, 10, 32, -4, -8, -11] {
                let tier = Tier::new(k).unwrap();
                assert!(names_of(*l, tier).is_none(), "T{k} in {}", l.tag());
                // ...but they always have a printable form.
                assert_eq!(display(*l, tier), alloc::format!("T{k}"));
            }
        }
    }

    #[test]
    fn names_resolve_in_both_locales() {
        assert_eq!(resolve(LocaleId::En, "deep").unwrap(), Tier::DEEP);
        assert_eq!(resolve(LocaleId::En, "deeps").unwrap(), Tier::DEEP);
        assert_eq!(resolve(LocaleId::Ru, "глубь").unwrap(), Tier::DEEP);
        assert_eq!(resolve(LocaleId::Ru, "глуби").unwrap(), Tier::DEEP);
        assert_eq!(resolve(LocaleId::Ru, "бой").unwrap(), Tier::BEAT);
        assert_eq!(resolve(LocaleId::Ru, "тик").unwrap(), Tier::TICK);
    }

    #[test]
    fn the_stable_key_works_in_any_locale() {
        // The key is the identity across locales, so `beat` resolves under `ru`
        // even though the Russian display name is `бой`.
        for l in LocaleId::ALL {
            assert_eq!(resolve(*l, "beat").unwrap(), Tier::BEAT);
            assert_eq!(resolve(*l, "deep").unwrap(), Tier::DEEP);
        }
    }

    #[test]
    fn index_and_exponent_notation_work_wherever_a_name_does() {
        // Rule N states this explicitly.
        for l in LocaleId::ALL {
            assert_eq!(resolve(*l, "T0").unwrap(), Tier::BEAT);
            assert_eq!(resolve(*l, "T-12").unwrap(), Tier::TICK);
            assert_eq!(resolve(*l, "5^60").unwrap(), Tier::BEAT);
            assert_eq!(resolve(*l, "5^220").unwrap(), Tier::new(32).unwrap());
            // Including for tiers that have no name at all (D-20).
            assert_eq!(resolve(*l, "T7").unwrap(), Tier::new(7).unwrap());
            assert_eq!(resolve(*l, "5^95").unwrap(), Tier::new(7).unwrap());
        }
    }

    #[test]
    fn ascii_names_fold_case_but_cyrillic_is_taken_as_written() {
        assert_eq!(resolve(LocaleId::En, "DEEP").unwrap(), Tier::DEEP);
        assert_eq!(resolve(LocaleId::En, "Deeps").unwrap(), Tier::DEEP);
        // Cyrillic is compared as written; folding it correctly needs tables this
        // crate has no reason to carry.
        assert!(resolve(LocaleId::Ru, "глубь").is_ok());
        assert!(resolve(LocaleId::Ru, "ГЛУБЬ").is_err());
    }

    #[test]
    fn unknown_names_are_e0011() {
        for l in LocaleId::ALL {
            assert_eq!(resolve(*l, "aeon").unwrap_err().code, Code::E0011);
            assert_eq!(resolve(*l, "").unwrap_err().code, Code::E0011);
        }
        // An off-grid exponent is a tier error, not a naming one.
        assert_eq!(resolve(LocaleId::En, "5^61").unwrap_err().code, Code::E0080);
    }

    #[test]
    fn locale_tags_round_trip() {
        for l in LocaleId::ALL {
            assert_eq!(LocaleId::parse(l.tag()).unwrap(), *l);
        }
        assert_eq!(LocaleId::parse("xx").unwrap_err().code, Code::E0010);
    }

    #[test]
    fn no_calendar_units_appear_in_the_ladder() {
        // Appendix D: day, year and cycle belong to a body's calendar, not to the
        // universal ladder. A name collision with one would invite exactly the
        // conflation §8.3 exists to prevent.
        for l in LocaleId::ALL {
            for (_, names) in table(*l) {
                for n in [names.singular, names.plural] {
                    for forbidden in ["day", "year", "month", "week", "hour", "second"] {
                        assert_ne!(n, forbidden, "locale {} names a calendar unit", l.tag());
                    }
                }
            }
        }
    }

    #[test]
    fn names_avoid_the_content_appendix_d_rules_out() {
        // "no mythological, religious, national, or numeric-prefix content".
        for l in LocaleId::ALL {
            for (_, names) in table(*l) {
                let s = names.singular;
                for prefix in ["kilo", "mega", "giga", "tera", "milli", "micro", "nano"] {
                    assert!(!s.starts_with(prefix), "{s} carries a numeric prefix");
                }
                assert!(!s.is_empty());
                // Short: every name is a single word.
                assert!(!s.contains(' '), "{s} is not a single word");
            }
        }
    }
}
