//! Colour, as a table of roles rather than escape sequences at call sites.
//!
//! # The invariant
//!
//! A style may add SGR sequences. It may never change a character:
//!
//! ```text
//! strip_ansi(doc.to_ansi(&style)) == doc.to_text()      // byte for byte
//! ```
//!
//! [`Doc::to_text`](crate::emit::Doc::to_text) is defined as
//! `to_ansi(&Style::PLAIN)`, so there is one renderer with plain as its identity
//! case rather than two renderings to keep in agreement. That makes the property
//! above hold by construction for the layout and leaves exactly one thing for the
//! test to check: that no `Style` smuggles a character through.
//!
//! The reason this is a mechanical check and not a convention is the same reason
//! `SignedWindow` has no operators. Anything a reader can learn from colour must
//! be learnable without it — from a pipe, from a log, from a terminal that has no
//! colour at all — and "we were careful" is not a property that survives a
//! contributor.
//!
//! # What colour is allowed to say
//!
//! Nothing that is not also said in words.
//!
//! The design opened with a different plan for [`Role::Padding`]: dim the digits
//! *below* a value's stated precision, making Rule T's interval visible in the
//! value. Measuring the output killed it. No shipped rendering pads below its
//! precision — every form truncates, so `--precision beat` produces a shorter
//! string rather than a padded one, and Rule T is already visible by length.
//! There was nothing to dim.
//!
//! What the forms do carry is padding at the other end. Rule S makes the base-5
//! form fixed-width so that lexicographic order is chronological order, and at
//! the present epoch that means 27 of its 45 groups are leading zeros — 135
//! base-5 digits of domain nobody has reached. Those are what `Padding` marks,
//! and dimming them shows at a glance how little of a 512-bit range is in use.
//!
//! The `precision` field still appears in every rendering, unchanged. Colour
//! shows a reader where the value sits in the domain; it is never the only place
//! something is said.

use std::io::IsTerminal as _;

/// What a run of text *is*, so a [`Style`] can decide how to show it.
///
/// Roles are about the data, not about an appearance. `Padding` does not mean
/// "grey"; it means "below the stated precision", and a style chooses what that
/// looks like — including choosing nothing, which is what [`Style::PLAIN`] does.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Role {
    /// A document heading and the rule beneath it.
    Title,
    /// A field name.
    Key,
    /// An ordinary value.
    Value,
    /// A digit run that is part of the measured value.
    Digits,
    /// The alternating group in a long digit run.
    ///
    /// A sixty-one-digit integer is unreadable in one run, and every fix that
    /// inserts a character makes it unpastable. Alternating the appearance of
    /// three-digit groups leaves the character stream identical, so selecting the
    /// number still yields the number. When a separator *is* asked for, this role
    /// still applies — the two are independent, and a reader who wants both gets
    /// both.
    DigitAlt,
    /// The leading zero run of a fixed-width form.
    ///
    /// Structurally zero because Rule S fixes the width at the profile's domain
    /// rather than at the value's magnitude — not zero because anything was
    /// rounded away. Never the only indication; see the module note.
    Padding,
    /// A separator inside a rendered form: the group mark, the tier boundary.
    Separator,
    /// Explanatory prose attached to a field or a document.
    Note,
    /// A `UCAL-W####` warning.
    Warning,
    /// A `UCAL-E####` error.
    Error,
}

/// A role-to-appearance table.
///
/// Held as [`anstyle::Style`] values, which render to SGR sequences and to
/// nothing at all when empty. `anstyle` arrives with clap and is not a new
/// dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style {
    title: anstyle::Style,
    key: anstyle::Style,
    value: anstyle::Style,
    digits: anstyle::Style,
    digit_alt: anstyle::Style,
    padding: anstyle::Style,
    separator: anstyle::Style,
    note: anstyle::Style,
    warning: anstyle::Style,
    error: anstyle::Style,
}

impl Style {
    /// Every role empty. The identity case, and what `to_text` renders with.
    pub const PLAIN: Style = Style {
        title: anstyle::Style::new(),
        key: anstyle::Style::new(),
        value: anstyle::Style::new(),
        digits: anstyle::Style::new(),
        digit_alt: anstyle::Style::new(),
        padding: anstyle::Style::new(),
        separator: anstyle::Style::new(),
        note: anstyle::Style::new(),
        warning: anstyle::Style::new(),
        error: anstyle::Style::new(),
    };

