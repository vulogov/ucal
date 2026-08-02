#import "../design.typ": *

#v(1cm)
#align(center)[
  #text(font: body_family, size: 20pt, weight: "bold", fill: ink_black,
    "Why this might be worth your time")
]
#v(6mm)

There is a distinction that people have been trying to hold steady for about
two and a half thousand years, and losing.

It is the line between what a measurement *establishes* and what it merely
*points at*. Everyone agrees it exists. Aristotle has a version of it, Basil of
Caesarea has a version of it, Kant spent a large part of the *Critique* on it,
and every one of them observed that it does not stay where you put it. Kant was
the bluntest: the illusion that erodes the line is natural, unavoidable, and
*survives being diagnosed*. You can see exactly why the moon is not larger at
the horizon and go on seeing it larger.

The usual remedy is vigilance — argue carefully, mark the boundary, and hope the
next reader is paying attention. It works for as long as attention lasts, which
is not long. In 1922 a first-rate mathematician who had the distinction fully
available to him, in his own tradition, going back fifteen centuries, crossed it
in a single paragraph and did not notice.

This book is about trying something else: holding the line with a *compiler*.

#v(2mm)

The project is a working piece of software — a calendar that counts time in
Planck units from an origin it declares it cannot measure. Inside it, the
uncertain claim about where that origin actually falls is recorded in full,
cited, with its exact magnitude — and given a type that has no arithmetic
operations at all. You can read it. You can print it. You cannot compute with
it, and three tests exist whose job is to fail to build if you ever could.

That is a small technical fact with a large consequence. A philosophical
argument reaches people who are willing to follow it. A type refuses people who
are not. Someone who thinks the distinction is pedantic sits down, writes the
line of code that ignores it, and is stopped in under a second by something with
no interest in whether they agree.

#v(2mm)

The other reason to keep reading is that the machinery answered back.

The specification behind this software claimed, twice, in two published
revisions, that its calendar-derivation mechanism reproduces both the Julian and
the Gregorian leap rules from first principles. It reproduces the Julian one
exactly. The Gregorian rule is not there — not at that depth, not at any depth,
and a rule twelve times simpler is more accurate than it.

The author found that out from his own program, about his own published claim,
and printed it. Chapter 10 is that story, and there is a check in the build now
whose only purpose is to stop anyone quietly reversing the correction.

If a book about a calendar that admits it cannot measure its own zero sounds
like it might be an elaborate way of saying nothing, chapter 10 is the answer,
and it is the shortest one available.

#v(2mm)

#section("Some of what is in here")

A calendar with no Earth in its arithmetic turns out to be a good instrument for
looking at calendars that do have Earth in theirs. Some of what came out:

/ The Julian leap rule is derivable and the Gregorian is not: #sym.dash.em Feed the mechanism
  nothing but Earth's rotation and orbit and it produces `1/4` as its first
  answer — the Julian calendar, with no knowledge of Rome. It never produces
  `97/400`. Two simpler fractions beat the Gregorian rule, one of them by a
  factor of 124.

/ The Persian calendar of 1079 *is* derivable: #sym.dash.em The rule worked out by a
  commission including Omar Khayyam is the third convergent — exactly where the
  arithmetic says the good approximation lives. So "derived" and "accurate" are
  independent properties, and the older calendar is the one the mathematics
  agrees with.

/ The Metonic cycle falls out unaided: #sym.dash.em Nineteen years, 235 lunar months — known
  to Babylon, named for an Athenian, still fixing the date of Easter and the
  Hebrew calendar's leap years. It appears as the sixth convergent of two
  numbers, with nothing supplied but Earth's periods.

/ Mars has no month, and that is the correct output: #sym.dash.em Neither of its moons is
  anything a person would call one. The mechanism returns nothing rather than
  inventing a Martian month, because "month-like" turns out to be an Earth
  predicate and there is no way to compute it for somewhere else.

/ The Hebrew calendar's epoch is the same idea, eleven centuries early: #sym.dash.em *Molad
  tohu* — "the new moon of chaos" — is a computational origin, placed before the
  event it anchors, and named for its own emptiness. Whoever fixed it understood
  precisely what this project spent four chapters working out, and named it
  better.

#section("And what kind of book it is")

Nine philosophical and religious traditions are read here — Greek, Jewish,
Islamic, Patristic, Orthodox, Latter-day Saint, modern European, Russian — and
none of them is argued to be true. They are treated as *readers of the artifact*:
each chapter says where the tradition and the software agree, and then, at
greater length, where they collide.

Every one of those chapters has a section headed *the conflict*, and four of
them cut at the project rather than at the tradition. A Baghdad argument from
1095 shows that the software took a metaphysical side without noticing it had
one. A Russian philosopher shows that its central prohibition is a contested
position rather than a safeguard. Those are in here because a tradition that
only agrees has not been read.

The book also marks itself. Where a passage is the author's interpretation
rather than a checkable fact, it sits in a ruled block that says so — and there
is a script that deletes every one of those blocks, rebuilds the book, and fails
if any technical claim stopped standing. The software marks where it stops
computing; the book marks where it stops asserting. That symmetry is the whole
design.

And there is a chapter reserved for what did not work. Six experiments were run
against real material; chapter 28 reports what each of them failed to establish,
including one that essentially failed outright and one measurement the author
cannot make and has recorded as not made.

