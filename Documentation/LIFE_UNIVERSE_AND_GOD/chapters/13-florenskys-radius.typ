#import "../design.typ": *

#chapter(number: 13, title: "Florensky's radius")

Chapter 12 described a rule against letting a formal artifact be read as a physical
fact. Rules against things exist because the things happen.

This chapter is about the clearest case of it happening, in a book by a mathematician
who was better at this than the present author, published in 1922, which contains
both the strongest precedent for what this project attempts and the exact failure it
guards against — separated by a few pages.

#section("Мнимости в геометрии")

*Imaginaries in Geometry*, by Pavel Florensky: mathematician, electrical engineer,
Orthodox priest, and the author of a study of geometry on the projective plane.

The book's project is to give geometric meaning to imaginary quantities, and most of
it is exactly what it says. Then, at the end, it turns to Dante.

#section("What Florensky got right")

The achievement first, because it is real and because the failure is only interesting
if the author was capable.

#claim("tradition")[
  Consider the *Comedy*'s trajectory. Dante and Virgil descend through the earth to
  its centre, pass the point of gravity, and continue — and emerge at the Mount of
  Purgatory on the far side. Dante then ascends through the spheres, past the fixed
  stars, to the Empyrean, and the poem ends.

  In a Euclidean cosmos the trajectory does not close. You go down, keep going, come
  out somewhere else, go up a long way, and stop. Standard diagrams of Dante's
  universe handle this by not quite drawing it.

  Florensky's observation is that the journey closes — that it returns coherently to
  its start — if the cosmos is *finite and non-orientable*. The traveller who passes
  through the centre and continues arrives back, reoriented, without ever turning
  around.
]

That is a genuine piece of mathematical reading. It takes a structural feature of a
poem seriously enough to ask what geometry the poem presupposes, and the answer is
neither trivial nor forced. Whatever one thinks of the rest, this part is good work.

#block(breakable: false, width: 100%)[
#v(3mm)
#align(center, image("../assets/images/florensky-cosmos.png", width: 74%))
#v(1mm)
#figcap[11][
  The cosmos Florensky's reading requires. The spheres carry their zodiacal and
  planetary signs; the infernal cone descends at the centre; the Mount of
  Purgatory rises beneath it; the Empyrean is at the upper right. The single
  traced line is the *Comedy*'s trajectory, and it closes — because the outermost
  sphere has rejoined the innermost without the traveller ever turning round.
]
]

#callout(label: "What this plate is")[
  An illustration made for this book, in the manner of a sixteenth-century
  cosmological engraving. It is *not* a reproduction of any diagram Florensky
  drew, and *Мнимости в геометрии* contains no plate like it.

  It is here because the geometry is the chapter's whole argument and prose is
  poor at conveying a shape. Saying what it is and is not costs one paragraph,
  and a book that spends four chapters on the difference between a structure and
  a claim about the world should not slip a modern picture in as a historical
  document.
]

#section("What Florensky did next")

In §9 of the same book, he turns to relativity.

The argument runs: consider a cosmos rotating rigidly. At a sufficient radius the
co-rotating velocity would exceed $c$. In the Lorentz factor $sqrt(1 - v^2 \/ c^2)$,
that makes the quantity under the root negative, and lengths and durations become
imaginary. He computes the radius at which this happens. It comes out near the
distance to the sphere of the fixed stars in the Ptolemaic scheme.

And then he identifies that surface with the Empyrean.

#claim("interpretation")[
  The distance between the Dante geometry and this is exactly one move, and it is worth
  naming precisely, because from the outside they look like the same kind of reasoning.

  In the first, a formal structure is used to *interpret* a text: the poem's
  trajectory has a property, and a geometry in which that property holds is exhibited.
  The claim is about the poem.

  In the second, a formal artifact — the imaginary value of an expression outside its
  domain of validity — is treated as *designating a place*. The claim is about the
  world, and it is licensed by nothing except that the number came out at a suggestive
  magnitude.

  One move. There is no third thing between them, and no bright line in the prose
  where the register changes.
]

#section("What was missing")

Florensky had no rule against the second move.

