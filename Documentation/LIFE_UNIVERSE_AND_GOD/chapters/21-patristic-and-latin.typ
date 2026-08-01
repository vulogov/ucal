#import "../design.typ": *

#chapter(number: 21, title: "Patristic and Latin")

Chapter 13 borrowed one image from this direction — Basil's road, which is not its own
beginning — and used it against Florensky. This chapter reads the direction properly,
and it produces the sharpest conflict in Part VI: the profile mechanism, which was
built for versioning, turns out to be doing theological work its author did not
design it for.

#section("What the direction holds")

#claim("tradition")[
  *Creation ex nihilo, including time.* Augustine, *City of God* XI.6: the world was
  made not *in* time but *with* time. Time is among the things created, so there is no
  interval before creation for the question "what was God doing then?" to be about.
  The question is malformed rather than hard.

  *The beginning of time is not a time.* Basil, *Hexaemeron* I.6: the beginning of a
  road is not the road, and the beginning of a house is not the house. A beginning is
  not a member of the series it begins.

  *Aevum.* Boethius and later Aquinas distinguish three modes of duration: *tempus*,
  successive time; *aeternitas*, the wholly simultaneous possession of unlimited life;
  and *aevum*, the mode belonging to created things that do not change successively.

  *Act and terminus.* Aquinas, *Summa Theologiae* I.13.7: when God is said to become
  Lord of creatures, the relation is real in the creature and not in God. A timeless
  act can have a temporal terminus, and dating the terminus does not date the act.

  *Learned ignorance.* Nicholas of Cusa, *De docta ignorantia*: the infinite is not
  reached by any proportion from the finite. Increasing the precision of a finite
  measure does not approach it, because approach requires proportion and there is
  none.
]

#section("Which rule it illuminates")

Rule Z, the unsigned domain. Rule Q, the stipulated datum. And the thesis of this book,
which Cusanus states first.

#section("The convergences")

#subsection("`UCAL-E0020` is Augustine's answer")

Chapter 3 said that asking for an instant before the datum returns an error rather than
a negative number, and that the difference is between refusing a question and answering
it.

Augustine's answer to *what was God doing before creating heaven and earth* is exactly
that structure. He declines the joke ("preparing hell for those who pry") and gives the
serious reply: there was no *then*, because time is among the created things. The
question presupposes a container that does not exist.

An error is a malformed query. A negative number is a well-formed query with a value on
the other side of a chosen origin. The system returns the first, and the fifth-century
argument for why is the better one available.

#subsection("Basil, and Rule Q's actual content")

Chapter 13 used the road-and-house image as a diagnosis of what Florensky's §9 lacked.
It is worth restating as a positive claim about the datum.

The datum is not the first tick of the universe. It is where counting starts. Those are
different sorts of thing, and the difference is not a technicality — it is the whole
reason the physical claim had to be held in a separate, non-computable type.

#claim("interpretation")[
  Basil is doing with an image what chapter 12 does with a type. Both are keeping a
  beginning from being treated as a member of the series it begins.

  The image is more elegant and reaches more people. The type is enforced.

  I do not think that ranks them. It marks a difference in what each medium can do,
  which is the argument of Part VIII arriving early and from an unexpected direction.
]

#subsection("Aquinas makes the project coherent under a timeless God")

This convergence matters more than it first appears, because it removes what would
otherwise be a fatal objection.

If God is timeless, dating divine action looks incoherent: an act with no temporal
location cannot be assigned a date. And a system that offers to timestamp anything at
all would seem to be committed to denying divine timelessness.

*ST* I.13.7 dissolves this. The relation is real in the creature and not in God. The
*terminus* of a timeless act is a created thing with a location in the created order,
and dating the terminus is dating the creature, not the act.

So a calendar can coherently timestamp events in the created order without any
commitment about how the acts that produced them are related to time.

#subsection("Cusanus states the thesis first")

*De docta ignorantia*: no proportion holds between the finite and the infinite, so no
increase in finite precision constitutes approach.

That is this book's thesis, arrived at five and a half centuries earlier and stated
more cleanly: *a measuring instrument may legitimately point at what it cannot
describe, provided it declares that it is only pointing.* The declaring is what learned
ignorance means — knowing the character of what you do not know, rather than merely
not knowing it.

#claim("interpretation")[
  The contribution this project can claim is not the thesis. Cusanus has it, and Gregory
  of Nyssa has a version before him.

  It is the second clause: that the declaration can be *enforced mechanically* rather
  than left to the author's discipline. Cusanus maintained learned ignorance by being
  Cusanus. Chapter 12 maintains it with a type that has no operators.
]

#conflict[
  *UC-Θ is heterodox by this standard, and the book must say so plainly.*

  Creation *ex nihilo* is not a peripheral commitment in this direction. It is held
  firmly, argued for repeatedly, and defined against precisely the alternative that
  chapter 12's unbuilt profile assumes.

  UC-Θ posits a datum at *organization* — matter arranged rather than created from
  nothing, with an origin of ordering that is not an origin of existence. That is,
  in the terms Augustine and Aquinas use, the Platonic opinion: pre-existent material
  shaped by a demiurge. It is the position these authors spent centuries refuting.

  So the two profiles are not two configurations of one system. They are two
  cosmologies, and by this direction's standard one of them is orthodox and the other
  is not.

  The book does not get to soften this by observing that UC-Θ is unbuilt. A profile
  that has been specified is a position that has been taken seriously.
]

#section("What it changes")

#claim("interpretation")[
  Rule P was written for failure mode F1: timestamps shifting when the age constant is
  revised. Profiles are named, versioned, type-bound, and tagged in every serialised
  form so that a value from one cannot be silently compared with a value from another.

  That is a versioning mechanism. It was designed by an engineer thinking about
  constant revision.

  What this chapter establishes is that it is *also* the mechanism that keeps two
  incompatible cosmologies from contaminating each other's timestamps — and that this
  is not an analogy. UC-1 and UC-Θ differ in what the datum *is*, which means a
  timestamp under one is not the same kind of statement as a timestamp under the other,
  which is exactly the condition Rule P exists to detect.

  The rule is doing theological work. Nobody designed it to.

  Two things follow. First, the specification should say so — a mechanism whose scope
  is wider than its stated purpose is under-documented, and the gap is the same kind
  chapter 20 found in Rule F.

  Second, and less comfortably: the fact that a versioning rule turned out to be
  adequate for this does not mean it is *sufficient* for it. Rule P prevents silent
  comparison. It does not require a profile to declare its cosmological commitments,
  and UC-Θ's commitments are exactly what a reader would need to see.
]

#recap((
  [Creation *ex nihilo* including time; the beginning of time is not a time; *aevum* as a third mode of duration; act and terminus; learned ignorance.],
  [`UCAL-E0020` is Augustine's move — a malformed question refused, not answered with a value on the far side of an origin.],
  [Aquinas' *ST* I.13.7 removes what would otherwise be fatal: dating a terminus in the created order commits you to nothing about how a timeless act relates to time.],
  [Cusanus states this book's thesis first. The contribution is not the thesis but the second clause — that the declaration can be enforced mechanically.],
  [*Conflict:* UC-Θ assumes organization rather than creation from nothing, which is the position this direction was built to refute. It is heterodox by this standard and the book says so.],
  [*What changes:* Rule P was written for versioning and turns out to be what keeps two cosmologies apart — doing theological work nobody designed it for, and insufficient for the job, since it never requires a profile to declare what it assumes.],
))
