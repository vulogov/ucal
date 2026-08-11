//! Wall-clock themes: a palette and a layout switch.
//!
//! Not a plug-in system. A theme is a `const` here and a key in [`by_name`], and
//! that is deliberate — the same reason `ucal cal list` enumerates a compiled-in
//! registry: a caller asking for a theme that does not exist should be told so
//! by name, and a catalogue you can enumerate is the only way to tell them what
//! does exist.

use ratatui::style::Color;
use ucal_core::{Code, TimeError};

/// How a face is arranged, as opposed to what colour it is.
///
/// This was a `bool` — `lcars: true` or not — for exactly as long as there were
/// two layouts. [`Z2-wallclock-faces.md`] predicted where that would stop being
/// honest: *"a third layout is where `lcars: bool` stops being honest and
/// `Theme` needs a `layout` enum"*. `starwars` is the third.
///
/// Closed on purpose. A layout is a body of drawing code in this module, not a
/// value a caller can supply, so an exhaustive match is the feature — see
/// `CLOSED_VOCABULARIES` in `xtask/src/lint.rs` for the rule.
///
/// [`Z2-wallclock-faces.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/Z2-wallclock-faces.md
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    /// Labels down the left, readout at the top. No chrome.
    Plain,
    /// An elbow into a vertical rail of blocks, LCARS.
    Lcars,
    /// A gunsight: canopy frame, reticle, crosshair, and a HUD strip.
    Targeting,
}

/// A clock face's colours and chrome.
#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct Theme {
    /// The key `--theme` takes.
    pub key: &'static str,
    /// One line, for `--theme list`.
    pub about: &'static str,
    /// How the face is arranged.
    pub layout: Layout,
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
    layout: Layout::Plain,
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
    layout: Layout::Lcars,
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

/// DEC VT220 amber phosphor: one warm hue on near-black, and no second one.
///
/// The block font in [`super::digits`] was drawn for this and had never been
/// shown in it. A palette and nothing else — the layout is `plain`'s.
pub const AMBER: Theme = Theme {
    key: "amber",
    about: "VT220 amber phosphor — one warm hue, no second one",
    layout: Layout::Plain,
    background: Color::Rgb(0x0A, 0x06, 0x00),
    text: Color::Rgb(0xFF, 0xB0, 0x00),
    label: Color::Rgb(0x99, 0x66, 0x00),
    primary: Color::Rgb(0xFF, 0xC8, 0x40),
    blocks: &[Color::Rgb(0xFF, 0xB0, 0x00)],
    blur: Color::Rgb(0x99, 0x66, 0x00),
};

/// The other half of the same idea: IBM 3270 / VT100 green.
pub const GREEN: Theme = Theme {
    key: "green",
    about: "3270 green phosphor — the other half of the same idea",
    layout: Layout::Plain,
    background: Color::Rgb(0x00, 0x0A, 0x00),
    text: Color::Rgb(0x33, 0xFF, 0x33),
    label: Color::Rgb(0x11, 0x88, 0x11),
    primary: Color::Rgb(0x66, 0xFF, 0x66),
    blocks: &[Color::Rgb(0x33, 0xFF, 0x33)],
    blur: Color::Rgb(0x11, 0x88, 0x11),
};

/// Dark on light, committed.
///
/// The useful one rather than the evocative one. [`PLAIN`] uses `Color::Reset`
/// and inherits whatever the terminal has, which is right until someone puts a
/// frame on a light background and the labels — `DarkGray` on white — go faint.
/// This one commits to the background it is drawn on.
pub const PAPER: Theme = Theme {
    key: "paper",
    about: "dark on light, committed — for light terminals and for print",
    layout: Layout::Plain,
    background: Color::Rgb(0xFA, 0xF8, 0xF2),
    text: Color::Rgb(0x1A, 0x1A, 0x1A),
    label: Color::Rgb(0x5A, 0x5A, 0x5A),
    primary: Color::Rgb(0x00, 0x00, 0x00),
    blocks: &[Color::Rgb(0x33, 0x33, 0x33)],
    blur: Color::Rgb(0x8A, 0x8A, 0x8A),
};

/// A targeting computer.
///
/// The other science-fiction interface everyone has seen, and structurally the
/// opposite of LCARS. LCARS is a *console*: coloured blocks, generous space,
/// numbers set against a rail, an interface for reading. A gunsight is an
/// *instrument*: a frame at the edge of vision, a reticle in the middle, one
/// number that matters, and everything else compressed into a strip along the
/// bottom. Amber wireframe on black, because that is what a lit reticle looks
/// like through a canopy.
///
/// It earns its place by being a third layout rather than a fourth palette —
/// which is the bar [`Z2-wallclock-faces.md`] set for a theme, and the reason
/// `blueprint` is not here.
///
/// [`Z2-wallclock-faces.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/Z2-wallclock-faces.md
pub const TARGETING: Theme = Theme {
    key: "starwars",
    about: "a targeting computer — canopy frame, reticle, and a HUD strip",
    layout: Layout::Targeting,
    background: Color::Black,
    text: Color::Rgb(0xFF, 0xA5, 0x2C),
    label: Color::Rgb(0x8A, 0x55, 0x14),
    primary: Color::Rgb(0xFF, 0xC8, 0x5C),
    blocks: &[
        Color::Rgb(0xFF, 0xA5, 0x2C),
        Color::Rgb(0xE0, 0x50, 0x20),
        Color::Rgb(0x4C, 0xD9, 0x64),
        Color::Rgb(0xFF, 0xC8, 0x5C),
        Color::Rgb(0x8A, 0x55, 0x14),
    ],
    blur: Color::Rgb(0xE0, 0x50, 0x20),
};

/// Every theme, in the order `--theme list` prints them.
pub const ALL: &[&Theme] = &[&PLAIN, &AMBER, &GREEN, &PAPER, &LCARS, &TARGETING];

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
