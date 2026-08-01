#import "../design.typ": *

#chapter(number: 18, title: "Greek")

#callout(label: "How to read Part VI")[
  Nine chapters, one shape. Each says what a direction holds, which rule of the
  artifact it illuminates, where the two converge, *where they conflict*, and what the
  conflict changes about how to read the instrument.

  The conflicts are fixed in advance and none is optional. A tradition that only
  agrees has not been read — it has been quarried. Where a conflict cuts against this
  project, the chapter says so and does not resolve it in the artifact's favour.

  No tradition here is argued true. The code is evidence for none of them.
]

#section("What the direction holds")

Three positions, from three centuries, that between them set the terms this book has
been using without acknowledging where they came from.

#claim("tradition")[
  *Aristotle*, *Physics* IV.11: time is "the number of motion with respect to before
  and after." Not motion itself, and not a container motion happens in — the
  *countable aspect* of change.

  In IV.14 he presses further and asks whether time could exist if there were no soul
  to count. His answer is careful: there would be the substrate of time, the changing
  thing, but not time as number, since number requires a numberer.

  In *Physics* VI he establishes that time is a continuum. Between any two instants
  lies another; the instant is a *limit* of an interval, as a point is a limit of a
  line, and not a very short piece of one.

  *Plotinus*, *Enneads* VI.6, distinguishes two senses of number: οὐσιώδης ἀριθμός,
  substantial number, which is constitutive of what a thing is, and μοναδικὸς ἀριθμός,
  monadic number, the counting-number we use to tally. The first is prior to being; the
  second is our instrument.

  Behind both stands Plato's doctrine of ideal numbers as ἀσύμβλητοι — non-addible.
  Two of them do not make four, because they are not units of a common kind.
]

#section("Which rule it illuminates")

Rule Q, on the datum and the counter. Rule G, on the tier grid and what a number is.
N1, the refusal to call the tick a quantum of time. And `derive_leap_rule`, which turns
out to be older than it looks.

#section("The convergences")

#subsection("Euclid X.2 is the intercalation algorithm")

This is the strongest of them, and it is not a resemblance.

#claim("tradition")[
  *Elements* X.2 gives the procedure Greek mathematics calls ἀνθυφαίρεσις — reciprocal
  subtraction. Given two magnitudes, subtract the smaller from the larger repeatedly;
  take the remainder and repeat against the smaller; continue. If the process never
  terminates, the magnitudes are incommensurable.

  Euclid's purpose is a criterion for incommensurability. The procedure is the
  Euclidean algorithm run on magnitudes rather than numbers.
]

Continued-fraction expansion *is* anthyphairesis. The quotients at each step are the
partial quotients; the convergents are what you get by truncating. Chapter 8's
`derive_leap_rule`, which takes a year and a day and returns 1/4, 7/29, 8/33, 31/128,
is running Euclid X.2 on two orbital periods.

#claim("interpretation")[
  So Appendix I of this project's specification is a Greek text with different
  vocabulary. That is not a coincidence and it is not anticipation. There is one good
  algorithm for this problem and it was found early, because the problem is old and
  the algorithm is short.

  What the convergence is evidence *for* is narrower and more useful: that the
  mechanism is not an invention of the author's. It is the standard procedure, and its
  results are therefore not tuneable in the way chapter 10's charge worried about.
]

#subsection("Aristotle's two numbers, and the ladder")

*Physics* IV.11 distinguishes the number *counted* from the number *by which we count*
— the twenty sheep from the twenty.

That distinction maps onto the artifact with unusual precision. The tick count is the
number counted: how many ticks have elapsed. The tier ladder is the number by which we
count: the system of units in which the count is expressed.

#subsection("Archimedes")

