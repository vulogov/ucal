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
//! Nothing that is not also said in words. The clearest case is [`Role::Padding`]:
//! the digits below a value's stated precision are dimmed, which makes Rule T's
//! interval visible in the value itself — but the `precision` field stays in every
//! rendering, because a reader who cannot see the dimming must still be told.

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
    /// A digit run *below the stated precision*: structurally zero, and not
    /// determined by the input. Never the only indication — see the module note.
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
