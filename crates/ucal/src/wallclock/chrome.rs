//! The words a face prints that are not data.
//!
//! # Why this exists
//!
//! `--locale` is this program's language flag and Rule N scopes it to tier
//! *names*. 1.5.0 made the wall clock's tier names follow it and left the chrome
//! alone, which made the Vostok panel the one place a theme overrode a user's
//! flag: `--gagarin --locale en` drew Cyrillic anyway. The other six faces had
//! the mirror-image bug and nobody called it one, because English chrome under
//! `--locale ru` looks like a translation nobody finished rather than like a
//! flag being ignored. It is the same bug.
//!
//! # What follows the locale and what does not
//!
//! **Sentences this program wrote follow it.** `ВРЕМЯ ВСЕЛЕННОЙ` is a
//! translation of *universe calendar*; a reader who asked for English should get
//! English, and one who asked for Russian should get Russian on every face.
//!
//! **Instrument legends do not.** `VERB`, `NOUN`, `PROG`, `KEY REL` and
//! `MONITOR DECIMAL · TIME` are the DSKY's own, printed on a panel in 1969;
//! `Q TO DISENGAGE` and `T1 ARC` belong to their consoles the same way. They are
//! proper nouns of the object being drawn, not descriptions of what the clock is
//! doing, and translating them would invent a Russian Apollo that never existed.
//!
//! That distinction is the whole content of this module: a theme may keep the
//! words its instrument actually bore, and may not keep a translation of a
//! sentence this program wrote.
//!
//! # Numbers stay in the strings
//!
//! `66 000 PER SECOND` and `2 MIN 26 S` are inside the translated text rather
//! than substituted into it. They are properties of the tier ladder — `T-1` is
//! 66 000 stops a second in every language — but a translator needs them *in*
//! the sentence to put them where the sentence wants them, and a format hole
//! that every locale fills identically is a hole for a locale to get wrong.

use ucal_core::LocaleId;

/// The translatable words of a face.
///
/// Every field is a whole sentence or label rather than a fragment to be
/// concatenated: word order is the first thing translation changes, and a
/// sentence assembled from three fields is a sentence only English can say.
///
/// `#[non_exhaustive]`, and it will stay that way: every face this program grows
/// brings sentences with it, so a caller who could match this struct
/// exhaustively would break on the next theme.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Chrome {
    /// The plain face's header: what this program is.
    pub program: &'static str,
    /// The same, for the face made of dials.
    pub program_dials: &'static str,
    /// A console's title: what this instrument measures.
    pub title: &'static str,
    /// A console's subtitle: what the units are.
    pub subtitle: &'static str,
    /// The Vostok panel's engraved plate, beside the beat.
    pub main_plate: &'static str,
    /// The Vostok panel's caption under the sub-visible bar, which prints its
    /// own tier label first.
    pub blur_caption: &'static str,
    /// The shared bar's caption, on the faces that draw it whole.
    pub blur_line: &'static str,
    /// The gunsight's caption: the flicker is on the axis of the cross.
    pub blur_axis: &'static str,
    /// The DSKY's caption: too fast for a register.
    pub blur_register: &'static str,
    /// The dial face's caption: the finest hand is not a hand.
    pub blur_no_dial: &'static str,
    /// The rate of the DSKY's moving register, after its tier name.
    pub beat_rate: &'static str,
    /// The pace of the arc, on the console that states it.
    pub arc_pace: &'static str,
    /// Why every dial has 3125 stops.
    pub stops_note: &'static str,
    /// The ready lamp.
    pub ready: &'static str,
    /// How to leave, on a console with a legend for it.
    pub quit: &'static str,
    /// How to leave, on a face that just says so.
    pub quit_hint: &'static str,
    /// The year label of a second calendar.
    pub year: &'static str,
    /// The day label of a second calendar.
    pub day: &'static str,
    /// Where a second calendar's year 1 comes from.
    pub anchor_note: &'static str,
    /// The second calendar's progress bar, after its percentage.
    pub through_local_day: &'static str,
    /// The odometer's label, counting up from an origin.
    pub since: &'static str,
    /// The odometer's label, counting towards an origin in the future.
    pub until: &'static str,
    /// What the big readout is, when it is not the tier that moves.
    pub hero_is_slow: &'static str,
    /// The label of a second calendar's anchor revision.
    pub anchor_revision_label: &'static str,
    /// That an anchor is an observation, printed after its revision.
    pub anchor_revision: &'static str,
}

const EN: Chrome = Chrome {
    program: "UCAL — universe calendar",
    program_dials: "UCAL — universe calendar, on dials",
    title: "UNIVERSE CALENDAR · UC1",
    subtitle: "ABSOLUTE TIME · PLANCK TICKS · BASE FIVE",
    main_plate: "PRIMARY COUNT",
    blur_caption: "66 000 PER SECOND · A POSITION, NOT A NUMBER",
    blur_line: "T-1 FLICKER · 66 000 PER SECOND · A POSITION, NOT A NUMBER",
    blur_axis: "T-1 FLICKER ON THE AXIS · 66 000 PER SECOND",
    blur_register: "T-1 FLICKER · TOO FAST FOR A REGISTER",
    blur_no_dial: "the finest hand has no dial: 66 000 stops a second is not a hand",
    beat_rate: "21 PER SECOND",
    arc_pace: "ONE STOP EVERY 2 MIN 26 S",
    stops_note: "every tier has 3125 stops, because every rung is 5^5 of the one below",
    ready: "READY",
    quit: "EXIT",
    quit_hint: "q to quit",
    year: "year",
    day: "day",
    anchor_note: "counted from the anchor — year 1 began there",
    through_local_day: "through the local day",
    since: "SINCE",
    until: "UNTIL",
    hero_is_slow: "a calendar display: this hand does not move while you watch",
    anchor_revision_label: "anchor revision",
    anchor_revision: "an anchor is an observation and is versioned (Rule J)",
};

