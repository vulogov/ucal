//! Round dials, drawn with integers.
//!
//! # Why this is not three lines of `f64::sin`
//!
//! A hand at position `p` of 3125 wants a sine and a cosine, and this program
//! contains no floating point anywhere. Rule E is not a style preference here —
//! the whole argument of the project is that time is exact integer arithmetic —
//! and a clock face is a bad place to make the first exception, because it is
//! the part a reader looks at.
//!
//! So the rotation is done by **CORDIC**: shifts, adds and a table of sixteen
//! integer constants. No multiplication by a transcendental, no lookup table of
//! sines, and nothing that could not be done on the machines the two space
//! programme themes are named after.
//!
//! # How it works, briefly
//!
//! CORDIC rotates a vector by decomposing the target angle into a fixed
//! sequence of ever-smaller angles `atan(2^-i)`, each of which can be applied
//! with a shift and an add:
//!
//! ```text
//!   x' = x - d·(y >> i)
//!   y' = y + d·(x >> i)
//! ```
//!
//! where `d` is ±1 chosen to drive the accumulated angle towards the target.
//! After `N` steps the vector has been rotated by the target angle and scaled by
//! a known constant `K`; starting from `x = 1/K` cancels it.
//!
//! Angles are in **units of 1/3125 of a turn**, scaled by [`SCALE`], because
//! that is the unit a tier's hand is already in. A dial has 3125 stops by
//! construction — every rung of the ladder is `5^5` of the one below — so the
//! angle unit and the calendar's unit are the same thing, which is the reason
//! this was worth doing rather than a reason it was hard.
//!
//! # What it draws on
//!
//! A braille canvas: each character cell is a 2x4 grid of dots, so a `w x h`
//! pane is a `2w x 4h` field. The aspect ratio is corrected when plotting,
//! because a cell is about twice as tall as it is wide and an uncorrected circle
//! is an ellipse.

/// Fixed-point scale for CORDIC. A power of two, so the shifts are exact.
pub const SCALE: i64 = 1 << 16;

/// Angle unit: one 3125th of a turn, scaled.
///
/// A full turn is `3125 * ANGLE_UNIT`, so a hand position maps to an angle by
/// multiplication and nothing else.
const TURN: i64 = 3125;

/// `atan(2^-i)` for `i = 0..16`, in units of a full turn scaled by `SCALE * TURN`.
///
/// Sixteen entries is more precision than a braille canvas can show: the finest
/// dial here is a few hundred dots around, and sixteen CORDIC steps resolve an
/// angle to about one part in 200 000.
///
/// Generated once and checked by a test that re-derives each entry from the
/// arctangent identity using integer arithmetic only — a table of magic numbers
/// with no way to check it is exactly the kind of thing this project refuses.
const ATAN: [i64; 16] = [
    25_600_000, 15_112_562, 7_985_063, 4_053_343, 2_034_537, 1_018_260, 509_254, 254_643,
    127_323, 63_662, 31_831, 15_915, 7_958, 3_979, 1_989, 995,
];

/// The CORDIC gain, inverted and scaled: `SCALE / K` where `K ≈ 1.6467602`.
const INV_GAIN: i64 = 39_797;

/// Cosine and sine of `position / 3125` of a turn, scaled by [`SCALE`].
///
/// Exact integer arithmetic throughout. The result is accurate to about one part
/// in 60 000, which is four orders of magnitude finer than any dial drawn here.
pub fn cos_sin(position: u32) -> (i64, i64) {
    // Reduce to a quadrant: CORDIC converges only for |angle| < ~99.9 degrees,
    // so the quadrant is handled by symmetry and the rotation covers the rest.
    let p = (position as i64).rem_euclid(TURN);
    let quadrant = (p * 4) / TURN;
    let within = p - quadrant * TURN / 4;

    // Target angle, in the same units as ATAN: a full turn is `SCALE * TURN`,
    // and `within` is already in 3125ths of a turn, so the conversion is one
    // multiplication. It was `* SCALE * 4` first, which is four times a turn and
    // put every hand somewhere plausible and wrong.
    let mut angle = within * SCALE;

    let (mut x, mut y) = (INV_GAIN, 0i64);
    for (i, step) in ATAN.iter().enumerate() {
        let d = if angle >= 0 { 1 } else { -1 };
        let (nx, ny) = (x - d * (y >> i), y + d * (x >> i));
        x = nx;
        y = ny;
        angle -= d * step;
    }

    // Back out of the quadrant. Clock convention: position 0 is straight up and
    // the hand runs clockwise, which is a rotation and a reflection away from
    // the mathematical convention CORDIC produces.
    // CORDIC gives `(x, y) = (cos θ, sin θ)` with θ measured from the +x axis.
    // A clock measures from straight up, clockwise, so `c` is the up component
    // and `s` the right one — which for the first quadrant is exactly `(x, y)`
    // and for the rest is that rotated by a right angle each time.
    match quadrant {
        0 => (x, y),
        1 => (-y, x),
        2 => (-x, -y),
        _ => (y, -x),
    }
}

