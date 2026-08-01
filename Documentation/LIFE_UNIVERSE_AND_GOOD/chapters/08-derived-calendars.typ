#import "../design.typ": *

#chapter(number: 8, title: "Derived calendars")

The ladder of chapter 4 has no days and no years in it, deliberately. But people live
on planets, and a calendar that cannot say what day it is locally is an
instrument of limited interest.

So there is a second layer: local calendars, derived from a body's own motion. This
chapter is how that derivation works. Part V is how far it generalises and where it
fails.

#section("Four components")

Every calendar in the system is the same shape:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    [*component*], [*what it is*], [*source*],
    [Body], [rotation, solar day, orbital period — as exact rationals of ticks], [cited measurement],
    [Anchor], [where the count starts in phase], [*declared, cited, empirical*],
    [LeapRule], [intercalation: how the fractional day is absorbed], [*derived*],
    [Cycles], [grouping periods, if the calendar names a satellite], [*derived*],
  )
]
#v(2mm)

Three of the four are computed. One is not, and the one that is not is the interesting
one — it gets flagged here and gets a full treatment in chapter 16.

#section("Units, as rationals of ticks")

A body's periods enter as exact rational numbers of ticks, each carrying its citation,
its epoch, its secular rate, and the window over which it is valid.

That last part matters more than it sounds. Earth's rotation is slowing by roughly 1.8
milliseconds per century; its tropical year is shortening by about half a second per
century. A parameter treated as a constant is wrong the moment you leave the epoch it
was measured at — and this system is aimed at deep time, where "leaving the epoch" is
the normal case.

So evaluating a parameter outside its stated window produces a warning rather than a
confident extrapolation. The system will tell you the number is out of its depth.

#section("Intercalation is derived, never declared")

Here is the heart of the mechanism.

A body's year is not a whole number of its days. Earth takes about 365.2422 rotations
per orbit. Some scheme has to absorb the 0.2422, and every civilisation that has
built a calendar has invented one.

This system does not invent one. It *computes* the schemes, by expanding the
fractional part as a continued fraction and reading off the convergents — the
sequence of best rational approximations, in the precise sense that no fraction with a
smaller denominator comes closer.

#terminal(caption: "ucal cal show earth-d — the derivation")[
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
```
]

Read the first line of that walk. Convergent 1 is *1/4* — one leap day every four
years. That is the Julian calendar, derived from Earth's rotation and orbit with no
knowledge of Rome.

The system then walks the ladder until a convergent meets the requested drift bound,
and reports the entire walk rather than only the winner — so you can see what was
rejected and why.

#claim("interpretation")[
  The reporting-the-whole-walk decision looks like a nicety and is not. A mechanism
  that returns only its answer cannot be argued with. One that shows its working can
  be checked, and — as chapter 10 will describe at length — can turn out to
  contradict the person who built it.

  The single most important output this project ever produced came from reading a walk
  and noticing what was *not* in it.
]

#section("Cycles, and the absence of them")

Months are grouping periods, and they come from a satellite.

If a calendar names a grouping satellite, the system derives the cycle structure the
same way — continued-fraction expansion of the ratio between the orbital period and
the satellite's synodic period. Earth names the Moon, and the derivation produces
235/19, the Metonic cycle, known since antiquity and recovered here from Earth's
periods alone.

If a calendar names no grouping satellite, it has no months. Not a default month, not
a synthesised one, not a fallback to Earth's. None.

#terminal(caption: "ucal cal list — three derived calendars")[
```
earth-d:
  leap_rule        31/128 (convergent 4)
  cycles           from moon

mars-d:
  leap_rule        45/76 (convergent 6)
  cycles           none — the calendar names no grouping satellite

titan-d:
  status  no anchor: complete in units, intercalation and cycles,
          incomplete in phase. Asking for local fields is UCAL-E0062.
