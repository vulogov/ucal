#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 7, title: "The bridge")

Chapter 1 said the design's whole question was whether Earth enters *the arithmetic*
or at *a declared boundary you can point to*. This chapter is the boundary.

#section("One constant")

Earth enters the system through exactly one number.

#terminal(caption: "the bridge constant")[
```
SECOND = 18 548 584 399 861 000 000 000 000 000 000 000 000 000 000 ticks
```
]

That is how many ticks are in one SI second. It is an exact integer — declared, not
computed — and it is the only place in the workspace where a foreign unit is named
at all. A lint fails the build if any identifier in the core crate mentions a foreign
unit outside the bridge declaration.

#claim("interpretation")[
  The number of things this constant is *not* doing is the point.

  It is not a conversion factor applied throughout the code. It is not a scale on the
  tick. It does not appear in any arithmetic that produces a timestamp from another
  timestamp. Every operation the system performs on absolute time is integer addition,
  subtraction and comparison on tick counts, and none of them has ever heard of a
  second.

  `SECOND` is consulted when a foreign value arrives and when one leaves. In between,
  it is inert.
]

#section("Why the direction matters")

Converting *into* absolute time is multiplication by an integer. Multiplication of
integers is exact. So a whole number of seconds becomes a whole number of ticks with
no rounding whatsoever, ever, under any circumstances.

Converting *out* is division, which is where rounding lives — and so the rendering
path is the only place a rounding mode is chosen, and it is always chosen explicitly
by the caller.

#v(3mm)
#align(center, cetz.canvas({
  import cetz.draw: *
  let box(x, y, w, h, label, sub, fill) = {
    rect((x, y), (x + w, y + h), fill: fill, stroke: 0.5pt, radius: 0.05)
    content((x + w / 2, y + h / 2 + 0.13), text(size: 8.5pt, weight: "bold", label))
    content((x + w / 2, y + h / 2 - 0.16), text(size: 7pt, fill: luma(90), sub))
  }
  box(-4.6, 0, 2.5, 0.85, "civil label", "2026-07-31T19:11 TT", luma(238))
  box(-0.9, 0, 1.8, 0.85, "SECOND", "exact integer", rgb("#f6f1e6"))
  box(2.3, 0, 2.5, 0.85, "tick count", "unsigned integer", luma(238))

  // in: exact
  line((-2.1, 0.60), (-0.9, 0.60), mark: (end: "straight"), stroke: 0.7pt)
  line((0.9, 0.60), (2.3, 0.60), mark: (end: "straight"), stroke: 0.7pt)
  content((0, 0.98), text(size: 7.5pt, "× SECOND — exact, never rounds"))

  // out: rounds
  line((2.3, 0.25), (0.9, 0.25), mark: (end: "straight"), stroke: (thickness: 0.7pt, dash: "dashed"))
  line((-0.9, 0.25), (-2.1, 0.25), mark: (end: "straight"), stroke: (thickness: 0.7pt, dash: "dashed"))
  content((0, -0.12), text(size: 7.5pt, "÷ SECOND — rounds, under a mode the caller names"))

  // the arithmetic region
  rect((2.3, -1.35), (4.8, -0.55), stroke: (dash: "dotted", thickness: 0.5pt))
  content((3.55, -0.95), text(size: 7.5pt, "+ − < ="))
  line((3.55, -0.05), (3.55, -0.55), stroke: 0.4pt)
  content((5.0, -0.95), anchor: "west", text(size: 7.5pt, fill: luma(90),
    "all arithmetic happens here,"))
  content((5.0, -1.22), anchor: "west", text(size: 7.5pt, fill: luma(90),
    "and never sees a second"))
}))
#v(1mm)
#figcap[9][
  The bridge. Earth enters and leaves at one declared constant; the arithmetic in
  between is integer operations on tick counts.
]

#section("Refusing rather than rounding")

Ask the system to accept a time finer than the bridge constant can represent and it
does not round. It returns `UCAL-E0043` — *foreign-unit input finer than the bridge
constant permits*.

The threshold is $10^(-30)$ seconds, and it is not arbitrary: it is where the
declared constant's own decimal precision runs out. Below it, the system does not
know the answer, and it says so instead of inventing one.

#callout(label: "The pattern, again")[
  This is the third time in three chapters. Before the datum: refuse the question
  (`UCAL-E0020`). Beyond the UCID range: refuse to truncate (`UCAL-E0031`). Finer than
  the bridge: refuse to round (`UCAL-E0043`).

  A system that answered all three would be more convenient and would be lying in
  three different ways.
]

#section("The invariants that fall out free")

Here is the part that makes the design feel less like a set of choices and more like
a consequence.