That is not a moral observation. He had a formal apparatus, a serious theological
project, and enormous mathematical facility, and no principle in his method that said:
*here is where the mathematics stops describing and starts being read*. Nothing in his
toolkit distinguished a structure that carries meaning from a structure that is a
physical fact.

#claim("interpretation")[
  This project has such a rule, and chapter 12 is what it looks like implemented: a
  claim about the world, declared with its citation and its magnitude, and made
  incapable of entering a computation.

  It would be comfortable to conclude that the rule makes this project safe from
  Florensky's error. It does not, and saying so is the point of putting this chapter
  in Part IV rather than in Part VI.

  What the rule protects is the *arithmetic*. `BIG_BANG_CLAIM` cannot become an
  operand. But nothing prevents a reader — or the author, on a bad day — from looking
  at a 61-digit integer and reading it as a fact about being rather than a count under
  a stipulation. That is a failure of interpretation, not of arithmetic, and no type
  system reaches it.

  The last chapter of this book is about exactly that residue.
]

#section("Basil, fifteen centuries earlier")

The insight Florensky's §9 lacked had been available for a very long time.

#claim("tradition")[
  Basil of Caesarea, in the *Hexaemeron*, addresses what "in the beginning" means and
  answers with an image: the beginning of a road is not itself the road, and the
  beginning of a house is not the house. A beginning is not a member of the series it
  begins.

  Applied to time: the beginning of time is not a time. It is not a very short
  duration, not the first instant of a sequence, not a moment you could in principle
  point at. It is of a different kind entirely.
]

That is Rule Q's content, stated in the fourth century with no arithmetic whatsoever.
The datum is not the first tick of the universe; it is where counting starts, and
those are different sorts of thing.

#callout(label: "What this convergence is and is not")[
  Basil is not being credited with anticipating anything, and Rule Q is not being
  validated by his agreement. Chapter 21 handles the Patristic material properly,
  including where it conflicts with this project.

  The point here is narrower and it is about Florensky. The distinction he needed in
  §9 was not obscure, not recent, and not outside his tradition — he was an Orthodox
  priest, and Basil is not a marginal figure in it. He had every resource required to
  see the problem, and the problem was not visible from where he stood.

  That is the useful version of the cautionary tale. Not *he lacked something*, but
  *he had it and it did not help*, because nothing in his working method forced him to
  reach for it at the moment it was needed.
]

#section("What it cost")

The chapter should not end on a methodological note, because the people in it did not.

Florensky was persecuted from the early 1920s onward. He continued working — on
electrical engineering, on materials science, producing real technical results — under
conditions that grew steadily worse. He was arrested, sent to Solovki, and executed in
1937.

Alexei Losev, whose work on number and name appears in chapter 25 and whose position
Rule N declines, was imprisoned and sent to the White Sea Canal construction. He went
nearly blind there. He survived, and spent decades publishing under constraints that
required him to disguise what he was arguing.

#claim("interpretation")[
  It matters that this book's clearest cautionary tale is a man who was killed for the
  intellectual tradition the tale is drawn from.

  The failure identified in §9 is a real failure and this chapter does not soften it.
  But there is a way of writing about historical mistakes that treats their authors as
  material, and it is available here in a particularly cheap form: the mathematician
  who confused a formal artifact with a place, held up as a lesson.

  He was a serious thinker working with the resources he had, on a question this
  project also finds worth asking, and he paid a price for the asking that no one
  writing today is being asked to pay. The rule in chapter 12 exists because his error
  is easy, not because he was careless.
]

#recap((
  [Florensky's reading of the *Comedy* — that its trajectory closes only in a finite, non-orientable cosmos — is genuine mathematical work on a real structural feature of the poem.],
  [In §9 of the same book he computes a radius from the Lorentz factor outside its domain of validity and identifies the resulting surface with the Empyrean.],
  [The distance between the two is exactly one move: interpreting a structure, versus asserting that a formal artifact designates a place.],
  [He had no rule against the second move, and Basil's road-and-house image — the distinction he needed — was fifteen centuries old and inside his own tradition.],
  [Chapter 12's rule protects the arithmetic. It does not protect the reading, and the last chapter of this book is about that residue.],
  [Florensky was executed in 1937; Losev went nearly blind at Belomorkanal. The error is easy, which is why the rule exists — not because its author was careless.],
))
