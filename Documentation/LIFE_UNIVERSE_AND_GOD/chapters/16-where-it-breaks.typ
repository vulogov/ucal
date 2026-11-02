#import "../design.typ": *

#chapter(number: 16, title: "Where it breaks")

Chapter 15 showed the mechanism working on fifteen bodies. This chapter is the same
length, and that is a rule rather than a coincidence: a book whose capability section
outruns its limits section is selling something.

Six limits. Each is stated as a limit, not as a caveat — the difference being that a
caveat is a qualification you are meant to read past.

#section("1. The anchor is empirical and cannot be derived")

This is the largest one, and everything else on the list is smaller.

The mechanism derives units, intercalation, and cycles from a body's periods. It
cannot derive *phase*. Knowing exactly how long Earth takes to rotate tells you
nothing about whether it is currently noon, and no amount of additional tick counting
will supply the missing fact.

Getting phase requires knowing where a body actually was at some actual moment, which
means ephemerides — observation, tabulated. That is a different kind of input and a
different discipline, and it is out of scope for this system.

So every calendar carries one declared, cited, interval-valued constant:

#terminal(caption: "ucal cal show earth-d — the anchor")[
```
anchor:
  phase         mean solar midnight
  revision      1
  method        mean solar midnight at the prime meridian on
                2000-01-01, i.e. 00:00:00 UT1, converted through
                TT = UT1 + Delta-T with Delta-T(2000.0) = 63.8285 s
  uncertainty   dominated by the resolution of the published Delta-T
                series, which is quoted to 0.0001 s near 2000.0; the
                window is widened to 1 ms to cover the series' own
                stated scatter
  window_ticks  37097168799722000000000000000000000000000
  citation      IERS Conventions (2010) and the IERS Earth
                Orientation Centre's published Delta-T series;
                iers.org no longer answers — the original document
                is available in the Internet Archive
```
]

#claim("interpretation")[
  It is tempting to present the anchor as a minor input, and the honest framing is
  less comfortable: *the mechanism is not self-sufficient, and one component of every
  calendar it produces is a measurement it did not make.*

  What can be said in its defence is structural. The anchor is no more privileged than
  the rotation period beside it — both are cited measurements with uncertainty windows
  — and it is no less necessary. The design does not pretend the anchor is derived,
  does not hide it among the computed fields, and does not default it. Its absence is
  an error rather than a fallback, which is what the second half of chapter 8 was
  about.
]

#section("2. A body with no qualifying satellite has no month")

Months come from a satellite. A body without a suitable one has none, and the
mechanism must return nothing rather than synthesise something.

Mars is the worked case, and it is worth going through carefully because the article
RFC for this book gets it wrong and the error is instructive.

Mars has two satellites. Under §9.6's formula as originally written — synodic period
measured against the primary's solar day — Phobos comes out at *0.450 sols* and Deimos
at *5.363 sols*.

The specification admitted a satellite as month-giving if its synodic period fell in
the bracket 5 to 100 solar days.

#callout(label: "Deimos is inside the bracket")[
  This book's specification says both moons "fail the bounds". Phobos does, at 0.45.
  Deimos does not: 5.363 is inside [5, 100], comfortably.

  So the RFC's own algorithm *admits* Deimos and hands Mars a month of 124.67 cycles
  per Mars year — while the same document states elsewhere that Mars has no month. The
  specification contradicted itself, and the implementation is what surfaced it.
]

There is a second layer to this, and it is the one that matters.

Delta D-A12 established that §9.6's formula computes the wrong quantity: it measures
against the primary's *solar day* where Appendix I.2's own worked example measures
against the *year*. Under the corrected, year-relative formula Deimos's synodic period
is *1.2315 sols*, which is outside the bracket.

So the bracket's verdict on Deimos depends entirely on which formula you use — 5.363
and admitted, or 1.2315 and rejected.

#claim("interpretation")[
  Two ways to fix a rule that admits a satellite you did not want. Tune the bracket
  until it excludes Deimos. Or notice that the bracket was calibrated on Earth's Moon —
  synodic period 29.5 days, comfortably interior — and is therefore an Earth constant
  sitting inside the one mechanism built to keep Earth out.

  The first is what a system built to make its conclusions provable would do, and it is
  precisely Maimonides' charge from chapter 10. The second is delta D-A5: the bracket
  was removed entirely, and a calendar must now *name* its grouping satellite with a
  citation for why.

  `mars-d` names none. So Mars has years and sols and no months — not because 5.363
  fell on the convenient side of a constant chosen for the Moon, but because nobody has
  declared a Martian month and the mechanism will not invent one.

  Whether a satellite is "month-like" is not derivable, because *month-like* is an
  Earth predicate.
]

