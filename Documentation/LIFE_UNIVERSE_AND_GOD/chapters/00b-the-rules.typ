#import "../design.typ": *

#pagebreak(weak: true)

#v(1cm)
#align(center)[
  #text(font: body_family, size: 20pt, weight: "bold", fill: ink_black,
    "The rules, in one page")
]
#v(6mm)

The software this book is about is specified by twenty-four rules, each named
by a letter. The book cites them constantly and from chapter 2 onward, so they
are here at the front rather than only in the back.

You do not need to learn these. Read past them on first encounter and come back
when a chapter leans on one — that is what this page is for.

#v(3mm)
#block(width: 100%)[
  #set text(size: 8.5pt)
  #set par(justify: false)
  #table(
    columns: (auto, 34%, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 4pt, y: 3.4pt),
    align: (center, left, left, center),
    [*rule*], [*name*], [*what it requires*], [*ch.*],
    [*Q*], [the datum is stipulated], [declared, not measured — and not computable with], [3],
    [*Z*], [time is unsigned], [the domain begins at the datum; earlier is not representable], [3],
    [*G*], [the tier grid], [units are powers of five; a timestamp is base 5], [4],
    [*N*], [names are display only], [a tier's identity is its exponent, not its name], [4],
    [*P*], [profiles are tagged and type-bound], [two profiles' values cannot be compared], [6],
    [*K*], [one mechanism; Earth is an instance], [every calendar is built by the same path], [8],
    [*J*], [the anchor is declared and required], [phase is supplied per body; absence is an error], [8],
    [*C*], [body parameters carry provenance], [epoch, rate, window — outside it, a warning], [8],
    [*X*], [certified enclosures], [an interval, with its two error sources kept apart], [5],
    [*Y*], [metrology], [foreign units cross one boundary you can point at], [7],
    [*B*], [fixed 64-byte binary], [big-endian, never minimal, so the format never changes], [6],
    [*S*], [sort order], [byte order is chronological order — for binary and UCID], [6],
    [*M*], [order is total and monotone], [of any two instants, exactly one order holds], [5],
    [*F*], [the frame is declared], [what it does not model, it says it does not model], [16],
  )
]

#v(3mm)

Ten further rules govern parts of the software the book does not discuss.
Appendix D lists all twenty-four, together with the non-goals, failure modes
and specification corrections cited in the text. The normative statements are
in `spec/RULES.md` in the source tree.

#callout(label: "Why letters")[
  Because the specification names them that way, and renaming them for the book
  would make every citation in the source tree — there are 538 — resolve to
  nothing.

  The letters are not mnemonic and the specification does not pretend they are.
  `Q` is the datum rule because it was the seventeenth thing written down.
]