`SECOND` was set to a multiple of $10^30$. Because $10^30 = 2^30 dot 5^30$, that
means `SECOND` carries thirty factors of five. The nanosecond — a thousandth of a
millionth of it — carries twenty-one.

So:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    [*quantity*], [*factors of 5*], [*consequence*],
    [`SECOND`], [30], [a whole second lands with 30 trailing base-5 zeros],
    [`NANOSECOND`], [21], [a whole nanosecond lands with 21 trailing base-5 zeros],
    [`ORIGIN_OFFSET`], [61], [the bridge epoch is zero in every tier below T0],
  )
]
#v(2mm)

Read that in the notation of chapter 6 and it means something concrete: *any instant
that came from a wall clock has a long tail of zeros in its digit form*. Six full
tier groups of them, for a nanosecond clock.

#claim("interpretation")[
  These are excellent invariant tests, and the project uses them as such — they are
  cheap to check and they fail loudly if the bridge is ever mis-declared.

  They are also the reason chapter 6 could say the UCID is not merely non-random but
  *structurally constrained*. The zeros are not a coincidence about particular
  timestamps; they are a theorem about every timestamp that entered through the
  bridge. An identifier whose low-order digits are provably zero is not a candidate
  for a unique key, and it is better to know that from the invariant than to discover
  it from a collision.
]

#section("One pivot, and where the leap seconds live")

Civil time is a mess, and the design's response is to confine the mess.

There is exactly one pivot scale: *Terrestrial Time*. TT is a uniform scale with no
leap seconds, which is precisely why it was chosen — a timestamp converted through TT
never has to know about the irregular ones.

Leap seconds exist, and the system knows all twenty-seven of them. They are applied
at the parse and format boundary and nowhere else. No arithmetic on absolute time
ever encounters one.

#terminal(caption: "ucal doctor — the leap table")[
```
leap_seconds:
  table_version     IERS Bulletin C 70 (no leap second to 2026-06-30)
  entries           27
  complete_through  2026-06-30
  pre_1972          the 1961-1972 rubber-second era is modelled exactly;
                    UTC before 1961-01-01 is UCAL-E0041
  network           never; the table is bundled and offline
```
]

Two details in that output are worth pausing on.

*The table is bundled and offline.* No command in this system performs network
access, ever. A time library that phones home is a time library whose answers depend
on whether the network was up.

*The rubber-second era is modelled exactly.* Between 1961 and 1972, UTC did not tick
at the same rate as TAI — it ran at declared fractional offsets that changed
periodically, and the offsets were not whole seconds. Most software declines to
handle this and starts its UTC support in 1972.

#claim("interpretation")[
  It turns out to be exactly representable in this system, and the reason is a small
  arithmetic accident worth recording. The era's rate coefficients — 0.001296,
  0.0011232, 0.002592 seconds per day — are all divisible by 27. A day is 86,400
  seconds, which carries $3^3 = 27$. The 27s cancel, and what is left is exact in the
  rationals the system already uses.

  Nobody designed that. It was discovered while implementing, and it meant an era that
  could have been declared out of scope became eleven years of exactly convertible
  history instead. Before 1961 there is nothing to convert exactly, and the system
  returns `UCAL-E0041` rather than guessing.
]

#section("What the bridge costs")

Honesty, since Part III is about what implementation refused.

The bridge is where the claim "no Earth content" is at its weakest. `SECOND` is an
Earth-derived constant sitting in the core crate, and the tick's *length* is fixed by
convention against it, as chapter 2 admitted.

What the design achieves is not the elimination of that dependency — that is not
available to anyone — but its *localisation*. There is one constant, in one declared
place, named by one identifier, with a lint that fails the build if a second one
appears. The arithmetic is clean; the boundary is dirty; and the boundary is visible.

#claim("interpretation")[
  Whether that is enough is a fair question and this book does not think it is
  settled. What can be said is that the alternative designs are worse in a specific
  way: a system with conversion factors distributed through its operations has the
  same dependency and no place to point at.

  A dependency you can point at can be argued about. One that has been dissolved into
  the arithmetic cannot.
]

#recap((
  [Earth enters through one exact integer constant, `SECOND`, and a lint fails the build if the core names a foreign unit anywhere else.],
  [Conversion *in* is multiplication and never rounds; conversion *out* is division and is the only place a rounding mode is chosen.],
  [Input finer than $10^(-30)$ s is refused, not rounded — the third refusal in three chapters.],
  [Whole seconds land with 30 trailing base-5 zeros and whole nanoseconds with 21, which makes good invariant tests and explains why UCID has no entropy.],
  [TT is the only pivot; all 27 leap seconds live at the parse/format boundary, offline. The 1961–1972 rubber-second era turns out exactly representable, by accident.],
  [The bridge does not eliminate the Earth dependency. It localises it to one place you can point at and argue about.],
))
