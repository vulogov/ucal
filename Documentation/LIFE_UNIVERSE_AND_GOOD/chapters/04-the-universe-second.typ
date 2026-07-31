#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 4, title: "The universe second")

A tick is far too small to think in. It is $5.4 times 10^(-44)$ seconds; the age of the
universe is about $8 times 10^(60)$ of them. Nobody reads a 61-digit number.

So there is a ladder of larger units above the tick, and it has an unusual property: every
rung is a whole power of five of the one below, and the whole ladder is a whole power of
five of the tick. That property is what makes the notation work, and this chapter is about
what it buys and what it costs.

#term("Beat")[
  $5^60$ ticks, about 46.762 milliseconds. The reference rung of the ladder, and what the
  specification calls the *universe second* — a unit of human-noticeable size with no Earth
  content whatever.
]

#section("Why base five")

Because $5^5 = 3125$, and 3125 is a number of exactly five base-5 digits.

That is the entire reason. It is worth being blunt about it, because a book that spends its
second half among Pythagoreans and Neoplatonists has an obligation to say clearly where its
numbers came from, and this one came from a digit-packing convenience.

#callout(label: "Rule N")[
  No constant, base, or magnitude in this system acquires significance by resembling a
  number in a tradition. Not the five, not the sixty in $5^60$, not the 3125.

  The research did, as it happens, go looking. The first place it looked — Losev on the
  Neoplatonic tetrad — had a five waiting, and the Pythagorean tetraktys sums to ten and
  builds from four. Those are interesting facts about those texts. They are not evidence
  about this software, they were not inputs to it, and the book reports them once, in Part
  VI, labelled as what they are.
]

#section("The ladder")

Each rung is $5^5 = 3125$ of the rung below, which is to say each rung is exactly five
base-5 digits wide. Rung $k$ is $5^(60 + 5k)$ ticks, indexed from the beat at $k = 0$.

#v(3mm)
#align(center, cetz.canvas({
  import cetz.draw: *
  let rows = (
    ("T5", "deep",    "441.6 Myr"),
    ("T4", "drift",   "141.3 kyr"),
    ("T3", "span",    "45.2 yr"),
    ("T2", "sweep",   "5.285 d"),
    ("T1", "arc",     "146.1 s"),
    ("T0", "beat",    "46.762 ms"),
    ("T−1", "flicker", "14.96 µs"),
    ("T−2", "glint",  "4.79 ns"),
  )
  let y = 0
  for (tier, name, size) in rows {
    let bold = (name == "beat")
    line((0, y), (0.55, y), stroke: (thickness: if bold { 1.4pt } else { 0.6pt }))
    content((0.75, y), anchor: "west",
      text(size: 8pt, weight: if bold { "bold" } else { "regular" }, tier))
    content((1.8, y), anchor: "west",
      text(size: 8pt, style: "italic", weight: if bold { "bold" } else { "regular" }, name))
    content((3.6, y), anchor: "west", text(size: 8pt, fill: luma(80), size))
    y = y - 0.52
  }
  line((0, 0.35), (0, y + 0.32), stroke: 0.8pt)
  content((0, 0.62), text(size: 7.5pt, fill: luma(110), "coarser"))
  content((0, y + 0.06), text(size: 7.5pt, fill: luma(110), "finer"))
}))
#v(1mm)
#figcap[2][
  The named rungs of the tier ladder. Each is 3125 of the one below. The ladder continues
  in both directions; only the named tiers are shown.
]

#section("What the ladder buys")

Three things, and they are all consequences of the same fact — that a timestamp is the tick
count written in base 5 and grouped in fives.

*Truncation is rounding.* Write fewer groups and you have said the same thing less
precisely. There is no separate rounding step, no scaling, no loss of a different kind: the
digits you dropped are exactly the precision you gave up. This is the property that makes
the uncertainty discipline of Part II possible at all.

*Prefix comparison is chronological comparison.* Two timestamps compare in the order their
digits compare, because base-5 positional notation is monotone. Sorting the text sorts the
times.

*Writing all the digits pinpoints one tick.* There is no accumulated conversion error
between the coarse view and the fine one, because they are the same integer read at
different widths.

Here is one real instant, at full precision and then at three coarser tiers:

#terminal(caption: "ucal explain — one instant, four precisions")[
```
ticks     8070205189128471254993117657693008777530466139316558837890625

human     UC1 0031·0687·2481·3000·1638·3018:0779·2671·2006·1837·…
tiers:
  T5 deep   31
  T4 drift  687
  T3 span   2481
  T2 sweep  3000
  T1 arc    1638
  T0 beat   3018
```
]

The `0031` at the front is deeps since the datum. The `3018` before the colon is beats
within the current arc. Every group is a base-5 number written in decimal, which is why
none of them exceeds 3124.

