//! Wall-clock themes: a palette and a layout switch.
//!
//! Not a plug-in system. A theme is a `const` here and a key in [`by_name`], and
//! that is deliberate — the same reason `ucal cal list` enumerates a compiled-in
//! registry: a caller asking for a theme that does not exist should be told so
//! by name, and a catalogue you can enumerate is the only way to tell them what
//! does exist.

use ratatui::style::Color;
use ucal_core::{Code, TimeError};

/// A clock face's colours and chrome.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Theme {
    /// The key `--theme` takes.
    pub key: &'static str,
    /// One line, for `--theme list`.
    pub about: &'static str,
    /// Whether to draw LCARS elbows and rails, or a plain frame.
    pub lcars: bool,
    /// The page.
    pub background: Color,
    /// Ordinary text.
    pub text: Color,
    /// Section headings and rail labels.
    pub label: Color,
    /// The big readout.
    pub primary: Color,
    /// The rail blocks, cycled top to bottom.
    pub blocks: &'static [Color],
    /// The sub-visible tier's bar.
    pub blur: Color,
}

/// The default: readable anywhere, no chrome, no assumptions about the palette.
///
/// Deliberately first and deliberately dull. A clock that only looked right in
/// one aesthetic would be a demo.
pub const PLAIN: Theme = Theme {
    key: "plain",
    about: "monochrome, no chrome — the default",
    lcars: false,
    background: Color::Reset,
    text: Color::Reset,
    label: Color::DarkGray,
    primary: Color::Reset,
    blocks: &[Color::DarkGray],
    blur: Color::DarkGray,
};

/// LCARS, the Star Trek library computer interface.
///
/// The palette is the one the production design settled on: a warm orange for
/// the structural elbows, peach and lilac for the rails, and a red reserved for
/// the one block that is not a readout. Black behind everything, because LCARS
/// is a lit surface rather than a printed one.
///
/// The layout follows: an elbow across the top-left joining a header bar to a
/// vertical rail of blocks, numbers right-aligned in the rail, and the readout
/// given the whole of the remaining space.
pub const LCARS: Theme = Theme {
    key: "startrek",
    about: "LCARS — the library computer interface, in its production palette",
    lcars: true,
    background: Color::Black,
    text: Color::Rgb(0xFF, 0xCC, 0x99),
    label: Color::Rgb(0x99, 0x99, 0xFF),
    primary: Color::Rgb(0xFF, 0x99, 0x00),
    blocks: &[
        Color::Rgb(0xFF, 0x99, 0x00),
        Color::Rgb(0xCC, 0x99, 0xCC),
        Color::Rgb(0x99, 0x99, 0xFF),
        Color::Rgb(0xFF, 0xCC, 0x99),
        Color::Rgb(0xCC, 0x66, 0x66),
    ],
    blur: Color::Rgb(0xCC, 0x66, 0x66),
};

/// Every theme, in the order `--theme list` prints them.
pub const ALL: &[&Theme] = &[&PLAIN, &LCARS];

/// Look a theme up by key.
///
/// `UCAL-E0016` for a name that is not one, which is the code for a name that is
/// not in a declared catalogue — the same answer `ucal cal show` gives for a
/// calendar that does not exist, for the same reason.
pub fn by_name(key: &str) -> Result<&'static Theme, TimeError> {
    ALL.iter()
        .find(|t| t.key == key)
        .copied()
        .ok_or_else(|| {
            TimeError::with_context(
                Code::E0016,
                "no such wall-clock theme; `ucal wallclock --theme list` names every one",
            )
        })
}
