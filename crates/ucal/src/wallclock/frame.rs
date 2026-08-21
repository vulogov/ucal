//! One frame, as text — the clock without a terminal.
//!
//! # Why this exists
//!
//! Everything about the clock's appearance was, until this module, a claim held
//! up by a test that renders into a buffer and greps it. That is a real
//! mechanism and an invisible one: nobody reading `Documentation/CLI.md` could
//! see a face, so the documentation described one in prose, and prose rots.
//!
//! `ucal wallclock --once --at <INSTANT>` writes a single frame and exits. With
//! a fixed instant and a fixed size the output is **deterministic**, which makes
//! it committable — `xtask gen-examples` writes one into the documentation and
//! `check-docs` fails when the binary stops producing it. The same mechanism
//! that catches a stale worked example now catches a stale screenshot.
//!
//! # Colour
//!
//! Plain text when colour is off, ANSI when it is on. The generated artefact
//! uses the first, because an escape sequence in a committed file is a diff
//! nobody can read; a person running the command in a terminal gets the second.

use super::face::Face;
use super::theme::Theme;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use ratatui::Terminal;
use ucal_core::{Code, TimeError};

/// Render one frame at a fixed size.
///
/// `TestBackend` and not `CrosstermBackend`: the size must be an argument rather
/// than whatever the terminal happens to be, or the output is not reproducible
/// and cannot be committed. That is the whole point of the command.
pub fn once(
    face: &Face,
    theme: &Theme,
    width: u16,
    height: u16,
    color: bool,
) -> Result<String, TimeError> {
    if !(20..=400).contains(&width) || !(10..=200).contains(&height) {
        // D-A24: a range left, which is one of the four shapes `E0018` names.
        // This raised `E0001` — *malformed timestamp* — for one release, because
        // that was what every other argument rejection in the binary used and
        // consistency beat a better name. The survey that fixed all of them is
        // in `Documentation/Proposals/V3-argument-codes.md`.
        return Err(TimeError::with_context(
            Code::E0018,
            "a frame is between 20x10 and 400x200; outside that the layout has \
             nothing to say and the output would not be a clock",
        ));
    }
    let mut term = Terminal::new(TestBackend::new(width, height)).map_err(|_| {
        TimeError::with_context(Code::E0017, "a frame buffer could not be allocated")
    })?;
    term.draw(|f| face.render(f, theme))
        .map_err(|_| TimeError::with_context(Code::E0017, "the frame could not be drawn"))?;
    Ok(to_text(term.backend().buffer(), color))
}

/// A buffer as lines, with trailing blanks removed.
///
/// Trailing spaces are stripped per line because a committed artefact with
/// invisible trailing whitespace is a file that every editor and every linter
/// will quietly disagree about.
fn to_text(buf: &Buffer, color: bool) -> String {
    let mut out = String::new();
    for y in 0..buf.area.height {
        let mut line = String::new();
        let mut open = false;
        let mut last: Option<(Color, Color)> = None;
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            if color {
                let want = (cell.fg, cell.bg);
                if last != Some(want) {
                    if open {
                        line.push_str("\u{1b}[0m");
                    }
                    let sgr = sgr_for(cell.fg, cell.bg);
                    if sgr.is_empty() {
                        open = false;
                    } else {
                        line.push_str(&sgr);
                        open = true;
                    }
                    last = Some(want);
                }
            }
            line.push_str(cell.symbol());
        }
        if open {
            line.push_str("\u{1b}[0m");
        }
        out.push_str(trim_end(&line));
        out.push('\n');
    }
    out
}

/// Trim trailing spaces without cutting an escape sequence in half.
fn trim_end(line: &str) -> &str {
    if line.contains('\u{1b}') {
        return line;
    }
    line.trim_end()
}

/// An SGR sequence for a foreground and background pair.
///
/// Written here rather than taken from `anstyle` because ratatui's `Color` is
/// the input and translating it is the whole job; routing it through a third
/// representation would add a conversion and no correctness.
fn sgr_for(fg: Color, bg: Color) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(c) = code(fg, false) {
        parts.push(c);
    }
    if let Some(c) = code(bg, true) {
        parts.push(c);
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!("\u{1b}[{}m", parts.join(";"))
    }
}

fn code(c: Color, background: bool) -> Option<String> {
    let base = if background { 40 } else { 30 };
    let bright = if background { 100 } else { 90 };
    Some(match c {
        Color::Reset => return None,
        Color::Black => format!("{}", base),
        Color::Red => format!("{}", base + 1),
        Color::Green => format!("{}", base + 2),
        Color::Yellow => format!("{}", base + 3),
        Color::Blue => format!("{}", base + 4),
        Color::Magenta => format!("{}", base + 5),
        Color::Cyan => format!("{}", base + 6),
        Color::Gray => format!("{}", base + 7),
        Color::DarkGray => format!("{}", bright),
        Color::LightRed => format!("{}", bright + 1),
        Color::LightGreen => format!("{}", bright + 2),
        Color::LightYellow => format!("{}", bright + 3),
        Color::LightBlue => format!("{}", bright + 4),
        Color::LightMagenta => format!("{}", bright + 5),
        Color::LightCyan => format!("{}", bright + 6),
        Color::White => format!("{}", bright + 7),
        Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", base + 8),
        Color::Indexed(i) => format!("{};5;{i}", base + 8),
    })
}