#claim("interpretation")[
  There is something worth noticing about that colon. It sits at the beat — at $5^60$ — and
  it is the only place in the notation where a human convenience has been allowed in. It is
  there because the beat is roughly the scale at which a person can notice a duration, so
  it is the natural place to put the decimal point of a time system built for beings who
  perceive at that scale.

  That is a concession to the reader, not to Earth. It changes no arithmetic.
]

#section("What the ladder costs")

Nothing on this ladder is near anything you know.

A second is 21.385 beats — not a whole number, and not close to one. An hour is 24.6 arcs.
A day is 0.189 sweeps. A year is 0.699 spans.

#callout(label: "The two seconds do not divide")[
  This is the sharpest form of the cost, and the system says so on its own ladder output:

  #v(1.5mm)
  #text(size: 9.5pt)[
    The two seconds are incommensurable above T−6. One bridge second is 21.385061835 beats,
    not a whole number, because `BEAT` carries $5^60$ while `SECOND` carries only $5^30$.
    They share a common measure only at the tick.
  ]
  #v(1.5mm)

  So the beat is not a second in disguise, and no amount of rescaling would make it one.
  They agree at the tick and nowhere above it — which is exactly why the tick is primitive
  rather than either of them.
]

That is the honest consequence of leaving the Earth paradigm. If you want units with no
planetary content, you do not get to keep the hour. The hour *is* planetary content.

#section("The mark")

The project's own mark is a diagram of this chapter, and it is worth reading as one.

#v(3mm)
#align(center, cetz.canvas({
  import cetz.draw: *
  let R = 2.0
  // rim
  circle((0, 0), radius: R, stroke: 1.2pt)
  // tick band: 25 medium divisions, 5 heavy
  for i in range(0, 25) {
    let a = 90deg - i * 14.4deg
    let heavy = calc.rem(i, 5) == 0
    let r0 = if heavy { R - 0.28 } else { R - 0.16 }
    line((calc.cos(a) * r0, calc.sin(a) * r0),
         (calc.cos(a) * R, calc.sin(a) * R),
         stroke: (thickness: if heavy { 1.1pt } else { 0.4pt }))
  }
  // elapsed sector, datum at top running clockwise
  let hand = 90deg - 137deg
  arc((0, 0), start: 90deg, stop: hand, radius: R - 0.42, anchor: "origin",
    mode: "PIE", fill: luma(20), stroke: none)
  // datum core, knocked out
  circle((0, 0), radius: 0.17, fill: white, stroke: 1.1pt)
  // zero notch
  line((-0.11, R + 0.1), (0.11, R + 0.1), (0, R + 0.3), close: true, fill: black)
  // hand
  line((0, 0), (calc.cos(hand) * (R - 0.5), calc.sin(hand) * (R - 0.5)), stroke: 1.4pt)

  // annotations
  content((0, R + 0.62), text(size: 7.5pt, "zero notch — the datum direction"))
  content((-2.55, 1.15), anchor: "east", text(size: 7.5pt, "tick band"))
  line((-2.5, 1.15), (-R * 0.72, R * 0.72), stroke: 0.4pt)
  content((2.7, -0.05), anchor: "west", text(size: 7.5pt, "hand on a fine tick"))
  line((2.65, -0.05), (calc.cos(hand) * (R - 0.5), calc.sin(hand) * (R - 0.5)), stroke: 0.4pt)
  content((-2.55, -1.3), anchor: "east", text(size: 7.5pt, "elapsed sector"))
  line((-2.5, -1.3), (-0.75, -0.85), stroke: 0.4pt)
  // The datum label goes left, away from the sector, which sweeps the upper right.
  content((-2.55, 0.05), anchor: "east", text(size: 7.5pt, "datum, knocked out"))
  line((-2.5, 0.05), (-0.19, 0.0), stroke: 0.4pt)
}))
#v(1mm)
#figcap[4][
  The mark, read as a diagram. Five heavy divisions on the tick band, not twelve — the
  base is visible at a glance. The centre is negative space.
]

The centre dot has its core knocked out. That is deliberate: the datum is stipulated rather
than observed, so the mark declines to draw anything there. The hand lands on a fine tick
rather than a coarse division, because the claim the instrument makes is that it can
pinpoint an arbitrary exact instant — not that it can round to a tidy one.

#claim("interpretation")[
  A mark that showed a filled centre would be making a claim the specification refuses. The
  identity was drawn after the rule existed, and it obeys it. That is a small thing, and it
  is the sort of small thing this project is made of.
]

#recap((
  [The beat is $5^60$ ticks ≈ 46.762 ms — the *universe second*, a human-scale unit with no Earth content.],
  [Base five because $5^5 = 3125$ is five base-5 digits wide. Rule N: no further meaning, and the book says so before Part VI can be misread.],
  [The uniform ladder makes truncation *be* rounding, prefix comparison *be* chronological comparison, and full precision pinpoint one tick.],
  [The cost is that nothing on the ladder is near a second or an hour — one second is 21.385 beats, and the two share a measure only at the tick.],
))