    /// The shipped colour scheme.
    ///
    /// Deliberately restrained, and built from the eight ANSI colours plus
    /// dimming rather than from 256-colour or truecolor codes: this output is
    /// read in terminals whose palettes are set by their owners, and a scheme
    /// that assumes a background is a scheme that is unreadable on half of them.
    pub const fn colored() -> Style {
        use anstyle::{AnsiColor, Color, Effects, Style as S};
        Style {
            title: S::new().effects(Effects::BOLD),
            key: S::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack))),
            value: S::new(),
            digits: S::new(),
            // The alternation has to be visible without being decorative: this
            // is one number, not two colours of number.
            digit_alt: S::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack))),
            padding: S::new().effects(Effects::DIMMED),
            separator: S::new().effects(Effects::DIMMED),
            note: S::new().effects(Effects::DIMMED),
            warning: S::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
            error: S::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
        }
    }

    /// True when no role in this table renders anything.
    ///
    /// Used to skip the SGR machinery entirely on the plain path, so the common
    /// case allocates no escape sequences at all.
    pub fn is_plain(&self) -> bool {
        *self == Style::PLAIN
    }

    /// The appearance for one role.
    pub fn get(&self, role: Role) -> anstyle::Style {
        match role {
            Role::Title => self.title,
            Role::Key => self.key,
            Role::Value => self.value,
            Role::Digits => self.digits,
            Role::DigitAlt => self.digit_alt,
            Role::Padding => self.padding,
            Role::Separator => self.separator,
            Role::Note => self.note,
            Role::Warning => self.warning,
            Role::Error => self.error,
        }
    }

    /// Wrap `text` in this role's sequences, or return it unchanged when the
    /// role renders nothing.
    pub fn paint(&self, role: Role, text: &str) -> String {
        let s = self.get(role);
        if s == anstyle::Style::new() || text.is_empty() {
            return text.to_string();
        }
        format!("{s}{text}{s:#}")
    }
}

impl Default for Render {
    fn default() -> Render {
        Render::PLAIN
    }
}

impl Default for Style {
    fn default() -> Style {
        Style::PLAIN
    }
}

/// When to colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ColorChoice {
    /// Colour when stdout is a terminal and nothing has asked otherwise.
    #[default]
    Auto,
    /// Always colour, even into a pipe. For `less -R` and for recording.
    Always,
    /// Never colour.
    Never,
}

impl ColorChoice {
    /// Parse a `--color` value.
    pub fn parse(s: &str) -> Result<ColorChoice, ucal_core::TimeError> {
        match s {
            "auto" => Ok(ColorChoice::Auto),
            "always" => Ok(ColorChoice::Always),
            "never" => Ok(ColorChoice::Never),
            _ => Err(ucal_core::TimeError::with_context(
                ucal_core::Code::E0001,
                "color must be auto, always or never",
            )),
        }
    }

    /// Resolve to a concrete style.
    ///
    /// Precedence, highest first: an explicit `--color`/`--no-color`, then the
    /// `NO_COLOR` environment variable, then whether stdout is a terminal. JSON
    /// is decided before this is called and never reaches it — see
    /// [`resolve_for_output`].
    pub fn resolve(self) -> Style {
        match self {
            ColorChoice::Never => Style::PLAIN,
            ColorChoice::Always => Style::colored(),
            ColorChoice::Auto => {
                // no-color.org: any non-empty value disables colour. Honoured
                // above tty detection, because a user who sets it has asked.
                if std::env::var_os("NO_COLOR").is_some_and(|v| !v.is_empty()) {
                    return Style::PLAIN;
                }
                // std's own tty test, so this needs no crate beyond anstyle.
                if !std::io::stdout().is_terminal() {
                    return Style::PLAIN;
                }
                Style::colored()
            }
        }
    }
}

/// The style for a run, given every input that can suppress colour.
///
/// `json` wins over everything. Colour inside a `--json` document is not a
/// preference a user can hold: §19.1 makes that output a stable, versioned
/// contract for a *program*, and an SGR sequence in it is a defect whether or
/// not a terminal is attached.
pub fn resolve_for_output(choice: ColorChoice, json: bool) -> Style {
    if json {
        return Style::PLAIN;
    }
    choice.resolve()
}

/// How to render a document: the style table, plus the choices a reader makes.
///
/// Separate from [`Style`] because these are not appearance. A group separator
/// changes the characters, so it is not covered by the strip invariant and must
/// not be — the invariant is a claim about *colour*, and quietly widening it to
/// cover a flag that inserts characters would make it vacuous.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Render {
    /// The role-to-appearance table.
    pub style: Style,
    /// Separator inserted between decimal digit groups, if any.
    ///
    /// Off by default, and that default is a deliberate trade rather than an
    /// oversight. A tick count is frequently copied out of this output and
    /// pasted into something that wants an integer; a separator breaks that,
    /// and the colour alternation gives a reader the same grouping without
    /// touching a single character. `--tick-sep` is for the reader who wants
    /// the separator anyway, or who is reading without colour — down a pipe,
    /// in a log, on a terminal that has none.
    pub group: Option<char>,
    /// Available columns. See [`crate::table`] for why the floor is fixed.
    pub cols: usize,
}

