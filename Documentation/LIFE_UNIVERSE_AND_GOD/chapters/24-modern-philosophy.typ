#import "../design.typ": *

#chapter(number: 24, title: "Modern philosophy")

Chapter 11 borrowed the First Antinomy and chapter 12 borrowed the
constitutive/regulative distinction. This chapter reads Kant properly, which means
reading the parts that do not help.

#section("What the direction holds")

#claim("tradition")[
  *Absolute against relational.* Newton's *Principia* declares absolute time, "of
  itself, and from its own nature, flowing equably without relation to anything
  external." Leibniz replies in the correspondence with Clarke that time is an order of
  successions and nothing more — that a universe shifted uniformly in time would be
  indiscernible from this one, and indiscernibles are identical.

  *A- and B-series.* McTaggart distinguishes the A-series — past, present, future,
  which shift as time passes — from the B-series — earlier-than and later-than, which
  do not. He argues the A-series is contradictory and the B-series alone cannot
  constitute time.

  *Kant's forms.* Time is a pure form of sensible intuition: not something perceived but
  the form under which anything is perceived. It is *empirically real* — every object of
  experience is in time — and *transcendentally ideal* — it is not a property of things
  as they are in themselves.

  *The First Antinomy.* Thesis: the world has a beginning in time. Antithesis: it does
  not. Both proofs valid; both conclusions false, because both assume the world-series
  is a completed given totality.

  *Number as schema.* In the Schematism, number is the schema of magnitude: the
  successive addition of homogeneous units, the representation of a procedure for
  synthesising a quantity.

  *Transcendental illusion.* The illusion is not a mistake to be corrected. It is
  natural and unavoidable, and it persists after diagnosis — "as the astronomer cannot
  prevent the moon from appearing larger at its rising, although he is not deceived by
  this illusion."
]

#section("Which rule it illuminates")

Rule Q and `UCAL-E0025`. N1. Rule F, the declared frame. Rule G, on number.

#section("The convergences")

#subsection("The First Antinomy is Rule Q's fourth reason")

Chapter 11 gave this in full: the specification offers three reasons the datum cannot be
measured, and Kant supplies a fourth it does not state — the question presupposes a
completed totality that is not a possible object of knowledge.

What chapter 11 did not say is where the antithesis argument lands.

#claim("tradition")[
  Kant's argument against the thesis runs: suppose the world began. Then there was an
  empty time before it. But no part of empty time has any distinguishing condition of
  existence rather than non-existence — one moment of nothing is indistinguishable from
  another — so no reason could determine the world to begin *then* rather than at some
  other moment. Therefore it did not begin.
]

That argument is why Rule Z is right to treat a "before" as malformed rather than as a
region with values in it. An empty time before the datum would have no distinguishable
parts, so a tick count within it would be a coordinate on a structure with no
distinctions to index. `UCAL-E0020` refuses to produce one.

#subsection("Quanta continua gives N1 a better defence")

Chapter 2 declined to claim the tick is a quantum of time and gave a procedural reason:
the discreteness is a fact about the instrument, and taking a metaphysical position as
a side effect of choosing an integer type is not respectable.

Kant supplies a stronger reason. Space and time are *quanta continua*: magnitudes in
which no part is the smallest, and the parts are possible only through limitation. The
instant is a limit, not a part — the same position chapter 18 found in Aristotle, now
argued from the form of intuition rather than from the analysis of motion.

So N1's refusal is not merely modest. It declines a claim that two of the most careful
treatments of time in the Western tradition independently argue is false.

#subsection("Constitutive and regulative, enforced")

Chapter 12 made this its closing move and it is worth restating precisely.

The datum is a regulative posit: *count from here*. The physical claim is
constitutive-looking: *this is where time began*. Kant's illusion is the slide from the
first to the second.

`UCAL-E0025` and the compile-fail tests make the slide impossible in the arithmetic.
Kant policed the boundary by argument and vigilance; here it is policed by a type with
no operators.

#conflict[
  *Two, and the first cuts at chapter 18's finding from the other side.*

  *Kant's number is monadic.* In the Schematism, number is the schema of magnitude — the
  successive addition of homogeneous units. That is exactly Plotinus' μοναδικὸς
  ἀριθμός, the counting-number, and it is exactly what Plotinus says is *not* the number
  that is constitutive of being.

  Chapter 18 found that Rule G asserts an identity between the number counted and the
  number by which we count, and that the Greeks would deny it. Kant denies it too, from
  the opposite direction: for him there is only the monadic sense, and the substantial
  sense is not a further kind of number but a confusion.

  So Rule G is caught between two rejections. Plotinus says the identity collapses a
  distinction that matters; Kant says one of the two terms does not exist. The rule
  survives by declining to be a claim about number at all — but it is stated as one.

  *UC-Θ asserts a first term.* The unbuilt profile posits that time begins to be at
  organization. That is the thesis of the First Antinomy, and Kant's whole point is
  that it cannot be asserted — not that it is false in a way the antithesis is true,
  but that asserting either is a transcendental error.

  Chapter 21 found UC-Θ heterodox by the Patristic standard. Kant finds it *ill-formed*,
  which is worse, and he finds UC-1's stipulation acceptable for exactly the reason
  UC-Θ's assertion is not: a stipulated origin claims nothing about the series.
]

#section("What it changes")

#claim("interpretation")[
  Transcendental illusion is incurable, and that is what chapter 12's achievement is
  bounded by.

  The astronomer knows the moon is not larger at the horizon. He sees it larger anyway.
  Diagnosis does not dissolve the appearance; it only stops him being deceived.

  `UCAL-E0025` is diagnosis with teeth. It stops the claim entering the arithmetic —
  which is a real thing to have stopped, and it is not the same as stopping the
  illusion. Every reader who sees a 61-digit integer will read it as a fact about being,
  because that is what a very precise number looks like. No type prevents that. Nothing
  prevents that.

  So the honest form of the book's claim is narrower than Part VIII would like: the
  instrument contains the illusion rather than curing it. It refuses to compute with a
  claim it cannot help but suggest.

  Chapter 32 is that, and this chapter is what earns it.
]

#recap((
  [Newton's absolute time against Leibniz's order of successions; McTaggart's two series; Kant's time as a form of intuition, empirically real and transcendentally ideal.],
  [The First Antinomy supplies Rule Q's unstated fourth reason, and its antithesis argument is why an empty "before" has no distinguishable parts to index.],
  [*Quanta continua* gives N1 a stronger defence than modesty: the instant is a limit, not a part.],
  [*Conflict:* Kant's number is monadic, so Rule G is rejected from both sides — Plotinus says it collapses a distinction, Kant says one of its terms does not exist.],
  [*Conflict:* UC-Θ asserts a first term of the series, which the First Antinomy holds cannot be asserted at all. Ill-formed, not merely heterodox.],
  [*What changes:* the illusion is incurable, so `UCAL-E0025` contains rather than cures. The instrument refuses to compute with a claim it cannot help but suggest.],
))