/// A grid of braille dots.
///
/// `2 x 4` dots per character cell, which is what braille gives and the densest
/// a terminal gets without a graphics protocol.
pub struct Canvas {
    cols: usize,
    rows: usize,
    dots: Vec<u8>,
}

/// Bit for a dot at `(x, y)` within a cell, in Unicode braille order.
const DOT_BITS: [[u8; 2]; 4] = [[0x01, 0x08], [0x02, 0x10], [0x04, 0x20], [0x40, 0x80]];

impl Canvas {
    /// A canvas `cols` characters wide and `rows` tall.
    pub fn new(cols: usize, rows: usize) -> Canvas {
        Canvas {
            cols,
            rows,
            dots: vec![0; cols * rows],
        }
    }

    /// Light the dot at dot-coordinates `(x, y)`, ignoring anything off-canvas.
    pub fn set(&mut self, x: i64, y: i64) {
        if x < 0 || y < 0 {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        let (cx, cy) = (x / 2, y / 4);
        if cx >= self.cols || cy >= self.rows {
            return;
        }
        self.dots[cy * self.cols + cx] |= DOT_BITS[y % 4][x % 2];
    }

    /// Draw the rim of a dial and one hand at `position`.
    ///
    /// # Resolution, and what a hand here can honestly mean
    ///
    /// A tier has 3125 stops. A rim of this size has a circumference of a few
    /// hundred dots, so it resolves roughly one stop in thirty and **no dial
    /// drawn here can show a stop**. Ticks are not drawn for that reason: 3125
    /// marks on a 70-dot rim is a solid ring claiming a precision it does not
    /// have, and a smaller number of marks is a grid the calendar does not use.
    ///
    /// So the hand says which *part* of the tier the instant is in, and the
    /// numeral printed beneath says which stop — the division of labour a clock
    /// with numerals on its face has always had.
    pub fn dial(&mut self, position: u32) {
        let (cx, cy) = (self.cols as i64, self.rows as i64 * 2);
        let radius = core::cmp::min(cx, cy) - 1;
        if radius < 3 {
            return;
        }

        // No aspect correction. A braille cell is two dots across and four down
        // and is about twice as tall as it is wide, so the *dots* are square and
        // a circle in dot-space is a circle on screen. The first version halved
        // the y term as if the dots were cells, and drew every dial as an
        // ellipse — a correction applied at the wrong level looks exactly like
        // one that was needed.
        let steps = radius * 8;
        for i in 0..steps {
            let p = (i as u64 * TURN as u64 / steps as u64) as u32;
            let (c, s) = cos_sin(p);
            self.set(cx + s * radius / SCALE, cy - c * radius / SCALE);
        }

        // The hand, as points along the radius rather than by a line algorithm:
        // the radius is short and the points are what a hand is.
        let (c, s) = cos_sin(position);
        for r in 0..radius {
            self.set(cx + s * r / SCALE, cy - c * r / SCALE);
        }
    }

    /// The canvas as lines of braille.
    pub fn lines(&self) -> Vec<String> {
        (0..self.rows)
            .map(|r| {
                (0..self.cols)
                    .map(|c| {
                        let bits = self.dots[r * self.cols + c];
                        char::from_u32(0x2800 + bits as u32).unwrap_or(' ')
                    })
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The table is re-derived rather than trusted.
    ///
    /// A table of magic numbers with no way to check it is exactly what this
    /// project refuses everywhere else. Each entry is `atan(2^-i)` in units of
    /// `SCALE * TURN` per full turn, re-derived here from the arctangent series
    /// in fixed-point integers — no float — and compared against what is
    /// committed.
    ///
    /// The first version of the table was wrong, and this test is how that was
    /// established rather than guessed: `ATAN[0]` is `atan(1)/2π` of a turn,
    /// which is exactly an eighth, so the first entry must be `SCALE * TURN / 8`
    /// on the nose. It read `25_565_312` against an exact `25_600_000`.
    #[test]
    fn the_arctangent_table_is_what_it_claims() {
        // An eighth of a turn, exactly, with no series needed.
        assert_eq!(
            ATAN[0],
            SCALE * TURN / 8,
            "atan(1) is an eighth of a turn by definition"
        );

        // atan(t) = t - t^3/3 + t^5/5 - ..., in fixed point with headroom.
        const P: i128 = 1 << 60;
        // 2*pi, scaled by P, as an integer constant with more digits than the
        // table needs. Rule E: this is a ratio of integers, not a float.
        let two_pi: i128 = 7_244_019_458_077_122_842; // round(2*pi * 2^60)
        // From `i = 1`. At `t = 1` the series is Leibniz's and needs thousands
        // of terms to reach four figures; eighty gets 25 396 313 against the
        // exact 25 600 000, which is the test being wrong rather than the table.
        // `i = 0` is checked exactly above and needs no series at all.
        for (i, entry) in ATAN.iter().enumerate().skip(1) {
            let t = P >> i;
            let mut term = t;
            let mut sum = 0i128;
            let mut k = 1i128;
            while term != 0 && k < 80 {
                let add = term / k;
                if (k / 2) % 2 == 0 {
                    sum += add;
                } else {
                    sum -= add;
                }
                // Multiply before dividing. `term / P` is the unscaled value,
                // which for `t < 1` truncates to zero and made every entry from
                // `i = 1` on come out of a series that had stopped after one
                // term. Fixed-point arithmetic gets this wrong in exactly one
                // direction and it looks plausible either way.
                term = term * t / P;
                term = term * t / P;
                k += 2;
            }
            // radians -> turns -> table units, all in integers.
            let want = sum * (SCALE as i128 * TURN as i128) / two_pi;
            let got = *entry as i128;
            assert!(
                (want - got).abs() <= 2,
                "ATAN[{i}] is {got}, re-derivation says {want}"
            );
        }
    }

    /// Every point CORDIC produces is on the unit circle.
    ///
    /// `cos^2 + sin^2 = 1`, scaled. This is the check that catches a wrong gain
    /// constant, a wrong quadrant, or a table entry with a typo — all three of
    /// which produce plausible-looking hands.
    #[test]
    fn every_position_lands_on_the_circle() {
        for p in (0..3125).step_by(7) {
            let (c, s) = cos_sin(p);
            let r2 = c * c + s * s;
            let want = SCALE * SCALE;
            let err = (r2 - want).abs() * 10_000 / want;
            assert!(err <= 5, "position {p}: |v|^2 off by {err} parts in 10 000");
        }
    }

    /// Position 0 is straight up, and the hand runs clockwise.
    ///
    /// The convention every clock face has. Getting it wrong produces a working
    /// dial that runs backwards, which is the kind of thing that survives review.
    #[test]
    fn zero_is_up_and_the_hand_runs_clockwise() {
        let (c, s) = cos_sin(0);
        assert!(c > SCALE * 99 / 100, "0 should be straight up, got ({c}, {s})");
        assert!(s.abs() < SCALE / 100);

        // A quarter turn clockwise is to the right.
        let (c, s) = cos_sin(3125 / 4);
        assert!(s > SCALE * 99 / 100, "a quarter turn should point right");
        assert!(c.abs() < SCALE / 50);

        // And half a turn is down.
        let (c, s) = cos_sin(3125 / 2);
        assert!(c < -SCALE * 99 / 100, "half a turn should point down");
        let _ = s;
    }

    /// A dial draws a ring and a hand, and the hand moves.
    #[test]
    fn a_dial_is_a_ring_with_a_hand_that_moves() {
        let ink = |p: u32| {
            let mut c = Canvas::new(20, 10);
            c.dial(p);
            c.lines().join("\n")
        };
        let a = ink(0);
        let b = ink(3125 / 4);
        assert!(a.chars().any(|ch| ch != ' ' && ch != '\u{2800}' && ch != '\n'));
        assert_ne!(a, b, "the hand did not move a quarter turn");
    }

    /// A canvas too small for a dial draws nothing rather than panicking.
    #[test]
    fn a_dial_with_no_room_draws_nothing() {
        for (w, h) in [(0usize, 0usize), (1, 1), (2, 1), (3, 1)] {
            let mut c = Canvas::new(w, h);
            c.dial(1000);
            let _ = c.lines();
        }
    }
}
