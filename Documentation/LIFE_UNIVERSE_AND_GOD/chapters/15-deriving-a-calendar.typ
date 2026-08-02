#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 15, title: "Deriving a calendar")

Three bodies, one mechanism, end to end. Earth because it can be checked against a
wall calendar; Mars because it is a different planet with a different answer; Titan
because it is strange enough to break a mechanism that only looked general.

#section("Earth")

The full derivation appeared in chapter 8 and the finding it produced was chapter 10.
Here it is as one line of a comparison:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    align: (left, right, left, left),
    [*body*], [*days/year*], [*rule chosen*], [*at the default bound*],
    [Earth], [365.242190], [31/128 — conv. 4], [1 day slips in 400,000 yr],
    [Mars], [668.599], [45/76 — conv. 6], [1 sol slips in 16,566 Mars yr],
    [Titan], [—], [derived], [no anchor: phase unavailable],
  )
]
#v(2mm)

The Julian rule, 1/4, is Earth's first convergent. The Gregorian rule is not a
convergent at any depth. Both facts came out of the mechanism with no knowledge of
Rome or of 1582.

#section("Earth's months, and the Metonic cycle")

Earth names the Moon as its grouping satellite, so the mechanism derives cycles the
same way it derives intercalation — continued-fraction expansion, this time of the
ratio between the year and the synodic month.

#terminal(caption: "ucal cal show earth-d — cycles")[
```
cycles:
  satellite        moon
  cycles_per_year  12.368266523
  convergents:
    12/1
    25/2
    37/3
    99/8
    136/11
    235/19        <- the Metonic cycle
    4131/334
    8497/687
```
]

The sixth convergent is *235/19*: 235 lunar months in 19 years.

That is the Metonic cycle, named for an Athenian astronomer of the fifth century BC,
used by the Babylonians before him, and still the basis of the Hebrew calendar's
intercalation and of the computus that fixes Easter. It falls out of the ratio of
Earth's two periods, with nothing supplied but those periods.

#claim("interpretation")[
  This is the most satisfying single output the mechanism produces, and it is worth
  being careful about what it demonstrates.

  It does not show that Meton was doing continued fractions. He was not. It does not
  show that the mechanism has rediscovered a truth about the cosmos.

  What it shows is that 235/19 is *the good approximation at that denominator* — that
  anyone, in any century, with an accurate enough ratio and a motive to find a
  repeating cycle, is under pressure toward the same fraction. The convergents are
  where the good approximations are; Meton found one by observation and patience, and
  the algorithm finds the same one because there was nowhere else for either to land.

  That is a claim about the arithmetic of approximation, not about anticipation.
]

#section("Mars")

A different planet, run through the identical code path.

#terminal(caption: "ucal cal show mars-d — intercalation")[
```
intercalation:
  whole_days_per_year  668
  rule                 45/76
  bound                1 local day per 10000 local years
  walked:
    1: 1/1     — 1 day slips in 2 local years
    2: 1/2     — 1 day slips in 11 local years
    3: 3/5     — 1 day slips in 128 local years
    4: 13/22   — 1 day slips in 796 local years
    5: 16/27   — 1 day slips in 2342 local years
    6: 45/76   — 1 day slips in 16566 local years   <- chosen
    7: 106/179 — 1 day slips in 76079 local years
   ...
   13: 486756/821993 — 1 day slips in never (exact)
```
]

A Mars year is 668.599 sols, so the fraction to absorb is 0.599 — much larger than
Earth's 0.242, and a different shape of problem. The convergent ladder is
correspondingly different: it starts at 1/1, meaning "every Mars year is a leap year",
which is nearly right and not right enough.

At the default drift bound the mechanism selects *45/76*: forty-five intercalary sols
every seventy-six Mars years, drifting one sol in 16,566 Mars years.

#callout(label: "A number the article RFC states differently")[
  This book's own specification says Mars selects *16/27* at a bound of one sol per
  2,342 years. Both figures are in the walk above, and they are consistent: 16/27 is
  convergent 5 and it does slip one sol in 2,342 Mars years.

  But 2,342 is not the default bound. At the default — one local day per ten thousand
  local years, the same bound Earth is given — 16/27 fails and the mechanism goes one
  rung further to 45/76.

  Rule S again. The specification quoted a real convergent at a bound it did not
  state, and the crate quotes what the default actually produces.
]

#claim("interpretation")[
  Notice what the drift bound is *stated in*: one local day per ten thousand local
  years. Not seconds. Not Earth days.

  That is delta D-A13, and chapter 9 recorded it as a cost. Here is the benefit. The
  identical bound, applied to two planets, means the same *thing* on both without
  meaning the same *duration* on either — and it selects different rules because the
  planets are different, not because the bound was tuned per planet.

  A bound in seconds would have been an Earth constant sitting inside the mechanism
  Rule K exists to keep Earth-free. The awkwardness of local units is the price of not
  having one.
]

