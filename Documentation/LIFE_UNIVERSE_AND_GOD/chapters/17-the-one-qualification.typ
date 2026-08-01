#import "../design.typ": *

#chapter(number: 17, title: "The one qualification")

The preface said the practical utility of this system is questionable and that there
is exactly one qualification to that. This is it. It is one page, deliberately, and it
does not grow.

#section("The qualification")

For a single planet, this is a curiosity. Everything in Parts I through IV stands: no
task you have today needs a Planck-tick count since a stipulated datum, and the
system is actively awkward for ordinary work because nothing on its ladder is near an
hour.

For timekeeping across *two or more bodies*, the picture changes — not because this
system becomes useful, but because the alternatives become incoherent.

The alternative to a common absolute reckoning is pairwise conversion: Earth time to
Mars time, Mars time to Titan time, each pair with its own leap rules, its own epoch,
its own accumulated conventions. With $n$ bodies that is $n(n-1)\/2$ conversions to
define, agree, implement and keep correct, and no fact that all of them are about.

A universal ladder with local overlays replaces that with $n$ conversions to one
referent. Chapter 15's cross-body rendering is what it looks like: one tick count,
three calendars, each carrying its kind and its anchor revision so that values from
different determinations are never silently compared.

#claim("interpretation")[
  The argument is about *coherence*, not efficiency. Pairwise conversion is not merely
  more work; it has no common object. "Now" under it is a web of agreements about
  translation, and there is nothing the agreements are agreements about.

  With a common referent there is something — a tick count — that every local calendar
  is a rendering of. Whether anyone needs that is a separate question, and the answer
  today is no.
]

#section("What this is not")

Three things this paragraph must not be read as saying, since it is the one place in
the book where the practicality question is even partly reopened.

It is *not* a recommendation for adoption. Whether a system should be used depends on
migration cost, tooling, training, and the enormous inertia of working conventions,
and none of those favour this. The claim is that a universal ladder *could* be
coherent, not that it *ought* to be adopted.

It is *not* a prediction. Multi-body civil timekeeping is not a live problem. If it
becomes one, the people facing it will have constraints nobody here can anticipate,
and they will very likely choose something else for reasons that are good.

It is *not* a retraction of the preface. The three facts stand: the artifact is real,
its practical utility is questionable, and it is therefore research of another kind.
A conditional about a situation that does not exist is not a use case.

#callout(label: "Why this chapter is one page")[
  Because the temptation is to let it be twenty.

  There is a version of this book where the multi-body argument grows — where each
  chapter finds another way the approach would help, and the honest assessment of
  Part V's limits gets balanced against a mounting case for eventual usefulness. That
  book would be more comfortable to write and would have quietly abandoned its thesis.

  The thesis is that this is research conducted in the medium of a working program, and
  that the rigor is the medium rather than a preliminary to a payoff. A rescue of the
  practicality question would not strengthen that. It would replace it.
]

#recap((
  [For one planet the system is a curiosity, and the preface's second fact stands unretracted.],
  [For two or more bodies, pairwise conversion has no common referent: $n(n-1)\/2$ agreements about translation with nothing they are about.],
  [A universal ladder with local overlays replaces that with $n$ renderings of one value. That is a claim about coherence, not efficiency.],
  [It is not a recommendation, not a prediction, and not a retraction — and it is one page because the temptation is to let it be twenty.],
))