impl Render {
    /// No colour, no separator. What [`Doc::to_text`](crate::emit::Doc::to_text)
    /// renders with, and byte-identical to the output that predates this module.
    pub const PLAIN: Render = Render {
        style: Style::PLAIN,
        group: None,
        cols: crate::table::BASELINE_WIDTH,
    };

    /// A style with no separator, at the baseline width.
    pub fn styled(style: Style) -> Render {
        Render {
            style,
            ..Render::PLAIN
        }
    }

    /// Set the available width, never below the documented baseline.
    ///
    /// The floor is one-directional on purpose. A wider terminal is used; a
    /// narrower one is not, because a layout that reflows below 80 columns would
    /// make the same command emit different bytes on different machines.
    pub fn width(mut self, w: usize) -> Render {
        self.cols = w.max(crate::table::BASELINE_WIDTH);
        self
    }

    /// The width to render at, given an explicit `--width` and the terminal.
    ///
    /// Off a terminal the answer is the baseline, always. If width followed the
    /// terminal on a redirected stream, the same command would put different
    /// bytes into a pipe than into a file.
    pub fn resolve_width(explicit: Option<usize>, terminal: Option<usize>) -> usize {
        explicit
            .or(terminal)
            .unwrap_or(crate::table::BASELINE_WIDTH)
            .max(crate::table::BASELINE_WIDTH)
    }

    /// Set the digit-group separator.
    pub fn group(mut self, sep: Option<char>) -> Render {
        self.group = sep;
        self
    }
}

/// Validate a digit-group separator.
///
/// A digit would be indistinguishable from the number it is separating, which is
/// the same reason §6.3 forbids one for the text forms. The rule is repeated here
/// rather than shared because the two flags are about different renderings and a
/// future divergence should be a decision, not a surprise.
pub fn parse_group_sep(s: &str) -> Result<char, ucal_core::TimeError> {
    let mut it = s.chars();
    let (Some(c), None) = (it.next(), it.next()) else {
        return Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0001,
            "the group separator must be exactly one character",
        ));
    };
    if c.is_ascii_digit() {
        return Err(ucal_core::TimeError::with_context(
            ucal_core::Code::E0001,
            "the group separator must not be a digit (§6.3)",
        ));
    }
    Ok(c)
}

/// Render a decimal integer in three-digit groups.
///
/// Two mechanisms over the same grouping, and they are independent:
///
/// - the appearance alternates between [`Role::Digits`] and [`Role::DigitAlt`],
///   which adds no character and so survives a copy-paste;
/// - a separator is inserted when one was asked for, which does add characters
///   and is therefore off unless requested.
///
/// Anything that is not a plain optionally-signed integer is returned painted as
/// a single [`Role::Digits`] run. Splitting a value this does not understand
/// would risk changing it, and a renderer that guesses at a format is how an
/// exact integer stops being one.
pub fn group_decimal(render: &Render, s: &str) -> String {
    let (sign, digits) = match s.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", s),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return render.style.paint(Role::Digits, s);
    }
    // Nothing to gain below two groups, and a separator there is just noise.
    if digits.len() <= 3 {
        let mut out = String::from(sign);
        out.push_str(&render.style.paint(Role::Digits, digits));
        return out;
    }

    // Group from the right, so the leading group is the short one.
    let first = match digits.len() % 3 {
        0 => 3,
        n => n,
    };
    let mut out = String::from(sign);
    let mut idx = 0;
    let mut group = 0;
    while idx < digits.len() {
        let take = if idx == 0 { first } else { 3 };
        let end = (idx + take).min(digits.len());
        if idx > 0 {
            if let Some(sep) = render.group {
                out.push_str(&render.style.paint(Role::Separator, &sep.to_string()));
            }
        }
        let role = if group % 2 == 0 {
            Role::Digits
        } else {
            Role::DigitAlt
        };
        out.push_str(&render.style.paint(role, &digits[idx..end]));
        idx = end;
        group += 1;
    }
    out
}