#section("Titan")

Titan is where a mechanism that merely looked general would fail, and it is in the
book for that reason.

Titan is tidally locked to Saturn. Its rotation period equals its orbital period about
Saturn, so its *solar day* — the interval between successive noons — is its month, in
any ordinary sense of the word. And its *year* is not its orbit about Saturn at all;
it is Saturn's orbit about the Sun, roughly 29.5 Earth years.

So the four components come apart in a way they do not for a planet: the body's
rotation and its primary-orbit are the same number, and the period that gives the
year belongs to a different body entirely.

#claim("interpretation")[
  The mechanism handles this with no special case, and the absence of the special case
  is the result.

  Nothing in the derivation knows that Titan is unusual. It takes a rotation period, a
  solar day, and an orbital period as exact rationals of ticks, and it expands a
  continued fraction. That two of those numbers coincide, and that the third comes
  from Saturn's motion rather than Titan's, are facts about the *data* — and the data
  is where facts about bodies are supposed to live.

  A mechanism that needed to be told about tidal locking would be a mechanism with a
  list of the cases its author had thought of.
]

Titan also demonstrates the mechanism's limit, immediately and unambiguously:

#terminal(caption: "ucal cal show titan-d")[
```
UCAL-E0062: calendar has no anchor; local fields cannot be produced
```
]

That is chapter 16's subject, and it is not a defect in the derivation. The units,
the intercalation and the cycles are all computed. What is missing is phase, and phase
is not derivable from any of them.

#section("One instant, three calendars")

The mechanism's output, seen from outside:

#terminal(caption: "ucal show — one instant, three calendars")[
```
ticks  8070205189128471254993117657693008777530466139316558837890625

earth-d:      earth-d/1: 0027-213.7987 c328    derived (Rule K)
mars-d:       mars-d/1:  0082-086.1665         derived (Rule K)
earth-civil:  2026-07-31T19:11:12 TT           legacy (§8.6)
                                               UCAL-W0005
```
]

#v(2mm)
#block(breakable: false, width: 100%)[
#align(center, cetz.canvas({
  import cetz.draw: *
  let rows = (
    ("earth-d",     "0027-213.7987 c328", "derived", false),
    ("mars-d",      "0082-086.1665",      "derived", false),
    ("earth-civil", "2026-07-31T19:11:12", "legacy",  true),
  )
  // the single instant
  line((3.4, 0.75), (3.4, -3.0), stroke: (thickness: 0.8pt, dash: "dashed"))
  content((3.4, 1.05), text(size: 8pt, weight: "bold", "one instant — one tick count"))
  let y = 0
  for (name, label, kind, legacy) in rows {
    content((0, y), anchor: "west", text(size: 8.5pt, weight: "bold", name))
    circle((3.4, y), radius: 0.09, fill: black, stroke: none)
    content((3.65, y), anchor: "west", text(size: 8pt, raw(label)))
    content((3.65, y - 0.28), anchor: "west",
      text(size: 7pt, fill: if legacy { rgb("#8a3a3a") } else { luma(110) },
        if legacy { kind + " — UCAL-W0005" } else { kind }))
    y = y - 0.9
  }
  content((3.4, -3.35), text(size: 7.5pt, fill: luma(100),
    "three renderings, one value — and \"now\" is not a shared object"))
}))
#v(1mm)
#figcap[7][
  Cross-body simultaneity. The same tick count rendered in three calendars, each
  carrying its kind and its anchor revision.
]
]

Three things to notice. Each rendering carries its *kind*, so a derived value is never
mistaken for a declared one. Each derived rendering carries its *anchor revision*, so
values computed under different anchor determinations are never silently compared. And
the legacy rendering carries a warning, every time.

#recap((
  [Earth: 31/128 at the default bound; the Julian rule is convergent 1 and the Gregorian is absent.],
  [Earth's Moon yields 235/19 — the Metonic cycle — as the sixth convergent, from the two periods alone. That is a fact about where good approximations live, not about anticipation.],
  [Mars: 45/76 at the same default bound, not the 16/27 the article RFC quotes — which is convergent 5, correct at a bound the RFC did not state.],
  [The bound is stated in *local* days and years, so the same bound means the same thing on both planets without meaning the same duration.],
  [Titan is tidally locked and takes its year from Saturn; the mechanism handles it with no special case, which is the result.],
  [One instant renders in three calendars, each carrying its kind and its anchor revision.],
))
