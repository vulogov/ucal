//! A block font, for the one readout that is meant to be read across a room.
//!
//! Five columns by five rows per glyph, drawn with `█`. Five and not seven
//! because the readout is four digits wide and a terminal is eighty columns:
//! `4 × (5 + 1)` leaves room for a label beside it, and `4 × (7 + 1)` does not.

/// The glyph rows for one digit, top to bottom.
///
/// `#` is ink. Anything else is space, which keeps the table readable as a
/// picture of what it draws — the point of a hand-built font.
const GLYPHS: [[&str; 5]; 10] = [
    [
        "#####", //
        "#   #", //
        "#   #", //
        "#   #", //
        "#####",
    ],
    [
        "   ##", //
        "    #", //
        "    #", //
        "    #", //
        "    #",
    ],
    [
        "#####", //
        "    #", //
        "#####", //
        "#    ", //
        "#####",
    ],
    [
        "#####", //
        "    #", //
        " ####", //
        "    #", //
        "#####",
    ],
    [
        "#   #", //
        "#   #", //
        "#####", //
        "    #", //
        "    #",
    ],
    [
        "#####", //
        "#    ", //
        "#####", //
        "    #", //
        "#####",
    ],
    [
        "#####", //
        "#    ", //
        "#####", //
        "#   #", //
        "#####",
    ],
    [
        "#####", //
        "    #", //
        "   # ", //
        "  #  ", //
        "  #  ",
    ],
    [
        "#####", //
        "#   #", //
        "#####", //
        "#   #", //
        "#####",
    ],
    [
        "#####", //
        "#   #", //
        "#####", //
        "    #", //
        "#####",
    ],
];

/// How many rows tall a rendered line is.
pub const HEIGHT: usize = 5;

/// Render a run of ASCII digits as [`HEIGHT`] lines of block characters.
///
/// Any character that is not a digit becomes a column of spaces the width of a
/// separator, so `"0031"` and `"00·31"` both render without the caller having to
/// strip anything.
pub fn render(s: &str) -> [String; HEIGHT] {
    let mut rows: [String; HEIGHT] = Default::default();
    for ch in s.chars() {
        let glyph = ch.to_digit(10).and_then(|d| GLYPHS.get(d as usize));
        for (r, row) in rows.iter_mut().enumerate() {
            match glyph.and_then(|g| g.get(r)) {
                Some(pattern) => {
                    for c in pattern.chars() {
                        row.push(if c == '#' { '█' } else { ' ' });
                    }
                    row.push(' ');
                }
                // A separator, or a digit this font does not have. Two columns,
                // so the groups stay distinguishable.
                None => row.push_str("  "),
            }
        }
    }
    rows
}

/// How wide [`render`] will be, without rendering it.
pub fn width(s: &str) -> usize {
    s.chars()
        .map(|c| if c.is_ascii_digit() { 6 } else { 2 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_digit_has_a_glyph_of_the_declared_size() {
        for (d, glyph) in GLYPHS.iter().enumerate() {
            assert_eq!(glyph.len(), HEIGHT, "digit {d}");
            for row in glyph {
                assert_eq!(row.chars().count(), 5, "digit {d} row `{row}`");
            }
        }
    }

    /// The glyphs must be distinguishable from each other, which a hand-built
    /// font is exactly the thing to get wrong. `8` and `9` differ by one row.
    #[test]
    fn no_two_digits_render_identically() {
        for a in 0..10u32 {
            for b in (a + 1)..10 {
                assert_ne!(
                    GLYPHS[a as usize], GLYPHS[b as usize],
                    "{a} and {b} are the same glyph"
                );
            }
        }
    }

    #[test]
    fn width_matches_what_render_produces() {
        for s in ["0", "0031", "00·31", "", "3124"] {
            let rows = render(s);
            for row in &rows {
                assert_eq!(row.chars().count(), width(s), "for `{s}`");
            }
        }
    }
}
