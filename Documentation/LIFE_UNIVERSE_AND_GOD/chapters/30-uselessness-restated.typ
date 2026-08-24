#import "../design.typ": *

#chapter(number: 30, title: "Uselessness restated")

The preface said the practical utility of this system is questionable, and said it as
the second of three facts rather than as an apology. Two hundred pages later, it is
worth restating — because the intervening chapters have supplied a better reason for it
than the one the preface had.

#section("The title stops being a paradox")

*An Instrument for the Immeasurable.* Read as engineering, that is a contradiction with
a rueful shrug in it: an instrument that cannot measure its subject is a failed
instrument, and the title is admitting as much with good humour.

Chapter 22 makes it something else.

#claim("tradition")[
  Gregory of Nyssa, against Eunomius: διάστημα — interval, extension, the spread between
  before and after — is the mark of createdness. Everything created is διαστηματικός.
  God is ἀδιάστατος, without interval.

  The gap is not one of degree along a scale. It is a difference of *kind*: there is no
  interval in God for a measure to be a measure of.
]

Every quantity in this system is an interval. A tick count is the interval from the
datum. A `Delta` is an interval. A `Window` is an interval with its uncertainty carried.
The whole apparatus, from the 512-bit integer to the tier ladder to the certified
cosmological enclosures, measures διάστημα and nothing else.

#claim("interpretation")[
  So the title is not confessing a shortfall. It is stating a category.

  The instrument reaches exactly as far as interval reaches, which is exactly as far as
  the created order reaches, and the limit is not a failure of precision. Adding digits
  does not approach what has no interval, in the same way — Cusanus' way — that no
  finite magnitude approaches the infinite by growing.

  An instrument for the immeasurable is therefore not an instrument that failed to
  measure. It is an instrument that measures interval, pointed at something that is not
  an interval, and *declaring that this is what it is doing*.

  Which is the thesis, arrived at from the theology rather than from the type system.
]

#section("Uselessness as thesis")

With that in hand, the second fact can be restated properly.

The preface put it defensively: no task needs a Planck-tick count, the system does not
compete with `chrono`, nothing on its ladder is near an hour. All true, and all framed
as things the project is not.

#claim("interpretation")[
  The better statement is positive. This is an instrument built to measure the one
  quantity that is coextensive with the created order, from the earliest point that
  order can be said to have, at a resolution no process can exceed.

  Nothing about that description mentions utility, and utility is not what it was
  aimed at. A system built to be useful would have started from a task and worked
  backwards to a unit. This started from a unit and never acquired a task, and the
  absence is not a stage it failed to reach.

  Chapter 17's one qualification stands where it was left: for two or more bodies a
  universal ladder may be the only coherent arrangement. One page, a claim about
  coherence, not a recommendation. It is not load-bearing here and was never meant to
  be.
]

#section("The rigor is the medium")

The preface's third fact — that this is research of another kind, conducted in the
medium of a working program — has been the least examined of the three. It is worth
saying what "the rigor is the medium" means concretely, now that there is evidence.

*The tests are the argument's form.* That the constants reproduce along two independent
integer routes is not a preliminary to chapter 12's claim. It is what makes chapter 12's
claim about a real object rather than a described one. An essay about a hypothetical
type system that would enforce a distinction is a different genre and a much weaker one.

*The refusals are the content.* `UCAL-E0020` before the datum, `UCAL-E0031` beyond the
UCID range, `UCAL-E0043` below the bridge's resolution, `UCAL-E0062` for a missing
anchor, `UCAL-W0003` outside a validity window. Chapter 16 counted four epistemic limits
and found the same response in all four: error or warn, never default. That consistency
is not a coding convention. It is the position the artifact holds, expressed in the only
vocabulary an artifact has.

*The corrections are the evidence.* Chapter 10's 97/400 finding, chapter 9's sixteen
deltas and one withdrawal, chapter 28's six null results. A body of work that reported
only its successes would be making the same claims with none of the standing.

#claim("interpretation")[
  There is a version of this project that produced a paper arguing that formal systems
  should mark the boundary between what they compute and what they merely record. That
  paper would be shorter, easier to write, and read by more people.

  It would also have no `SignedWindow` in it, no compile-fail test, no sixteen deltas,
  and no case where the machinery contradicted its author. The paper would assert that
  the discipline is possible; the artifact demonstrates that it is, and pays for the
  demonstration in the only currency that counts — a specification that was wrong in
  sixteen places and said so.

  That is what "the rigor is the medium" means. Not that rigour makes the argument more
  respectable. That without the rigour there is no argument, only a proposal.
]

#section("What was actually built")

Six crates. 609 tests on two integer backends. A specification vendored, corrected in
place, and cited by the source about a thousand times, with a build step that fails if
a citation resolves to nothing — including, since 1.7.0, the citations in this book and
in every other document here, which had never been checked at all. A tier grid
generated from the library so it cannot drift. A lint that reports every exemption it
honours, and refuses to report anything if it read too few files to be believed. A
corpus of twenty-two recorded defects, one per check, so that every check is known to
object to the thing it exists for. A book, marked throughout where it stops asserting
fact, with a script that checks the marking.

None of it is useful.

#claim("interpretation")[
  I want to leave that sentence sitting there, because the temptation at the end of a
  long book is to soften it, and softening it would undo the work.

  The three facts of the preface are still the three facts. The artifact is real; its
  practical utility is questionable; therefore it is research of another kind. The
  third does not rescue the second. It *depends* on it — a useful instrument would have
  been answerable to its users, and this one is answerable only to whether it is right.

  Chapter 31 says what is not claimed, and chapter 32 says what remains after every
  claim has been withdrawn. Neither of them is a rescue either.
]

#recap((
  [Διάστημα makes the title a category rather than a paradox: the instrument measures interval, interval is the mark of createdness, and what lies beyond has none.],
  [The limit is not imprecision. Adding digits does not approach what has no interval — Cusanus' point, with Nyssa's reason under it.],
  [The positive statement: an instrument for the one quantity coextensive with the created order, at a resolution no process exceeds. Utility was never the aim, and its absence is not a stage the project failed to reach.],
  [The rigor is the medium in three concrete senses: the tests make the claim about a real object, the refusals *are* the position, and the corrections are what gives it standing.],
  [The paper version would be shorter and read more widely, and would have no case in it where the machinery contradicted its author.],
  [None of it is useful. The third fact depends on the second rather than rescuing it.],
))
