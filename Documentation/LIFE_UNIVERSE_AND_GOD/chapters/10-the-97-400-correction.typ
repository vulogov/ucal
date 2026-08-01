#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 10, title: "The 97/400 correction")

This is the chapter the book exists for.

Everything else here is a claim that a program can do philosophical work. This chapter
is the evidence, and the evidence takes the only form that counts: the program
produced a result its author did not want, about a claim its author had published
twice, and the author printed it.

#section("The claim that was made")

Chapter 8 described the intercalation mechanism. A body's year is not a whole number
of its days; the fraction is expanded as a continued fraction; the convergents are the
best rational approximations, in the precise sense that no fraction with a smaller
denominator comes closer.

The specification, in two successive revisions, said this about it:

#claim("tradition")[
  The mechanism reproduces the Julian and Gregorian rules as convergents.
]

It is a satisfying sentence. It says the machinery, given nothing but Earth's rotation
and orbit, rediscovers the two calendars Western civilisation actually built. If true,
it would be the strongest possible demonstration that the derivation is not merely
consistent but *correct* — that it finds what people found, from first principles.

Half of it is true.

#section("What the mechanism actually returns")

#terminal(caption: "ucal cal show earth-d — the full walk")[
```
intercalation:
  whole_days_per_year  365
  rule                 31/128
  bound                1 local day per 10000 local years
  walked:
    1: 1/4        — 1 day slips in 128 local years
    2: 7/29       — 1 day slips in 1234 local years
    3: 8/33       — 1 day slips in 4269 local years
    4: 31/128     — 1 day slips in 400000 local years   <- chosen
    5: 752/3105   — 1 day slips in 62100000 local years
    6: 4543/18758 — 1 day slips in 937900000 local years
    7: 9838/40621 — 1 day slips in 4062100000 local years
    8: 24219/100000 — 1 day slips in never (exact) local years
```
]

Convergent 1 is *1/4*. One leap day every four years. That is the Julian calendar,
derived from Earth's motion with no knowledge of Rome, and the first half of the claim
is confirmed exactly.

Now look for `97/400`.

It is not there. It is not at depth 4, where the chosen rule sits. It is not at depth
8, where the expansion terminates because the parameter is exact. *It is not at any
depth*, and there is a check in the constants harness whose only job is to assert its
absence.

#section("Why it cannot be there")

This is not an accident of where the walk stopped. It is a theorem.

The convergents of a continued fraction have a defining property: convergent $p\/q$ is
the *best* rational approximation with denominator at most $q$. Nothing with a smaller
or equal denominator gets closer. So if `97/400` were a convergent, no fraction with a
denominator below 400 could beat it.

#block(breakable: false, width: 100%)[
#v(2mm)
#align(center, cetz.canvas({
  import cetz.draw: *
  // log-scale ladder: x = log10(denominator), y = log10(1/error)
  let pts = (
    (4,     128,        "1/4"),
    (29,    1234,       "7/29"),
    (33,    4269,       "8/33"),
    (128,   400000,     "31/128"),
    (3105,  62100000,   "752/3105"),
  )
  let X = d => calc.log(d) * 1.55
  let Y = s => (calc.log(s) - 2) * 0.62
  // axes
  line((0, 0), (6.4, 0), stroke: 0.5pt, mark: (end: "straight"))
  line((0, 0), (0, 4.6), stroke: 0.5pt, mark: (end: "straight"))
  content((3.2, -0.45), text(size: 8pt, "denominator (log)"))
  content((-0.35, 2.3), angle: 90deg, text(size: 8pt, "years before 1 day slips (log)"))
  // the convergent ladder
  let prev = none
  for (d, s, name) in pts {
    let p = (X(d), Y(s))
    if prev != none { line(prev, p, stroke: 0.5pt + luma(150)) }
    circle(p, radius: 0.075, fill: black, stroke: none)
    content((p.at(0) + 0.12, p.at(1) + 0.2), anchor: "west", text(size: 7.5pt, name))
    prev = p
  }
  // the Gregorian, off the ladder
  let g = (X(400), Y(3226))
  circle(g, radius: 0.1, fill: rgb("#8a3a3a"), stroke: none)
  content((g.at(0) + 0.16, g.at(1) - 0.05), anchor: "west",
    text(size: 8pt, weight: "bold", fill: rgb("#8a3a3a"), "97/400"))
  // the vertical gap to the ladder at the same denominator
  line(g, (X(400), Y(400000) - 0.55), stroke: (dash: "dotted", thickness: 0.6pt,
    paint: rgb("#8a3a3a")))
  content((X(400) + 0.16, (g.at(1) + Y(400000)) / 2 - 0.3), anchor: "west",
    text(size: 7pt, fill: rgb("#8a3a3a"), "124× worse than 31/128,"))
  content((X(400) + 0.16, (g.at(1) + Y(400000)) / 2 - 0.55), anchor: "west",
    text(size: 7pt, fill: rgb("#8a3a3a"), "with a 3× larger denominator"))
}))
#v(1mm)
#figcap[6][
  Earth's convergent ladder, and the Gregorian rule off it. Every point on the line is
  the best approximation available at its denominator. `97/400` is not on the line.
]
]

