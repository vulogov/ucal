#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 6, title: "Notation")

One value, four ways of writing it down. This chapter is about why that is not
redundancy, and about what each form is for.

#terminal(caption: "ucal explain — one instant, four notations")[
```
ticks     8070205189128471254993117657693008777530466139316558837890625

human     UC1 0031·0687·2481·3000·1638·3018:0779·2671·2006·1837·…

digit5    UC1/5 00000.00000. … .00111.10222.34411.44000.23023. …

ucid      0000000000050PM6K45MKCAVY5MPYAMHCJQ142JHE26A2ZAJ9FJ1
```
]

#section("Two text forms, and why both")

The *human form* writes each tier group as a decimal number. The *digit form*
writes the underlying base-5 digits directly, five to a group.

They encode the same integer. The difference is who is reading.

A person reading `0031·0687·2481` can see at a glance that this is 31 deeps, 687
drifts, 2481 spans — three quantities in a unit system, each written the way people
write numbers. The same prefix in the digit form is `00111.10222.34411`, which is
correct, canonical, and unreadable.

#claim("interpretation")[
  The temptation is to call the human form a convenience wrapper and the digit form
  the real one. That is not quite right, and the distinction matters for Part VIII.

  The digit form is canonical *for machines* — parsing, sorting, round-tripping. The
  human form is canonical *for statements about time*, because tier groups are the
  units the system actually reasons in. Neither is a rendering of the other; both are
  renderings of the integer.
]

Each form carries a tag — `UC1` or `UC1/5` — so a value never travels without
declaring which profile and which notation produced it. That is Rule P doing its
work at the notation layer: two timestamps from different profiles cannot be
compared by accident, because the text itself says they are different things.

#section("Where each form anchors")

This is the detail that most repays care, and it is the subject of one of the
corrections this project made against its own specification.

A form printed to fewer groups is *less precise*, not *padded with zeros*. Chapter 4
called this "truncation is rounding". But saying that precisely requires knowing
which tier each form counts from.

The human form anchors at *T0*, the beat: the group immediately before the colon.
The digit form anchors at *T32*, the top of the ladder. So the same act of dropping
trailing groups means the same thing in both — you have said less — but the tier a
given group sits at is read from a different reference.

#callout(label: "D-A8 — an amendment the implementation forced")[
  The specification originally described the anchoring loosely enough that two
  readers could disagree about what a truncated form denoted. Implementation made
  the ambiguity concrete: the human form *cannot express a precision coarser than
  T0*, because it has no groups above the colon to drop.

  That is not a defect. It is a consequence of anchoring at the beat, and it is why
  the command line falls back to a different form when asked to render a timeline at
  a tier above T0 — rather than printing a blank field, which is what the first
  implementation did and which looked exactly like a value.
]

#section("The canonical binary form")

Sixty-four bytes, big-endian, fixed width. Never length-prefixed, never minimal,
never a varint.

The width is not negotiable and the reason is worth stating: *byte order is numeric
order*. A fixed-width big-endian encoding sorts lexicographically into chronological
order, so the encoding is directly usable as a database key or a sort key with no
comparator at all.

A minimal encoding — the obvious space optimisation — would break that, and it would
break something else: the wire format would depend on the *magnitude* of the value
rather than on the profile. Two timestamps of different sizes would have different
lengths, and a reader would have to know the value to know how to read it.

#claim("interpretation")[
  Sixty-four bytes for a timestamp is extravagant by any ordinary standard. Unix time
  fits in eight.

  What the extra fifty-six buy is that the format never has to change. The domain is
  512 bits, the encoding is 512 bits, and there is no future in which a stored
  timestamp becomes unreadable because the range was widened. That is a real cost
  paid once against a class of migration that software normally pays repeatedly.
]

#section("UCID: the transcribable form")

The fourth notation is a 52-character identifier in Crockford base-32.

#terminal(caption: "the alphabet")[
```
0123456789ABCDEFGHJKMNPQRSTVWXYZ
```
]

`I`, `L` and `O` are absent because they are confusable with `1` and `0`. `U` is
absent to avoid accidental obscenity. The alphabet is strictly ascending in ASCII —
and there is a test that checks that rather than trusting it — which is what makes
lexicographic order equal numeric order here too.