#section("3. A rogue planet has no year")

A planet not bound to a star has no orbital period. There is no fraction to absorb, no
continued fraction to expand, and no intercalation to derive.

The mechanism has nothing to compute, and it returns nothing. A day-count is still
available — rotation is intrinsic — but a calendar in the ordinary sense, with a
repeating annual structure, does not exist for such a body and cannot be manufactured.

This is a small limit in practice and a clarifying one in principle: it shows that
three of the four components depend on a *relationship* between the body and something
else, and only rotation is the body's own.

#section("4. A tidally locked body's day equals its orbit")

Chapter 15 treated this as a success, and it is one — Titan is handled with no special
case. It is also a limit, and both are true.

When a body is tidally locked to its primary, two of the four calendar components
collapse into one number. The solar day and the primary-orbit are the same period, so
a structure that assumes they are independent has one fewer degree of freedom than it
expects.

Nothing breaks. But the calendar that results is thinner than a planet's: there is no
relationship between rotation and primary-orbit to derive anything from, because they
are the same relationship.

#section("5. Relativistic environments are out of scope")

There is no relativistic model here at all.

No time dilation. No worldline. No transformation between frames. The profile declares
a single comoving frame — FLRW cosmological time, in the rest frame of the microwave
background — and every quantity is counted within it. That is the frame in which the
age of the universe is quoted, so for the cosmological use the choice costs nothing a
cosmologist has not already paid.

The one place the frame touches the arithmetic's edge is the bridge. Terrestrial Time
is a clock on Earth's geoid, and the geoid is not comoving with the microwave
background: the solar system moves against it at about 370 kilometres a second, and
sits in the Galaxy's potential. So the datum output carries a second piece of
metadata beside `BIG_BANG_CLAIM`: a *frame-bridge claim*, a declared bound of five
parts in a million on the rate difference between the profile's frame and its bridge
scale. Over the whole span that is about 69,000 years, 290 times smaller than the
claim's half-width, and like the claim it is held in a type no arithmetic may consume.
It is a rate, so it cancels exactly in any difference of two instants carried through
the same bridge — which is every interval this program computes. It bears only on
reading an absolute tick count as elapsed cosmological time.

#claim("interpretation")[
  It is tempting to call the missing physics a missing feature, and the honest
  description is harder: it is the price of the central data structure. One unsigned
  integer per instant asserts that there is one time, and that assertion is the thing
  general relativity denies. A clock on Earth's surface, a clock in orbit, and a clock
  in the comoving frame do not agree on how many ticks have elapsed, and no single
  column of integers can hold all three.

  For deep time — comparing epochs, dating recombination, measuring διάστημα across
  the span — the declared frame is the right one, and the disagreement between clocks
  is small against the intervals measured. For a body deep in a gravity well, or for
  two clocks that must agree to a precision where dilation matters, this system is
  the wrong instrument. It would give an answer, and the answer would be wrong in a
  way it could not detect, which is the worst kind.

  What an extension would look like is known, and saying so keeps the limit from
  being mistaken for a refusal to think about it. The relativistic content belongs at
  the bridge, not in the arithmetic. A second frame is a second *profile*, declaring
  its own frame under Rule F and kept from mixing with UC-1 by Rule P; the conversion
  between them is a declared, certified ratio at a boundary you can point at — the
  same localisation chapter 7 performed on the SI second. Inside each profile the
  tick count remains a coordinate in its declared frame, which is what Rule F already
  says it is. The frame-bridge claim above is that design's first line, written
  before the second profile exists: the gap between frames is on the record, as a
  bound, in a place the compiler can reach. `ucal dilate`, below, is its second: the
  ratio between two clocks, computed and certified, before the profile that would
  consume it exists.

  Rule F is what makes this a limit rather than a hazard: the frame is *declared*, so
  the assumption is visible rather than implicit. That does not make the system usable
  outside the frame. It makes the boundary findable.
]

The pinned tree carries one command that touches this limit, and saying exactly what
it does keeps the limit honest in both directions. `ucal dilate` computes gravitational
time dilation at a radius as a certified interval — `√(1 − r_s/r)` for a static clock,
`√(1 − 3r_s/2r)` for one in circular orbit — bracketed by integer square roots in exact
rationals, so the interval is proved to contain the value rather than converged to it.
At the Sun's surface:

