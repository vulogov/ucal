#import "../design.typ": *

#chapter(number: 3, title: "The zero of time")

A count needs somewhere to start counting from. This chapter says where, and says the one
thing about it you need in order to read Parts II and III without being misled. The full
argument is Part IV, four chapters of it, and the deferral is deliberate — but a reader who
stops at the end of Part I must not leave with a false impression, so the essential point
is made here and made plainly.

#term("Datum")[
  Tick zero. A *stipulated* reference point, conventionally identified with the
  Friedmann–Lemaître–Robertson–Walker $t arrow.r 0$ limit. It is not a measurement and not
  an observed event.
]

#section("The domain is unsigned")

Absolute time in this system is an unsigned integer. There is no tick $-1$.

This is not a storage optimisation, and it is not a claim that nothing existed earlier. It
is a statement about what the system is willing to *represent*: the domain begins at the
datum, because that is where the count begins, and a count does not go backwards past its
own start.

The specification is explicit that the two differ. `BIG_BANG_CLAIM` is a *signed* window
precisely because the FLRW limit may lie before the datum — and a system asserting that
nothing precedes tick zero would have no use for a type able to express something that
does.

Ask for an instant before tick zero and you do not get a negative number. You get an error
— `UCAL-E0020` — and the operation fails.

#claim("interpretation")[
  There is a difference between answering a question with a negative number and refusing
  the question, and the system takes the second option deliberately.

  A negative tick count would be an *answer*: it would say that the time before the datum
  exists, is measurable, and happens to lie on the other side of a chosen origin — the way
  1 BC lies on the other side of the Gregorian epoch. The refusal says something else: that
  the question, as posed to this instrument, is malformed.

  Augustine gives the classic form of the move in *Confessions* XI, answering what God was
  doing before creating heaven and earth. His answer is not a description of that interval
  but a rejection of the question's shape, because time is among the things created and
  there is no "before" of the sort the question wants. Whether or not you find the theology
  congenial, the logical structure is exactly `UCAL-E0020`: an error, not a negative number.
]

#section("Why it is stipulated and not measured")

Here is the one-line version. Part IV gives it four chapters and three further reasons;
this is the one you can carry.

*Exactness cannot come from measurement.*

The age of the universe is a measured quantity. The current best figure is 13.787 billion
years, plus or minus 0.020 billion. That is an excellent measurement. It is also, like every
measurement, an interval rather than a point.

Convert that uncertainty into ticks and it becomes:

#terminal(caption: "ucal datum — the claim's magnitude")[
```
big_bang_claim:
  half_width_ticks   11706976141141069872000000000000000000000000000000000000000
  half_width_drifts  141.53
```
]

Fifty-eight digits. About 0.145% of the whole span from the datum to now.

Now suppose the datum were defined *as* the measured age. Every timestamp in the system
would inherit that uncertainty, because every timestamp is an offset from the datum. The
exact integer arithmetic — the thing this entire design exists to protect — would be
computing precise-looking answers on top of a foundation that wobbles by a number with
fifty-eight digits in it.

You cannot get an exact origin from an inexact measurement. So the origin is not taken from
the measurement. It is *declared*, and the measurement is recorded separately.

#section("What is actually claimed")

Precision about this matters, so here is the claim in full, with nothing left implicit.

The system asserts that tick zero is tick zero. That is a definition, and definitions are
not the sort of thing that can be wrong.

It asserts, *separately and as metadata*, that tick zero is conventionally identified with
the FLRW $t arrow.r 0$ limit, and that the published uncertainty in that identification is
±0.020 Gyr, citing Planck 2018. This is a claim about the world and it may be revised.

And it asserts nothing whatever about whether time began, whether there was a first moment,
or whether the FLRW limit corresponds to a physical event at all.

#callout(label: "Deferral is not evasion")[
  You have now been told that the datum is a decision rather than a discovery, and why the
  short reason forces it. What Part IV adds is three further reasons the specification
  gives, one more that it does not, and — the part that turned this project into a book —
  what was *done* about it: how a claim can be declared, cited, given an exact magnitude,
  and simultaneously made impossible to compute with.

  If you are on the engineer's path and skipping Part IV, the sentence to carry forward is
  this one: the uncertainty in the age of the universe does not make the timestamps
  uncertain, because it is not an operand.
]

#section("The datum is in ordinary company")

One last thing, because "stipulated origin" can sound exotic and is not.

TAI's zero is 1 January 1958, chosen by agreement. The Julian Day epoch is 1 January 4713
BC, chosen because it made a convenient common multiple of three cycles. The Unix epoch is
1 January 1970, chosen because it was recent and round. None of these is a discovery about
the universe; all of them are decisions that turned out to be useful.

#claim("interpretation")[
  What is unusual here is not that the origin is stipulated. It is that the stipulation is
  *declared as such in the artifact*, with the physical claim held separately and marked
  non-computable — rather than being a fact about the system's history that you would have
  to go and read about.

  Most epochs are stipulated and silent about it. This one is stipulated and says so, in a
  place the compiler can reach.
]

#recap((
  [Tick zero is the datum: stipulated, conventionally identified with the FLRW $t arrow.r 0$ limit, and explicitly not a measurement.],
  [The domain is unsigned. A request for a time before the datum is an error, not a negative number — a refusal of the question rather than an answer to it.],
  [Exactness cannot come from measurement: the ±0.020 Gyr uncertainty is a 58-digit number of ticks, and an exact count cannot inherit it.],
  [The physical claim is recorded separately as metadata, cited, and — as Part IV shows — made impossible to use in arithmetic.],
))
