#import "../design.typ": *

#chapter(number: 11, title: "Why it cannot be measured")

Chapter 3 gave one reason and promised three more. Here they are, and then a fourth
the specification does not state, which is the best of them.

The question this part answers is narrow and it is worth keeping in view: *why is the
origin of an exact integer count not simply the measured age of the universe?* The
measurement exists. It is very good. Why not use it?

#section("First: exactness cannot come from measurement")

The published age is 13.787 billion years, plus or minus 0.020 billion.

The plus-or-minus is not a formality. Converted into ticks it is a number with
fifty-eight digits — about 0.145% of the entire span from the datum to now, or 141.53
drifts on the ladder of chapter 4. If tick zero were *defined* as the measured age,
every timestamp would be an offset from a foundation that wobbles by that much, and
the exactness of the integer arithmetic would be decoration over a guess.

This is the reason that forces the decision. The others explain why the decision is
not a workaround.

#section("Second: the limit is not an event")

The FLRW $t arrow.r 0$ limit is not an observable event. It is where a mathematical
model's coordinate goes when you extrapolate backwards, and the model is known to
stop describing anything real well before you get there.

You cannot measure the time of an event that the theory containing it declines to
assert exists. What is measured is the *age* — how long the universe has been
expanding under the model — which is a different quantity that happens to be
numerically what you want.

#section("Third: the extrapolation is model-dependent")

The figure 13.787 is not read off an instrument. It is inferred: a set of parameters
is fitted to observations, and the age falls out of the fit. Change the model and the
number changes.

A datum defined by that inference would inherit not only its error bar but its
*dependence on a cosmological model that will be revised*. Timestamps written today
would mean something slightly different after the next data release — which is
failure mode F1, and the reason profiles exist.

#section("Fourth: Kant, and the question's shape")

The specification stops at three. There is a fourth, older than all of them, and it
is the one that makes the other three feel less like obstacles and more like
symptoms.

#claim("tradition")[
  In the *Critique of Pure Reason*, the First Antinomy sets out a thesis — that the
  world has a beginning in time — and an antithesis — that it does not — and argues
  that *both proofs are valid*.

  Kant's resolution is that both conclusions are false, because both share a premise
  that cannot be granted: that the world-series is a completed totality, given as a
  whole, about which a question of finite-or-infinite extent has an answer waiting to
  be found. On his account the series of past states is not an object of possible
  experience at all; it is something reason extends indefinitely, never something it
  is handed.
]

If that is right, then the search for the true origin is not a hard empirical problem
awaiting better instruments. It is a well-formed-looking question with nothing of the
appropriate kind to be true *of*.

#claim("interpretation")[
  This is the strongest available support for stipulating the datum, and it is worth
  being precise about what it does and does not do.

  It does not show that the universe has no beginning. Kant is explicit that the
  antithesis fails too. What it shows is that the *question as posed* — where is the
  real zero, so that we may count from it — is asking for something that could not be
  supplied even in principle.

  Which means stipulating is not a retreat from a better method. There is no better
  method. Chapter 3 said the system refuses the question rather than answering it with
  a negative number; this is the philosophical form of the same refusal, and it
  arrived two centuries before the measurement did.
]

The book returns to Kant once more, in its last chapter, for the part of his argument
that is less comfortable.

#section("The datum is in ordinary company")

All of that can make a stipulated origin sound exotic. It is the normal case.

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    [*epoch*], [*zero*], [*chosen because*],
    [TAI], [1 January 1958], [international agreement; atomic clocks were running],
    [Julian Day], [1 January 4713 BC], [a convenient common multiple of three cycles],
    [Unix], [1 January 1970], [recent, round, and before the machines that used it],
    [UC-1], [the datum], [an exact count needs an exact origin],
  )
]
#v(2mm)

None of the first three is a discovery about the universe. All are decisions that
turned out to be useful, and nobody finds them troubling.

#callout(label: "The exact parallel: the SI second")[
  The second is the sharpest case, because it is doing the same thing in the same
  system.

  The SI second is defined as 9,192,631,770 periods of a caesium-133 hyperfine
  transition. That number is not a fact about caesium that was waiting to be found.
  It was *chosen*, in 1967, to match the ephemeris second then in use — itself defined
  from a particular fraction of a particular tropical year.

  So the second is a stipulation calibrated against an older stipulation, and the
  calibration carries an uncertainty that the definition does not inherit. Nobody says
  the second is uncertain because the 1900 tropical year was imperfectly measured. The
  definition is exact; the *correspondence* to what it was calibrated against is not.

  That is precisely the structure of the datum, and it is why the datum is unremarkable
  once you see where it already lives.
]

#section("What is left over")

Stipulation solves the exactness problem completely and leaves a different problem
entirely intact.

The physical claim is still true or false. Something *is* the case about the
relationship between tick zero and the early universe, and the published uncertainty
in that relationship is real information that a serious system should not discard.

So there are two things to hold at once: an origin that is exact by definition, and a
claim about that origin that is uncertain by measurement. Conflate them and you get
either a false precision or a useless one.

Chapter 12 is how they were kept apart — and the answer is the reason this project
became a book rather than a library with a good README.

#recap((
  [The measured age carries ±0.020 Gyr — 141.53 drifts, 0.145% of the span. An exact count cannot inherit that.],
  [The FLRW $t arrow.r 0$ limit is not an observable event; what is measured is the age under a model, which is a different quantity.],
  [The inference is model-dependent, so a datum defined from it would shift with each revision — failure mode F1.],
  [Kant's First Antinomy adds a fourth: the question presupposes a completed totality that is not a possible object of knowledge, so there is nothing of that kind to find.],
  [Stipulated origins are ordinary — TAI, Julian Day, Unix — and the SI second is the exact parallel: a definition calibrated against something uncertain, without inheriting the uncertainty.],
  [What remains is a real claim about the world that must be kept, and kept separate.],
))