#subsection("Why 256 bits and not 512")

Because the UCID is meant to be read, typed, and quoted by a person, and the domain
is not.

At five bits per character, covering the full 512-bit domain would take *103
characters*. At 256 bits it takes 52, which is 260 bits of capacity — so the leading
character encodes a single significant bit and is always `0` or `1`.

#block(breakable: false, width: 100%)[
#v(2mm)
#align(center, cetz.canvas({
  import cetz.draw: *
  // 0.05 cm per character keeps the 103-character bar inside the B5 text block.
  let w = 0.05
  let bar(y, n, fill, label, sub) = {
    rect((0, y), (n * w, y + 0.34), fill: fill, stroke: 0.4pt)
    content((n * w + 0.18, y + 0.17), anchor: "west",
      text(size: 8pt, weight: "bold", label))
    content((n * w + 0.18, y - 0.09), anchor: "west",
      text(size: 7pt, fill: luma(100), sub))
  }
  bar(0.0, 103, luma(230), "103 characters", "the full 512-bit domain — unusable")
  bar(-0.75, 52, luma(160), "52 characters", "2^256 — the UCID")
  bar(-1.5, 13, luma(60), "13 characters", "2^64, for scale")
}))
#v(1mm)
#figcap[3][
  UCID length against range. Five bits per character, so width is linear in bits —
  and the domain is twice the bits.
]
]

The ceiling that buys is $2^256$ ticks, about $1.98 times 10^26$ years. That is past
the end of the stelliferous era and thirty orders of magnitude beyond the present
epoch — and far short of the domain's $2.29 times 10^103$ years.

Above the ceiling you do not get a truncated identifier. You get `UCAL-E0031`,
*instant outside UCID range*. The same refusal-rather-than-approximation that runs
through the whole design.

#subsection("What UCID is not")

The source documentation is emphatic here, and so is this book.

*UCID contains no randomness.* It is a pure function of the instant. Two events at
the same tick receive the same identifier, which makes it useless as a unique key
for concurrent events.

It is worse than merely non-random. Chapter 7 shows that an instant read from a
nanosecond clock has at least twenty-one trailing base-5 zeros by construction, so
the low-order characters are not just predictable but *structurally constrained* —
consecutive nanoseconds share more than twenty leading characters.

There is a test called `ucid_has_no_entropy` whose entire job is to measure how badly
the identifier would fail if someone used it as a UUID.

#claim("interpretation")[
  Writing a test that demonstrates your own type is unsuitable for an obvious misuse
  is an unusual thing to do. It costs nothing to omit; nobody would notice.

  It is here because the failure would be silent. A UCID *looks* like a UUID — same
  length class, same character set, same shape in a log line — and the moment someone
  reaches for it as one, the collisions begin and never announce themselves. A test
  that fails loudly at the boundary is cheaper than an incident.
]

#section("Four forms, one integer")

To close the chapter, the whole picture in one table.

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr, auto, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    align: (left, left, left, left),
    [*form*], [*for*], [*width*], [*sorts?*],
    [`ticks`], [the value itself, in decimal], [variable], [no],
    [human], [statements about time, by people], [variable], [only if padded],
    [`digit5`], [parsing, round-tripping, canonical text], [variable], [only if padded],
    [binary], [storage, wire, database keys], [64 bytes], [yes],
    [UCID], [quoting, transcribing, URLs], [52 chars], [yes],
  )
]
#v(2mm)

The two that sort are the two that are fixed-width. That is not a coincidence, it is
the whole mechanism, and it is why the text forms carry the caveat "only if padded"
rather than being quietly assumed to sort.

#recap((
  [Two text forms encode one integer: the human form in decimal tier groups, the digit form in raw base-5. Neither is a rendering of the other.],
  [Each form declares its anchor — human at T0, digit at T32 — which is what makes truncation mean *less precise* rather than *padded*.],
  [Canonical binary is 64 bytes, fixed-width, big-endian, so byte order is chronological order and the format never has to change.],
  [UCID is 52 characters over $2^256$ because 512 bits would need 103. Beyond the ceiling is `UCAL-E0031`, not a truncation.],
  [UCID carries no entropy and is not a UUID. A test measures exactly how badly it would fail as one, because that failure would otherwise be silent.],
))
