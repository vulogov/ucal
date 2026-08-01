#import "../design.typ": *

#chapter(number: 2, title: "The tick")

Everything this system computes is an unsigned integer count of ticks. Nothing else is
primitive — not the second, not the beat, not the day. If you understand why the tick was
chosen and, more importantly, what choosing it does *not* claim, you have the foundation
of the whole instrument.

#term("Tick")[
  The Planck time, $t_P = sqrt(planck G \/ c^5) approx 5.391 times 10^(-44)$
  seconds. In this system it is the atomic unit of duration: the smallest quantity
  representable, and the unit in which every other quantity is counted.
]

#section("Why this unit")

The Planck time is composed from three constants: the gravitational constant $G$, the
reduced Planck constant $planck$, and the speed of light $c$. Those three bound
gravitation, quantum action, and the propagation of causality respectively — and there is
exactly one combination of them with the dimension of time.

That is the appeal, and it is worth stating carefully because it is easy to overstate. The
tick is not defined by any body's motion. It does not know about Earth, or about the
Solar System, or about matter being organised into planets at all. It is the interval you
get when you ask the three limiting constants of physics what a duration would be, and
they answer.

For a system whose entire purpose is to have no Earth content in its arithmetic, that is
the right floor.

#section("What the tick is not")

Here is where a great deal of writing about the Planck time goes wrong, and where this
project declines to follow.

*The tick is not a quantum of time.* It is the resolution floor of an instrument, not a
structure discovered in the world. Nothing in this project asserts that time comes in
discrete lumps, that there is a smallest possible interval, or that asking about a shorter
duration is physically meaningless. The system cannot represent a shorter duration; that
is a fact about the system.

The distinction matters more than it looks, because asserting otherwise would silently
take a side in an argument that has been running for twenty-four centuries.

#claim("tradition")[
  Aristotle holds in *Physics* VI that time is a continuum: divisible without limit, and
  the instant is a *limit* of an interval rather than a part of it, in the way that a
  point is a limit of a line and not a very short line. On this view a smallest duration
  is not merely unobserved but incoherent.

  Against that, temporal atomism has a long and respectable history. Epicurus held that
  there are *minima* of time as there are minima of magnitude. The Kalām tradition
  developed a thoroughgoing atomism in which time is composed of indivisible moments and
  accidents do not endure from one to the next.
]

The instrument is discrete. If discreteness were asserted as a claim about the world, this
project would have joined Epicurus and the Kalām against Aristotle — and it would have done
so not on the strength of an argument but as a side effect of choosing an integer type.

That is not a respectable way to take a metaphysical position. So the position is declined,
explicitly, and the declining is written down: the tick is where the instrument's
resolution stops, and the instrument makes no claim about where the world's does.

#callout(label: "A rule you will see again")[
  This pattern — *the implementation forces a choice; name the choice; refuse the claim
  that would come free with it* — recurs throughout the design. It is the same move that
  Part IV makes about the datum, at much greater length and with much higher stakes.
]

#section("The concession: the tick's length")

There is one place where the tick is not free of Earth, and it should be admitted here
rather than discovered later.

The Planck time's *numerical value* is known only as well as $G$ is measured, and $G$ is
the worst-measured of the fundamental constants by a wide margin. Worse, expressing it at
all requires a unit of time to express it *in* — and that unit is the SI second, which is
defined by a caesium transition counted by instruments on Earth.

So the tick's length is fixed by convention against the second. The system declares a
value, records where the value came from, and uses that declared value everywhere. It does
not re-derive it, and it does not pretend the value is more certain than the measurement
behind it.

#claim("interpretation")[
  What this concedes is *metrology* and nothing else. The arithmetic contains no Earth
  content: ticks are counted, not converted. What is Earth-flavoured is the statement of
  how long a tick is in seconds — which is a sentence about translating between two
  systems, not a fact used inside either.

  Whether that is a satisfying answer is a fair question, and Part III returns to it. It is
  at least an honest one, and its honesty is structural: the conversion lives at a declared
  boundary you can point to rather than dissolved into the operations.
]

#section("Counting, not measuring")

One more consequence, which will matter in Part IV.

Because time here is a *count*, the system's exactness is the exactness of integer
arithmetic. Tick 8,070,204,002,895,596,515,944,343,085,635,637,180,530,466,139,316,558,837,890,625
is a specific tick, and adding one to it gives the next one, and that is all perfectly
exact — in the same way that the number of steps you have walked is exact whether or not
you know it.

What is *not* exact, and cannot be made exact by any amount of careful arithmetic, is the
correspondence between tick zero and any physical event. That is a separate question, it
has a separate answer, and keeping those two things apart is the discipline the rest of
this book is about.

#recap((
  [The tick is the Planck time, built from $G$, $planck$, and $c$ — the one combination of the three limiting constants with the dimension of time.],
  [It is the resolution floor of an instrument, *not* a quantum of time. Asserting otherwise would take a side in a live argument as a side effect of choosing an integer type.],
  [Its length in seconds is fixed by convention against SI. This concedes metrology and nothing else, at a boundary you can point to.],
  [Time is counted, not measured. The count is exact; what tick zero corresponds to is a different question entirely.],
))
