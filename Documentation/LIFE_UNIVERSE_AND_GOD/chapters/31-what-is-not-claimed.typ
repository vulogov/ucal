#import "../design.typ": *

#chapter(number: 31, title: "What is not claimed")

A negative inventory. Every item is a sentence this book was in a position to write and
did not, and each is listed with what would have made it writable.

It is here because a book that surveys nine traditions alongside a Rust workspace will
be quoted, and the quoting will not always be careful. This chapter is what the author
can point at.

#section("About time and the datum")

*Not claimed: that time began.* The system stipulates a datum and counts from it.
Chapter 11 gave four reasons the origin cannot be measured, and Kant's is that the
question presupposes something not available to be true. UC-1 asserts that tick zero is
where counting starts. It asserts nothing about whether anything started.

*Not claimed: that the Big Bang happened at tick 0.* `BIG_BANG_CLAIM` records a
published identification with its uncertainty, cited, as metadata. Chapter 12 made it
impossible to compute with. A recorded claim is not an endorsed one, and the type is
the difference.

*Not claimed: that the tick is the smallest unit of time.* Chapter 2 declined this
explicitly. The tick is the resolution floor of an instrument. Whether the world has a
smallest interval is a question this project takes no position on, and would have taken
one on by accident had it not said so.

*Not claimed: that the uncertainty in the age of the universe makes timestamps
uncertain.* It does not, because it is not an operand. This is the one negative claim
the artifact enforces rather than the author asserting.

#section("About the traditions")

*Not claimed: that any tradition surveyed is correct.* No chapter of Part VI concludes
that a direction got something right about the world. Convergences are reported as
convergences.

*Not claimed: that any tradition anticipated this work.* Chapter 18 found Euclid X.2
*is* the intercalation algorithm; chapter 19 found *molad tohu* stating Rule Q's content
nineteen centuries early; chapter 23 found D&C 130:4–5 stating Rule K's premise in 1843.
None of those is anticipation. There is one good algorithm for continued fractions, one
obvious solution to needing an exact origin, and body-relative reckoning is a thought
available to anyone who considers another planet.

*Not claimed: that the convergences cannot be coincidence.* Some of them are structural
— Euclid's algorithm and this one are the same algorithm, and that is not luck. Some are
coincidence, and chapter 18 labelled Archimedes' $10^63$ as resonance and used it for
nothing.

*Not claimed: that any tradition is wrong.* Chapter 19 is four pages on this. The
instrument can compute a finding about *Seder Olam*'s chronology; the author does not
conclude from it. Chapter 28 records that the audit was not merely withheld but not run,
and that the restraint is partly standing in for an absence of competence.

*Not claimed: that this survey is adequate to any of them.* Chapter 26 says the method
holds the code constant and lets the traditions vary, that this is a position rather
than a neutral frame, and that the reverse book would ask better questions this one
cannot reach.

#section("About the design")

*Not claimed: that base 5 is meaningful.* $5^5 = 3125$ is five base-5 digits. That is
the whole reason, stated in chapter 4 before Part VI could be misread, and Rule N
forbids any constant acquiring significance by resembling a number in a tradition.

*Not claimed: that the system is useful.* Chapter 30. The one qualification in chapter
17 is about coherence across bodies and is not a recommendation for adoption.

*Not claimed: that anyone should adopt this calendar.* Explicitly not, and the chapter
that could have grown into an argument for it was held to one page for that reason.

*Not claimed: that the approach works everywhere.* Chapter 16 lists six limits at
greater length than chapter 15 lists capabilities. Four are epistemic and in all four
the system errors or warns rather than defaulting.

*Not claimed: that the design has no unargued commitments.* Three were found in Part VI
alone — a Rushdian ontology in chapter 20, a clean structure/reading line in chapter 25,
the comparative frame in chapter 26. All three were discovered by reading the artifact
against traditions, and none is declared in the specification.

#section("About the medium")

*Not claimed: that mechanical enforcement is a better argument.* Chapter 29 says the
opposite: it is a stronger way to maintain a distinction and a worse way to argue for
one, and the asymmetry is why the book exists alongside the artifact.

*Not claimed: that a compiler settles anything about truth.* If the rule is wrong, the
compiler enforces a wrong rule perfectly. That the distinction enforced here is the
right one is not established by the enforcement, and is not established by this book
either.

*Not claimed: that the instrument reaches God.* It measures διάστημα, which chapter 22
establishes is the mark of the created order by definition. That is not a limitation
being confessed; it is what an interval-measure is.

#section("The one thing that is claimed")

Stated once, with the qualifications attached rather than deferred.

#v(3mm)
#align(center, block(width: 84%, breakable: false)[
  #set par(justify: false)
  #text(size: 11.5pt, style: "italic")[
    A measuring instrument may legitimately point at what it cannot describe, provided
    it declares that it is only pointing — and that declaration can be enforced
    mechanically rather than left to the author's discipline.
  ]
])
#v(3mm)

The first clause belongs to Cusanus, Gregory of Nyssa, and Frank. The second is this
project's, and it means one thing: `BIG_BANG_CLAIM` is fully readable, fully cited,
carries an exact magnitude, and cannot enter a computation — enforced by a type with no
operators and three tests that must fail to build.

That is all. It is a small claim about a narrow mechanism, and the reason it took two
hundred pages is that establishing a small claim honestly requires reporting everything
that did not establish it.

#claim("interpretation")[
  A reader who takes only one sentence from this book should take this one: *the
  distinction between what a system computes and what it merely records can be made
  mechanical, and until it is mechanical it depends on someone remembering.*

  Everything else here — the nine traditions, the six samples, the fourteen deltas, the
  convergent ladders — is either evidence for that or an honest account of what failed
  to be.
]

#recap((
  [Not claimed: that time began, that the Big Bang was at tick 0, that the tick is time's smallest unit, or that the age uncertainty affects timestamps.],
  [Not claimed: that any tradition is correct, anticipated this, or is wrong — nor that this survey is adequate to any of them.],
  [Not claimed: that base 5 is meaningful, that the system is useful, that anyone should adopt it, or that the approach works everywhere.],
  [Not claimed: that the design has no unargued commitments. Three were found in Part VI alone, none declared in the specification.],
  [Not claimed: that mechanical enforcement is a better argument, that a compiler settles truth, or that the instrument reaches God.],
  [Claimed: that the declaration can be enforced mechanically. One small claim about one narrow mechanism, and two hundred pages of reporting what did not establish it.],
))