const RU: Chrome = Chrome {
    program: "UCAL — календарь вселенной",
    program_dials: "UCAL — календарь вселенной, на циферблатах",
    title: "ВРЕМЯ ВСЕЛЕННОЙ · UC1",
    subtitle: "АБСОЛЮТНОЕ ВРЕМЯ · ПЛАНКОВСКИЕ ТИКИ · ОСНОВАНИЕ ПЯТЬ",
    main_plate: "ОСНОВНОЙ ОТСЧЁТ",
    blur_caption: "66 000 В СЕКУНДУ · ПОЛОЖЕНИЕ, НЕ ЧИСЛО",
    blur_line: "T-1 МЕРЦАНИЕ · 66 000 В СЕКУНДУ · ПОЛОЖЕНИЕ, НЕ ЧИСЛО",
    blur_axis: "T-1 МЕРЦАНИЕ НА ОСИ · 66 000 В СЕКУНДУ",
    blur_register: "T-1 МЕРЦАНИЕ · СЛИШКОМ БЫСТРО ДЛЯ РЕГИСТРА",
    blur_no_dial: "у самой тонкой стрелки нет циферблата: 66 000 делений в секунду — это не стрелка",
    beat_rate: "21 В СЕКУНДУ",
    arc_pace: "ОДИН ШАГ КАЖДЫЕ 2 МИН 26 С",
    stops_note: "у каждого разряда 3125 делений, потому что каждая ступень — 5^5 предыдущей",
    ready: "ГОТОВ",
    quit: "ВЫХОД",
    quit_hint: "q — выход",
    year: "год",
    day: "сутки",
    anchor_note: "отсчёт от опорной точки — там начался год 1",
    through_local_day: "местных суток пройдено",
    since: "ПРОШЛО ОТ",
    until: "ОСТАЛОСЬ ДО",
    hero_is_slow: "календарь, а не часы: эта стрелка не движется, пока вы смотрите",
    anchor_revision_label: "версия опорной точки",
    anchor_revision: "это наблюдение, а наблюдения версионируются (правило J)",
};

/// The chrome for a locale.
///
/// Falls back to English for a locale this module has no words in. That
/// fallback is visible rather than silent: the tier names come from
/// `ucal-core` and will still be in the caller's language, so a half-translated
/// face reads as one language's chrome around another language's labels — which
/// is what it is.
pub fn of(locale: LocaleId) -> Chrome {
    match locale.tag() {
        "ru" => RU,
        _ => EN,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fields(c: &Chrome) -> Vec<(&'static str, &'static str)> {
        vec![
            ("program", c.program),
            ("program_dials", c.program_dials),
            ("title", c.title),
            ("subtitle", c.subtitle),
            ("main_plate", c.main_plate),
            ("blur_caption", c.blur_caption),
            ("blur_line", c.blur_line),
            ("blur_axis", c.blur_axis),
            ("blur_register", c.blur_register),
            ("blur_no_dial", c.blur_no_dial),
            ("beat_rate", c.beat_rate),
            ("arc_pace", c.arc_pace),
            ("stops_note", c.stops_note),
            ("ready", c.ready),
            ("quit", c.quit),
            ("quit_hint", c.quit_hint),
            ("year", c.year),
            ("day", c.day),
            ("anchor_note", c.anchor_note),
            ("through_local_day", c.through_local_day),
            ("since", c.since),
            ("until", c.until),
            ("hero_is_slow", c.hero_is_slow),
            ("anchor_revision_label", c.anchor_revision_label),
            ("anchor_revision", c.anchor_revision),
        ]
    }

    /// Every shipped locale has chrome, and none of it is empty.
    #[test]
    fn every_shipped_locale_has_words() {
        for l in LocaleId::ALL {
            let c = of(*l);
            for (what, s) in fields(&c) {
                assert!(!s.trim().is_empty(), "{}: {what} is empty", l.tag());
            }
        }
    }

    /// Every field of the Russian table differs from the English one.
    ///
    /// A translation table with a copied entry passes every other test in this
    /// file and leaves one English sentence in the middle of a Russian face.
    /// Checked field by field rather than as a whole, because the whole differs
    /// as soon as one field does.
    #[test]
    fn no_russian_field_was_left_in_english() {
        let en = fields(&of(LocaleId::parse("en").expect("en ships")));
        let ru = fields(&of(LocaleId::parse("ru").expect("ru ships")));
        for ((what, e), (_, r)) in en.iter().zip(ru.iter()) {
            assert_ne!(e, r, "{what} is the same in both locales");
        }
    }

    /// The Russian table is in Cyrillic, apart from what a console bore.
    ///
    /// `no_russian_field_was_left_in_english` would pass on a field translated
    /// into a third language, or into Latin transliteration. This one says the
    /// words are actually Russian, allowing the tier labels (`T-1`), the
    /// program's own name (`UCAL`) and the numbers, which no locale changes.
    #[test]
    fn the_russian_table_is_cyrillic() {
        for (what, s) in fields(&of(LocaleId::parse("ru").expect("ru ships"))) {
            assert!(
                s.chars().any(|c| ('\u{400}'..='\u{4ff}').contains(&c)),
                "ru: {what} has no Cyrillic in it: {s}"
            );
        }
    }
}
