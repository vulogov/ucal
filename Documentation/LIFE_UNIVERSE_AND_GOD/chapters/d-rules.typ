#import "../design.typ": *

#appendix(letter: "D", title: "The rules")

All twenty-four, with the chapters that use them. Subject lines are taken from
`spec/RULES.md`, which is the normative statement; the glosses are this book's.

The fourteen the book actually cites are marked. The other ten govern parts of
the software the book does not discuss, and are listed so that the set is
complete rather than curated.

#section("The twenty-four rules")

#block(width: 100%)[
  #set text(size: 9pt)
  #table(
    columns: (auto, auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 4pt),
    align: (center, left, left, center),
    [*rule*], [*subject*], [*what it requires*], [*ch.*],
    [A], [atomicity], [Everything is counted in ticks. No other unit is fundamental — not the second, not the beat.], [—],
    [*B*], [fixed 64-byte canonical binary], [Big-endian, never minimal, so byte order is numeric order and the format never has to change.], [6],
    [*C*], [body parameter provenance], [Epoch, secular rate, validity window, and the as-measured value. Outside the window, a warning rather than extrapolation.], [8],
    [D], [two text forms, one value], [The human form and the digit form encode the same integer, each declaring its own anchor.], [—],
    [E], [integrality], [Not in a signature, a field, an intermediate, or the rendering path. A lint enforces it.], [—],
    [*F*], [the frame is declared], [The reference frame is stated rather than assumed. What the system does not model, it says it does not model.], [16],
    [*G*], [the tier grid], [Units are 5^(5k) ticks. A timestamp is the tick count written in base 5 and grouped in fives.], [4],
    [I], [UCID range and non-uniqueness], [52 characters below 2^256, sortable, containing no randomness. Not a UUID.], [—],
    [*J*], [the anchor], [Phase cannot be derived from period, so it is supplied per body — and its absence is an error, never a default.], [8],
    [*K*], [one derivation mechanism], [Every calendar is built by the same path from a body's own periods. There is no Earth branch.], [8],
    [L], [leap seconds at the boundary only], [TT is the only pivot. No arithmetic on absolute time ever meets a leap second.], [—],
    [*M*], [monotone total order], [Of any two instants, exactly one of earlier, same, later holds.], [5],
    [*N*], [names are display-only], [A tier's identity is its exponent. Nothing decides behaviour from a name.], [4],
    [O], [overflow is a typed error], [Arithmetic never wraps and never saturates, in release builds as well as debug.], [—],
    [*P*], [profile binding], [Two timestamps from different declared constants cannot be compared, and the text says which is which.], [6],
    [*Q*], [the datum is stipulated], [Tick 0 is declared, not measured or observed. The physical claim about it is recorded separately and cannot be computed with.], [3],
    [R], [rounding only on rendering], [Values round when displayed, never when constructed, and always under a mode the caller names.], [—],
    [*S*], [sort order], [Lexicographic order equals chronological order for the fixed-width forms — and not for text.], [6],
    [T], [truncation is uncertainty], [A value printed to a coarser tier *is* an interval, not a point padded with zeros.], [—],
    [U], [interval arithmetic], [Operations on intervals return intervals. A midpoint is a rendering choice, not a measurement.], [—],
    [W], [one domain across backends], [The fixed-width and arbitrary-precision integers enforce one identical domain.], [—],
    [*X*], [certified enclosures], [An interval that provably contains the answer, with quadrature error and parameter uncertainty reported separately.], [5],
    [*Y*], [metrology], [Earth units cross one declared boundary, and never appear in the arithmetic.], [7],
    [*Z*], [zero and the unsigned domain], [Nothing precedes the datum. A result that would be earlier is an error, not a negative number.], [3],
  )
]

#section("Non-goals and failure modes")

The specification numbers what it refuses (`N`) and what it is built to prevent
(`F`). The book cites four.

#block(width: 100%)[
  #set text(size: 9pt)
  #table(
    columns: (auto, auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 4pt),
    [*id*], [*kind*], [*what it says*], [*ch.*],
    [`N1`], [non-goal], [The tick is not claimed to be a quantum of time. It is the resolution floor of an instrument.], [2],
    [`N12`], [non-goal], [No time before the datum. The value is not representable and the request is refused.], [3],
    [`F1`], [failure mode], [Timestamps shifting when the age constant is revised — what Rule P prevents.], [11],
    [`F9`], [failure mode], [Earth becoming the template rather than an instance — what Rule K prevents.], [14],
  )
]

#section("Specification corrections")

Where verification found the specification wrong, the correction carries a
`D-A` number. Chapter 9 is the account; `spec/SPEC-DELTAS.md` is the record.

#block(width: 100%)[
  #set text(size: 9pt)
  #table(
    columns: (auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 4pt),
    [*delta*], [*what changed*], [*ch.*],
    [`D-A4`], [Appendix C's human forms are truncated at T−5, not tick-exact], [6],
    [`D-A5`], [grouping cycles are declared per body, not admitted by a global bound], [9, 16],
    [`D-A7`], [full-width encode is 45 divmod steps, not 44], [9],
    [`D-A8`], [what a printed form means, and how each form is anchored], [6, 9],
    [`D-A11`], [obliquity is an angle and cannot be a rated parameter], [9],
    [`D-A12`], [§9.6's synodic formula computes the wrong quantity], [9, 16],
    [`D-A13`], [a drift bound is a rate in local units, not a duration], [9, 15],
    [`D-A14`], [§10.3's integral is improper and cannot be quadratured as written], [9],
  )
]

#callout(label: "Three the book found in itself")[
  Chapter 20 found the artifact assuming that periods have natures from which
  their behaviour follows. Chapter 25 found it assuming a clean line between a
  structure and a reading of it. Chapter 26 found the *book* assuming that the
  code is the invariant and the traditions the variables.

  None of the three is a rule. All three are commitments the artifact and the
  book make without declaring, and there is no `N` or `F` number for them —
  which is itself a gap in the scheme.
]