#v(4mm)
#align(center, line(length: 26%, stroke: 0.5pt + ink_rule))
#v(4mm)

Three things are true about this project, and the third only follows if you accept
the first two. So they go here, on the first page, before anything has a chance to
soften them.

*The artifact is real.* `ucal` is working Rust — six crates, a library and a command
line, published and installable. Time is an unsigned integer count of Planck-time
units since a stipulated datum. There is no floating-point value anywhere in the
workspace, not in a signature, a field, an intermediate, or the rendering path. Every
constant in it is reproducible by two independent derivations that agree bit for bit.
At the commit this book is pinned to, 381 tests pass on both integer backends. You can
check every sentence in that paragraph; the last page of this book tells you how.

*Its practical utility is questionable.* No task you have today needs a Planck-tick
count since a stipulated datum. It does not compete with `chrono`, `time`, or
`hifitime`, and it does not try to. Nothing on its ladder of units is near a second or
an hour — a second is 21.4 beats, an hour is 24.6 arcs — so it is not merely
unnecessary for ordinary work but actively awkward for it. There is exactly one
qualification to this, it concerns timekeeping across more than one planet, it takes
up a single page in Part V, and it is not a rescue.

*Therefore it is research of another kind.* Art, philosophy, theology — conducted in
the medium of a working program. That is not a consolation prize awarded after
usefulness failed. It is what the thing is.

#section("The rigor is the medium")

Here is the part that matters, and it is easy to misread as modesty.

That the code compiles, that the constants reproduce, that the tests pass — these are
not preliminaries to the argument. They *are* the argument, in the same way that the
metre of a poem is not preliminary to the poem.

An essay can assert that a distinction ought to be respected. It can argue at length,
and persuade you, and then it is over, and the discipline it recommended survives
exactly as long as the next author's attention does. A type system can do something an
essay cannot: it can make violating the distinction *fail to build*. Not discouraged.
Not deprecated with a warning. Refused, by a machine, to a person who disagrees.

That difference is the thesis of this book, and the reason it is a book about a
program rather than a program with a book attached.

#section("What this is not")

Some of these will be obvious. They are listed because a book that surveys nine
philosophical and religious traditions alongside a Rust crate will be misread in
predictable directions, and it is cheaper to close those doors now than to keep
apologising later.

/ Not an apologetic: No tradition is argued true here. The code is evidence for none of
  them. Where a tradition and the instrument agree, that is reported as a convergence
  and never as a vindication of either.
/ Not a proof of usefulness: The second fact above stands unretracted.
/ Not a claim of discovery: When a tenth-century argument turns out to match a
  twenty-first-century type signature, the older text is not thereby shown to have
  anticipated anything. Convergence is not anticipation.
/ Not numerology: No constant, base, or magnitude in this system acquires meaning by
  resembling a number in a tradition. Base 5 was chosen because $5^5 = 3125$ gives a
  tier of exactly five digits. That is the whole reason. The book says so plainly and
  then reports, as a curiosity and not as evidence, that the first place the research
  went looking for a five, it found one waiting.
/ Not a tutorial: If you want to use the crate, its documentation is better at that
  than this book will be.
/ Not a defence of the datum: Tick zero is stipulated. That it is stipulated is the
  point, and Part IV is four chapters about why.

#section("Two paths through the book")

The book serves two readers who are rarely the same person, and it does not ask either
of them to pretend to be the other.

The *engineer's path* is Parts I, II, III, and V. It is a complete technical article
about an unusual piece of software: what it computes, what its type system refuses,
where the design lost arguments with the compiler, and how far the approach
generalises beyond Earth. You can stop at the end of Part V and have received the
whole of that.

The *reader's path* is Parts I, IV, VI, VII, and VIII. It is a book about what it
means to build a measuring instrument that points at something it declares itself
unable to measure, and about nine traditions that have thought carefully about origins,
number, and time — read here as readers of the artifact rather than as authorities over
it.

Part I belongs to both. Nothing later in the book may redefine what Part I establishes.

#callout(label: "A note on marking")[
  This book marks its own claims. Where a passage is *interpretation* — the author
  reading a meaning into the artifact — it sits in a ruled block that says so. Where a
  coincidence is genuinely striking but proves nothing, it is labelled *resonance*, and
  it may never appear as a premise for anything.

  That apparatus is not decoration. It is the book submitting to the discipline it
  documents: the software marks where it stops computing, and the book marks where it
  stops asserting fact. If you delete every marked block, every technical claim in
  Parts I through III and V must still stand. Making sure that is true is a scheduled
  step in producing this book, not a hope about it.
]

#section("A word about being wrong")

The most valuable document in this project's history is a correction.

Two revisions of the specification asserted that the continued-fraction machinery
reproduces the Julian and Gregorian calendar rules as convergents. The Julian rule does
appear — it is the first convergent. The Gregorian rule does not appear at any depth,
and a rule twelve times simpler is more accurate than it. The machinery contradicted its
author, in writing, on a claim he had published twice.

That correction is in this book, dated, including the revisions that were wrong. It is
in here because a system built to make its author's conclusions provable would never
have produced it, and because the charge that such systems are *always* built that way
is nine hundred years old and deserves an answer made of evidence rather than assurance.

If you find yourself reading a passage that seems to be presenting that correction as
foresight rather than as an error published — a demonstration of rigour rather than an
argument the author lost — then the book has failed at the thing it cares most about,
and you should say so.
