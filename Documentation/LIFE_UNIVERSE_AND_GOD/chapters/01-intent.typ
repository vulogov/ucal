#import "../design.typ": *

#chapter(number: 1, title: "Intent")

This began as an irritation, and it is worth being honest that it began as an
engineer's irritation rather than a philosopher's question.

Read almost any account of the early universe and you will find a sentence like this
one: *recombination occurred about 380,000 years after the Big Bang.* It is a good
sentence. It is doing real work. And if you stop and ask what its units are, something
uncomfortable happens.

A year is the time Earth takes to go around the Sun. Not approximately — definitionally.
The Julian year used in astronomy is exactly 365.25 days of exactly 86,400 seconds, and
those numbers are what they are because of how fast one particular rock spins and how
long it takes to circle one particular star. So *380,000 years after the Big Bang* means
380,000 × 365.25 × 86,400 seconds, which means it is a quantity expressed in units
defined by the rotation and orbit of a planet that would not exist for another nine
billion years.

The number is correct. It is not wrong in any way that matters to a cosmologist. But it
carries a passenger, and the passenger is Earth.

#section("The irritation, stated precisely")

The problem is not that the units are arbitrary. All units are arbitrary. The metre is
a bar in a vault, then a wavelength, then a fraction of a light-second; none of that
makes it a bad metre.

The problem is *provenance leaking into arithmetic*. When you compute with Earth years,
Earth's orbital period is inside every intermediate value. Usually that costs nothing.
Occasionally it costs you a rounding you did not intend, because 365.25 days is not a
whole number of anything and the conversions do not close. And structurally — which is
the part that would not let go — it means the description of an event 13.8 billion years
old is phrased in terms of a body that had no bearing on it.

#callout(label: "What this is not an argument for")[
  None of this is a criticism of how cosmology is done. Astronomers use Julian years
  because they are stable, agreed, and unambiguous, which is exactly what a unit should
  be. The complaint here is narrow and it is aesthetic before it is technical: a
  quantity ought to be expressible in units that do not smuggle in a planet.
]

#section("What would have to be true instead")

Suppose you wanted a time system with no Earth content in its arithmetic. What would it
need?

It would need a *unit* that is not defined by any body's motion — something built out of
constants of nature rather than out of a rotation.

It would need an *origin* that is not a calendar event, since every calendar epoch is
someone's civil history.

It would need a *notation* in which writing fewer digits means saying less precisely,
rather than meaning zero — because at these magnitudes you will constantly be quoting
numbers you do not know to full precision, and a system that silently pads them with
zeros is lying on your behalf.

And it would need to be honest about the one thing it cannot do: it cannot tell you what
time it is on Earth without, at some point, being told about Earth. That contact has to
happen somewhere. The question is whether it happens *in the arithmetic* or at a
declared boundary you can point to.

That last question turned out to be the whole design.

#section("What was built")

`ucal` is the answer to those four requirements, and Part II is the technical account of
it. In one paragraph: absolute time is an unsigned integer count of Planck-time units
since a stipulated datum; those ticks are grouped in a ladder of powers of five, so a
timestamp is the tick count written in base 5 and truncating it *is* rounding it; Earth
enters through exactly one declared constant and leaves at the formatting boundary; and
no floating-point value appears anywhere in the workspace.

It works. It is also, as the preface said and will keep saying, of questionable use.

#section("Where the trouble started")

The requirement that has no clean answer is the second one: the origin.

If time is a count since some zero, the zero has to be somewhere. And the obvious place
to put it — the beginning — is not available, for reasons that took four chapters of
this book to state properly and that Part IV is entirely about.

The short version, so you can carry it through Parts II and III: *the age of the universe
is a measurement, and measurements have error bars, and an error bar cannot be the origin
of an exact integer count.* The published figure is 13.787 billion years give or take
0.020 billion. That uncertainty, converted to ticks, is a number with fifty-eight digits.
If the datum inherited it, every timestamp in the system would inherit it too, and the
exactness the whole design exists to protect would be theatre.

So the datum is *stipulated*. Declared, not discovered. Tick zero is a reference point
that the specification says, in as many words, is not a measurement and not an observed
event.

And then the interesting problem: how do you say that in a way that a future maintainer —
or the author, three years later, in a hurry — cannot quietly forget?

#claim("interpretation")[
  The answer this project arrived at is the reason it turned into a book. The physical
  claim about the origin is declared, cited, and given an exact magnitude. And then it is
  made *impossible to compute with*: a type with no arithmetic operations at all, guarded
  by a test that fails to compile if the type ever reaches an operand position.

  You cannot forget a discipline that the compiler enforces. That is a small technical
  fact with a large philosophical shadow, and the shadow is what Parts IV, VII, and VIII
  are about.
]

#section("What this book will and will not claim")

Before Part II starts and the prose becomes technical for a hundred pages, here is the
calibration.

It *will* claim that a distinction enforced in a type system is a philosophical
contribution and not merely an implementation detail. It will claim that the approach
generalises to any celestial body, and it will spend a chapter on exactly where it fails.
It will claim that the instrument does philosophical work no prose argument can do, and
it will demonstrate that rather than assert it.

It will *not* claim that time began, that any tradition is correct, that the base is
meaningful, or that anyone should adopt this calendar.

#recap((
  [Cosmic durations are conventionally expressed in units defined by one planet's motion — correct, useful, and carrying a passenger.],
  [A time system with no Earth content in its arithmetic needs a physical unit, a non-calendrical origin, a notation where truncation means imprecision, and one declared point of contact with Earth.],
  [The origin cannot be measured, because a measurement has error bars and an exact integer count cannot inherit one.],
  [So it is stipulated — and the project's contribution is making that stipulation mechanically impossible to forget.],
))