#claim("resonance")[
  The *Sand Reckoner* is addressed to King Gelon and its purpose is to refute the
  claim that the number of sand grains filling the cosmos is infinite — or if not
  infinite, at least unnameable. Archimedes' response is to *build a notation*: a
  positional hierarchy of orders and periods, expressly constructed so that a
  cosmic-scale magnitude becomes expressible.

  He arrives at something around $10^63$.

  The present tick count is about $8 times 10^60$.

  Three orders of magnitude apart, from a problem about sand and a problem about time,
  twenty-two centuries apart. This is a coincidence. It is recorded because the
  *method* is the same — invent a positional hierarchy so that the unnameable becomes
  nameable — and because the numerical closeness is striking and means nothing.

  It is not a premise for anything in this book.
]

#subsection("Two boundary markers")

Epicurus held that time, like magnitude, has minima — indivisible least parts. So
temporal atomism is native to Greek thought, and N1's refusal to call the tick a
quantum declines a Greek position rather than only a later one.

The Stoics held ἐκπύρωσις: the cosmos periodically consumed by fire and reconstituted,
identically, without end. That is a cosmology in which *no datum is possible*. There is
no first cycle to count from, and any origin you stipulate is arbitrary in a way that
UC-1's is not — because UC-1's is at least stipulated *against* a model that has a
limit to point at.

#conflict[
  *Two, and the second is worse.*

  *Time is a continuum; the instrument is discrete.* Aristotle's *Physics* VI is not a
  casual position — it is argued at length, and the argument that the instant is a
  limit rather than a part is one of the more durable things in the corpus. This
  system's finest addressable quantity is a tick, and between tick $n$ and tick $n+1$
  there is nothing.

  N1 declines to claim that reality is discrete, and that helps. But it does not
  dissolve the tension: an instrument whose resolution is a smallest unit is not
  neutral about a continuum. It can only ever address a countable subset of it.

  *Physics* IV.14 is the harder one. If time as number requires a counter, then the
  datum's objectivity depends on there being someone to count from it — and Rule Q,
  which makes the datum stipulated rather than found, *concedes this rather than
  answering it*. The datum is where a counter decided to start. Aristotle would
  recognise that immediately, and he would not regard it as a solution to the problem
  he raised.
]

#section("What it changes")

Here is the thing the Greek material makes visible, and it is uncomfortable.

The tick count is the number counted. The tier ladder is the number by which we count.
And in this system *they are the same integer*: the ladder is not a separate
apparatus applied to the count — a timestamp is the count itself, written in base 5
and grouped in fives. Chapter 4 presented that identity as the design's central
elegance.

#claim("interpretation")[
  Neither Aristotle nor Plotinus would grant it.

  For Aristotle the two numbers are distinct in kind. The twenty sheep and the twenty
  are not the same thing; one is a feature of the flock and the other is what the mind
  supplies. Collapsing them is not simplification but conflation.

  For Plotinus it is worse. Substantial number is prior to being and monadic number is
  our tally, and to identify them is to mistake the instrument for the constitution —
  which is close to the error chapter 13 spent a chapter on.

  So the design's most elegant property is the one this direction rejects most firmly,
  and the rejection is not a misunderstanding. It is a considered position from people
  who thought about number harder than this project has.

  I do not think the identity should be withdrawn — the count and its notation really
  are one object here, and pretending otherwise would add a layer that does no work.
  But it should be *defended* rather than presented as obviously good, and this book
  has until now presented it as obviously good. That is what the chapter changes.
]

#recap((
  [Time as the number of motion; the instant as a limit, not a part; two senses of number, substantial and monadic.],
  [Euclid X.2's anthyphairesis *is* continued-fraction expansion — the intercalation mechanism is a Greek algorithm, which is why it is not tuneable.],
  [Archimedes built a positional hierarchy to make a cosmic magnitude expressible and landed within three orders of the present tick count. Recorded as resonance; a premise for nothing.],
  [*Conflict:* the instrument is discrete where Aristotle holds a continuum — and *Physics* IV.14 makes the datum's objectivity depend on a counter, which Rule Q concedes rather than answers.],
  [*What changes:* the design's central identity — that the number counted and the number by which we count are one integer — is precisely what this direction denies. It needs defending, not displaying.],
))
