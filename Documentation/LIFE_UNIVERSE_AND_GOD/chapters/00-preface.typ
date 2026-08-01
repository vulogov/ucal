#import "../design.typ": *

#v(1cm)
#align(center)[
  #text(font: body_family, size: 20pt, weight: "bold", fill: ink_black,
    "Before we begin")
]
#v(6mm)

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