Two fractions with smaller denominators beat it:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    align: (left, right, right, left),
    [*rule*], [*denominator*], [*1 day slips in*], [*against 97/400*],
    [8/33], [33], [4,269 yr], [1.32× more accurate, denominator 12.1× smaller],
    [31/128], [128], [400,000 yr], [*124× more accurate*, denominator 3.1× smaller],
    [97/400], [400], [3,226 yr], [— the Gregorian rule],
  )
]
#v(2mm)

`31/128` is a hundred and twenty-four times more accurate than the Gregorian rule
while being a simpler fraction. That is the finding, and it is not close.

#section("What that means, and what it does not")

It does not mean the Gregorian calendar is bad. The reform of 1582 solved the problem
it was aimed at — Easter drifting against the equinox — and it solved it well enough
to still be in use four and a half centuries later. Accuracy against the tropical year
was one constraint among several, and it was not the binding one.

It does not mean 97/400 was a mistake. A rule of 400 years is expressible as "every
four years, except centuries, except every fourth century," which a person can apply
without arithmetic. `31/128` cannot be stated that way at all.

What it means is precisely this: *the Gregorian rule is a declared table, not a
derivation.* Chapter 8's classification — legacy versus derived — is not a value
judgement dressed up as taxonomy. It is a factual claim about where a rule's
authority comes from, and here is the case where it bites.

#claim("interpretation")[
  The interesting thing is not that the specification was wrong about a fraction. It is
  that the specification was wrong *in the direction of its own thesis*.

  A machinery that rediscovers both historical calendars is a better story than one
  that rediscovers one and quietly demonstrates the other was never derived. The
  sentence was not a careless error; it was the error the argument wanted to be true.
  It survived two revisions because nobody checked the half that was flattering.
]

#section("Maimonides' charge")

There is a nine-hundred-year-old objection to systems like this one, and it is worth
stating in its own words because the correction above is the only kind of answer it
admits.

In the *Guide of the Perplexed* I.73, Maimonides considers the Kalām practice of
constructing metaphysical premises and observes — about people he otherwise treats
with respect — that their premises were not derived from the world but assembled to
make a desired conclusion provable. They knew where they were going and built the road
backwards from it.

The charge generalises immediately to any formal system built by someone with a thesis.
A calendar that derives calendars, built by a person who believes derivation is
superior to declaration, is exactly the sort of thing that could be tuned until it
produced the flattering result. The drift bound could be adjusted. The parameter could
be chosen. The depth could be stopped where the answer looked good.

#claim("interpretation")[
  You cannot answer that charge with assurance, because assurance is what it is about.
  You can only answer it with evidence, and the evidence has to be a case where the
  system contradicted its builder on something he had already committed to in public.

  This is that case. The claim was published twice. The mechanism said no. The
  mechanism's answer was recorded, the two revisions that said otherwise are named as
  wrong, and the harness now contains an assertion whose only purpose is to keep the
  correction from being quietly reversed.

  That is not proof of good faith — nothing is. It is the strongest available
  substitute: a system that has demonstrably been permitted to answer back.
]

#section("What was done about it")

Four things, and the fourth is the one that matters in five years.

The specification's sentence was corrected rather than deleted, and the revisions that
carried it are named.

The mechanism was left alone. No bound was adjusted, no parameter retuned, no depth
extended in the hope that `97/400` would appear further along. It does not appear
further along; the expansion terminates at depth 8 because Earth's parameter is exact
in the system, and the fraction is absent from all eight.

`earth-civil` was reclassified as a legacy calendar, and its leap rule carries the
label in the tooling's own output:

#terminal(caption: "ucal cal list")[
```
earth-civil:
  kind       legacy — declared tables (§8.6)
  arbitrary  4
  leap_rule  97/400 (NOT a convergent — declared, not derived)
```
]

And a check was added to the constants harness asserting that `97/400` is absent from
Earth's convergents at every depth. It runs on every build. If someone ever changes a
parameter in a way that makes the flattering claim true, the build will say so — which
is the only form of vigilance that survives the author losing interest.

#recap((
  [The specification claimed twice that the mechanism reproduces the Julian *and* Gregorian rules as convergents.],
  [The Julian rule is convergent 1, exactly as claimed. `97/400` is not a convergent at any depth, and it cannot be: two simpler fractions beat it.],
  [`31/128` is 124× more accurate with a denominator 3.1× smaller; `8/33` beats it with a denominator 12.1× smaller.],
  [This is not a criticism of the Gregorian reform. It is the factual content of the legacy/derived distinction — where a rule's authority comes from.],
  [The error ran in the direction of the author's own thesis, which is why it survived two revisions.],
  [Maimonides' charge in *Guide* I.73 admits only one kind of answer: a case where the system contradicted its builder in public. This is that case, and a build-time assertion now keeps the correction from being reversed.],
))