```
]

#callout(label: "Mars has no month, and that is the correct output")[
  Mars has two satellites. Phobos orbits in about 0.45 of a Martian day; Deimos in
  about 5.4. Neither is anything a person would call a month.

  The system could have synthesised one — divided the Martian year into twelve, say,
  or borrowed a period from somewhere. It returns nothing instead, because a
  month-shaped structure imposed on a body that has no month is Earth structure
  leaking through a mechanism built specifically to keep it out.

  The absence is not a gap in the implementation. It is the implementation working.
]

#section("The anchor: the one thing that cannot be derived")

Everything above came out of arithmetic on cited parameters. The anchor does not, and
cannot.

Intercalation tells you how long a year is. It does not tell you *when the year
starts* — and no amount of tick counting will, because phase is not a consequence of
period. You can know exactly how long Earth takes to rotate and still not know
whether it is currently noon.

So the anchor is declared: one cited, interval-valued constant per body, with a
revision number and an uncertainty window.

#terminal(caption: "ucal cal show earth-d — the anchor")[
```
anchor:
  phase         mean solar midnight
  revision      1
  method        mean solar midnight at the prime meridian on 2000-01-01,
                i.e. 00:00:00 UT1, converted through TT = UT1 + Delta-T
                with Delta-T(2000.0) = 63.8285 s
  uncertainty   dominated by the resolution of the published Delta-T series
  window_ticks  37097168799722000000000000000000000000000
  citation      IERS Conventions (2010) and the IERS Earth Orientation
                Centre's published Delta-T series
```
]

A calendar with no anchor is not an error to construct. It is an error to *ask local
fields of*: `UCAL-E0062`. Titan is in exactly that state, and chapter 16 explains why
it will stay there.

#claim("interpretation")[
  The anchor is where this mechanism admits it is not self-sufficient, and the admission
  is structural rather than apologetic. It is one declared constant, no more privileged
  than the rotation period sitting beside it and no less necessary.

  What would have been dishonest is a default — an anchor that quietly fell back to
  Earth's, or to zero, so that every body appeared to have a working calendar. That
  would produce confident local dates for a body whose phase nobody knows.
]

#section("Derived and legacy, held apart")

The Gregorian calendar is in this system, and it is not a derived calendar.

#terminal(caption: "ucal cal list — the legacy entries")[
```
earth-civil:
  kind       legacy — declared tables (§8.6)
  arbitrary  4
  leap_rule  97/400 (NOT a convergent — declared, not derived)

earth-julian:
  kind       legacy — declared tables (§8.6)
  arbitrary  4
```
]

`earth-civil` has four items of *arbitrary content* — irregular month lengths, the
seven-day week (which corresponds to no astronomical period at all), the leap rule,
and the epoch. None of them follows from Earth's motion. They are declared tables,
and the type system keeps them in a separate trait: a function that wants a derived
calendar will not accept a legacy one, and there is no blanket conversion.

Note the parenthesis on the leap rule. `97/400` is marked *NOT a convergent*. That
label is the subject of chapter 10, and it is where this book's most uncomfortable
finding lives.

#claim("interpretation")[
  It is worth being explicit that "legacy" here is a technical classification and not a
  judgement. A declared table is not a worse calendar than a derived one; the Gregorian
  reform solved a real problem and solved it well enough to still be in use.

  What the classification records is *where the authority comes from* — arithmetic on
  cited parameters, or a decision someone made. Part VI applies that same distinction
  to scriptural chronologies, and it has to apply it evenhandedly or not at all. A
  system that calls the Gregorian derived and Seder Olam declared would be doing
  something other than classifying.
]

#recap((
  [Every calendar is (Body, Anchor, LeapRule, Cycles). Three are computed; the anchor is not.],
  [Body parameters are exact rationals of ticks carrying epoch, secular rate, and validity window — and evaluating outside the window warns rather than extrapolating.],
  [Intercalation is derived by continued-fraction expansion, and the whole walk is reported, not just the winner. Convergent 1 for Earth is 1/4 — the Julian rule, derived.],
  [Cycles come from a named grouping satellite or do not exist. Mars has no month, and the absence is the mechanism working.],
  [The anchor cannot be derived, because phase is not a consequence of period. It is one declared, cited, interval-valued constant, and its absence is `UCAL-E0062` rather than a default.],
  [Legacy calendars are held apart by the type system. `97/400` is marked *not a convergent* — the subject of chapter 10.],
))