/// Paint a rendered timestamp form: `UC1 0031·0687·…`, `UC1/5 00000.…`, a UCID.
///
/// Three regions, and the split is structural rather than a guess at what looks
/// good:
///
/// - the form tag up to the first space, which names the encoding;
/// - the leading zero run, which is domain the value has not reached — Rule S
///   fixes the width at the profile's ceiling, not at the value's magnitude;
/// - the digits, with their group separators.
///
/// A string with no space is taken as all body, which is what a UCID is.
pub fn paint_form(render: &Render, s: &str) -> String {
    let style = &render.style;
    let (tag, body) = match s.find(' ') {
        Some(i) => (&s[..=i], &s[i + 1..]),
        None => ("", s),
    };

    let mut out = String::with_capacity(s.len());
    if !tag.is_empty() {
        out.push_str(&style.paint(Role::Value, tag));
    }

    // Leading zeros end at the first character that is neither `0` nor a
    // separator. A separator inside the run stays part of it, so a whole group
    // of zeros dims as one region rather than in five-character pieces.
    let leading_end = body
        .char_indices()
        .find(|(_, c)| c.is_alphanumeric() && *c != '0')
        .map(|(i, _)| i)
        .unwrap_or(body.len());

    // Batch runs of one role, so a 135-digit region costs one pair of sequences
    // rather than 135.
    let mut run = String::new();
    let mut run_role: Option<Role> = None;
    let flush = |out: &mut String, run: &mut String, role: &mut Option<Role>| {
        if let Some(r) = role.take() {
            out.push_str(&style.paint(r, run));
        }
        run.clear();
    };
    for (i, c) in body.char_indices() {
        // A separator inside the leading run takes the run's role rather than
        // its own. Otherwise the region alternates between two roles and emits
        // a sequence pair per group: 27 groups became 54 pairs for one prefix,
        // and the dimming read as stripes instead of a region.
        let role = if i < leading_end {
            Role::Padding
        } else if !c.is_alphanumeric() {
            Role::Separator
        } else {
            Role::Digits
        };
        if run_role != Some(role) {
            flush(&mut out, &mut run, &mut run_role);
            run_role = Some(role);
        }
        run.push(c);
    }
    flush(&mut out, &mut run, &mut run_role);
    out
}

/// Remove every SGR sequence from `s`.
///
/// Exists for the invariant test rather than for the renderer, which is why it
/// handles the full CSI shape rather than only the sequences [`Style`] emits: a
/// stripper that only understands what we currently write would pass the test by
/// agreeing with the bug.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\u{1b}' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            // CSI: parameter bytes 0x30-0x3f, intermediate 0x20-0x2f, final 0x40-0x7e
            Some('[') => {
                chars.next();
                for c in chars.by_ref() {
                    if ('\u{40}'..='\u{7e}').contains(&c) {
                        break;
                    }
                }
            }
            // Two-character escape.
            Some(_) => {
                chars.next();
            }
            None => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_paints_nothing() {
        let s = Style::PLAIN;
        assert!(s.is_plain());
        assert_eq!(s.paint(Role::Padding, "0000"), "0000");
        assert_eq!(s.paint(Role::Error, "UCAL-E0001"), "UCAL-E0001");
    }

    #[test]
    fn colored_paints_and_strips_back() {
        let s = Style::colored();
        assert!(!s.is_plain());
        for role in [
            Role::Title,
            Role::Key,
            Role::Value,
            Role::Digits,
            Role::Padding,
            Role::Separator,
            Role::Note,
            Role::Warning,
            Role::Error,
        ] {
            let painted = s.paint(role, "abc");
            assert_eq!(strip_ansi(&painted), "abc", "role {role:?} changed the text");
        }
    }

    #[test]
    fn empty_text_is_never_wrapped() {
        // An empty painted run would still emit an on/off pair, which strips
        // back to the same string but bloats every line with dead sequences.
        assert_eq!(Style::colored().paint(Role::Padding, ""), "");
    }

    #[test]
    fn strip_handles_sequences_we_do_not_emit() {
        assert_eq!(strip_ansi("\u{1b}[2J\u{1b}[1;31mred\u{1b}[0m"), "red");
        assert_eq!(strip_ansi("\u{1b}[38;2;12;34;56mx\u{1b}[m"), "x");
        assert_eq!(strip_ansi("plain"), "plain");
        // A lone escape at the end must not panic or eat the buffer.
        assert_eq!(strip_ansi("a\u{1b}"), "a");
        assert_eq!(strip_ansi("a\u{1b}["), "a");
    }

    #[test]
    fn json_suppresses_colour_whatever_was_asked() {
        assert!(resolve_for_output(ColorChoice::Always, true).is_plain());
        assert!(resolve_for_output(ColorChoice::Auto, true).is_plain());
        assert!(resolve_for_output(ColorChoice::Never, true).is_plain());
        // And --color=always still colours when it is not JSON.
        assert!(!resolve_for_output(ColorChoice::Always, false).is_plain());
    }

    #[test]
    fn color_choice_parses_and_refuses() {
        assert_eq!(ColorChoice::parse("auto").unwrap(), ColorChoice::Auto);
        assert_eq!(ColorChoice::parse("always").unwrap(), ColorChoice::Always);
        assert_eq!(ColorChoice::parse("never").unwrap(), ColorChoice::Never);
        assert!(ColorChoice::parse("yes").is_err());
        assert!(ColorChoice::parse("").is_err());
    }
}