#terminal(caption: "ucal dilate --rs-over-r 0.0000042467 — the Sun's surface")[
```
rs_over_r  42467/10000000000
observer   static at r — gravitational dilation only, √(1 − r_s/r)
digits     40
proper_per_coordinate:
  lo  0.999997876647745687
  hi  0.999997876647745688
redshift_z:
  lo  0.000002123356762946
  hi  0.000002123356762947
certification:
  rounded, trunc, 18 digits  lo
  rounded, ceil, 18 digits   hi
```
]

It reports a ratio between two clocks and does not claim that either of them is the
one `ucal` keeps; its own closing note says so, and gives the reason this chapter gave
— one unsigned integer per instant asserts there is one time. It refuses `r_s/r ≥ 1`,
where the Schwarzschild coordinate stops being a clock at all, rather than returning a
larger number.

#claim("interpretation")[
  So `dilate` is not the missing relativistic model, and it is not nothing. It is the
  conversion this chapter says would have to live at the bridge — computed, certified,
  and kept apart from the tick count — built before the profile that would consume it.
  The limit stands as stated: one column of integers, one declared frame, and the ratio
  to any other clock computed beside the count rather than folded into it.
]

#section("6. Body parameters are wrong outside their validity windows")

The final limit is the one most likely to bite in practice, and it bites hardest at
exactly the scales this project exists for.

Every body parameter carries an epoch, a secular rate, and a window of validity. They
are not constants. Earth's rotation is lengthening by roughly 1.8 milliseconds per
century; its tropical year is shortening by about half a second per century.

Evaluate a parameter outside its stated window and the system warns rather than
extrapolating confidently.

#claim("interpretation")[
  Consider what that means for a system whose domain reaches $2.29 times 10^103$ years.

  The parameters are good near J2000 and degrade as you move away. A calendar derived
  for a body a billion years hence is derived from *today's* rotation rate, and Earth's
  rotation a billion years from now will not be today's — the Moon is receding and the
  tidal braking that drives the change will itself have changed.

  So the mechanism's reach and its accuracy point in opposite directions. It can
  address any tick in a $10^103$-year domain, and it can derive a trustworthy calendar
  only near the epochs where somebody measured something.

  The warning is the honest response and it is not a solution. A drift bound guaranteed
  over 400,000 years is guaranteed *under parameters that will not hold for 400,000
  years*, and the system says so rather than quietly implying otherwise.
]

#section("What the six have in common")

Read together, the limits fall into two kinds, and the distinction is worth having.

Limits 3 and 4 are *structural*: the mechanism gives less because the body has less to
give. A rogue planet genuinely has no year. Those are not deficiencies; correct output
for an unusual body is unusual output.

Limits 1, 2, 5 and 6 are *epistemic*: the mechanism needs something it cannot compute.
Phase requires observation. A month requires a declaration nobody has made. A
relativistic answer requires a model this system does not have. A parameter far from
its epoch requires a measurement nobody took.

#claim("interpretation")[
  In every one of the four epistemic cases the system's response is the same: return an
  error or a warning, and never a default.

  `UCAL-E0062` for a missing anchor. An empty cycle list for an undeclared satellite.
  `UCAL-W0003` for a parameter out of its window. A declared frame that says what it
  does not model.

  That consistency is the actual claim of Part V — not that the approach works
  everywhere, because it does not, but that where it stops working it *says so* rather
  than producing a confident number nobody can audit.

  A calendar that silently defaulted Titan's anchor to Earth's would work. Every
  function would return a value, no test would fail, and every Titanian date it
  produced would be wrong by an unknown amount, with nothing anywhere to indicate it.
]

#recap((
  [The anchor is empirical and cannot be derived: phase is not a consequence of period, and getting it needs ephemerides.],
  [A body with no declared grouping satellite has no month. Mars is the case — and the article RFC's claim that both moons fail the bracket is wrong, which is how the bracket's Earth-calibration came to light.],
  [A rogue planet has no year; three of the four components depend on a relationship, and only rotation is the body's own.],
  [A tidally locked body collapses two components into one — handled without a special case, and thinner for it.],
  [Relativistic environments are out of scope entirely. The declared frame makes that boundary findable, not passable.],
  [Parameters are wrong outside their validity windows, and the reach of the domain and the accuracy of the derivation point in opposite directions.],
  [Four of the six limits are epistemic, and in all four the system errors or warns rather than defaulting. That is Part V's actual claim.],
))
