#import "../design.typ": *

#chapter(number: 29, title: "Why a program")

The thesis, restated with everything the book has gathered behind it:

#v(3mm)
#align(center, block(width: 84%)[
  #set par(justify: false)
  #text(size: 12pt, style: "italic", fill: ink_black)[
    A measuring instrument may legitimately point at what it cannot describe, provided
    it declares that it is only pointing — and that declaration can be enforced
    mechanically rather than left to the author's discipline.
  ]
])
#v(3mm)

The first clause is not new and this book has been careful to say whose it is. Cusanus
has it in 1440: no proportion holds between finite and infinite, so precision does not
constitute approach. Gregory of Nyssa has the reason a century before: διάστημα is the
mark of createdness, and what lies beyond has no interval for a measure to measure.
Frank reaches it independently in 1939.

The second clause is the contribution, and this chapter is what it amounts to.

#section("The difference a compiler makes")

An essay can assert that a distinction ought to be respected. It can argue at length,
and persuade you, and then it ends — and the discipline it recommended survives exactly
as long as the next reader's attention does.

Chapter 12's `SignedWindow` does something an essay cannot. It refuses.

#claim("interpretation")[
  Consider the three ways a rule can be maintained, in ascending order of how much they
  ask of the person maintaining them.

  A *comment* addresses a reader who is paying attention, at the moment they are reading
  that line, and only if they agree.

  A *runtime check* addresses a program that reaches a particular line of execution. It
  is stronger — it does not depend on agreement — and it is bounded by coverage: a path
  nobody thought about is a path the check does not guard.

  A *type* addresses everyone who ever writes code against the library, at compile time,
  on every path, including the ones nobody thought about. It does not require the
  programmer to have read anything. It does not require them to agree. It requires
  nothing of them at all.

  That is not a difference of degree. A comment is a request, a runtime check is a
  guard, and a type is a *precondition of the code existing*.
]

#section("Why this is philosophical and not merely technical")

The obvious objection: enforcing an invariant with a type is good engineering, and good
engineering is not philosophy. Every well-typed program encodes constraints. Why should
this one count as anything more?

Three answers, in increasing order of how much they claim.

*First: the constraint is about what may be asserted, not about what is well-formed.*
Most type-level invariants encode structural facts — a list is non-empty, an index is in
range. `UCAL-E0025` encodes an epistemological one: that a claim about the world with a
published uncertainty may be *recorded* and may not be *computed with*. That is not a
statement about data shape. It is a rule about the relationship between measurement and
inference, and it happens to be expressible as a missing trait implementation.

*Second: the distinction it enforces is one that has needed enforcing for a long time
and has never had a mechanism.* Kant maintained the constitutive/regulative boundary by
argument and vigilance, and said explicitly that the illusion it guards against
persists after diagnosis. Florensky, working in a tradition that had the necessary
distinction available for fifteen centuries, crossed it in one paragraph of a book he
was otherwise right in. This is not a boundary people fail to cross for want of being
told about it.

*Third — and this is the strong claim — a program can be run by someone who
disagrees.*

#claim("interpretation")[
  This is where the argument stops being about engineering.

  A philosophical argument reaches only those willing to follow it. Its whole mode of
  operation is voluntary. Someone who rejects the premise is untouched, and there is no
  version of the essay that reaches them, because reaching them was never what an essay
  does.

  A program does not ask. Someone who thinks the distinction between a stipulated datum
  and a measured origin is pedantic — who thinks the uncertainty should just be added
  in — sits down, writes `t.checked_add(&claim)`, and the compiler says no. They have
  not been persuaded. They have been *stopped*, in under a second, by something with no
  interest in whether they agree.

  I am aware of how this can be misread, so: this is not an argument that force beats
  reason, and a compiler is not an authority about what is true. If the rule is wrong,
  the compiler enforces a wrong rule perfectly.

  The claim is narrower. It is that for a distinction whose failure mode is *drift* —
  gradual erosion by people who were never persuaded either way, one convenience at a
  time — a mechanism that does not depend on persuasion is a different kind of
  instrument, and philosophy has not had one.
]

#section("What the medium costs")

The argument would be cheap without this section.

*A program's argument is only as good as its premises, and it hides them.* Chapter 20
found the artifact assuming a Rushdian ontology — that periods have natures from which
their behaviour follows — with no declaration anywhere. Chapter 25 found it assuming a
clean line between structure and reading, which Losev denies. Chapter 26 found the book
assuming the code is the invariant and the traditions the variables. Three metaphysical
commitments, none argued, all discovered by reading the artifact against traditions
rather than by inspecting it.

An essay wears its premises. A program buries them in type signatures where they
function silently and are found only if someone comes looking.

*A program's audience is smaller than an essay's.* The `SignedWindow` argument is fully
available to people who read Rust. To everyone else it is a claim about a piece of
code, which is exactly the position an essay is in.

*A program can be wrong with more authority.* A false claim in an essay is contestable
by anyone who reads it. A false claim compiled into a type is enforced against everyone
downstream, and the enforcement carries no indication of whether the rule is good.

#claim("interpretation")[
  So the honest form of the thesis has a clause the preface did not include: mechanical
  enforcement is a *stronger* way to maintain a distinction and a *worse* way to argue
  for one.

  This book exists because of that asymmetry. The artifact enforces; the book argues. If
  the enforcing could argue, there would be no need for 200 pages, and there would also
  be no way to find out that three of the artifact's commitments were never argued at
  all.
]

#section("What the book claims, exactly")

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5.5pt),
    [*claim*], [*status*],
    [An instrument may point at what it cannot describe],
      [old — Cusanus, Nyssa, Frank. Not this project's],
    [That declaration can be enforced mechanically],
      [this project's contribution],
    [A type addresses those who disagree, as argument cannot],
      [the strong form, argued here],
    [Mechanical enforcement is a better *argument*],
      [*not* claimed — it is a worse one],
    [The distinction enforced here is the right one],
      [not established by the enforcement, and not by this book],
  )
]
#v(2mm)

The fourth and fifth rows are what keep the third honest. A compiler settles nothing
about whether a rule deserves to be enforced. It settles that the rule *is* enforced,
which was the whole difficulty.

#recap((
  [The thesis' first clause is Cusanus', Nyssa's and Frank's. The second — that the declaration can be enforced mechanically — is this project's.],
  [Comment, runtime check, type: a request, a guard, and a precondition of the code existing. The last is a difference in kind.],
  [It is philosophical because the constraint governs what may be *asserted*, because the distinction has never had a mechanism, and because a program can be run by someone who disagrees.],
  [The cost: a program hides its premises — three unargued commitments were found only by reading it against traditions — reaches a smaller audience, and can be wrong with more authority.],
  [Mechanical enforcement is a stronger way to *maintain* a distinction and a worse way to *argue for* one. That asymmetry is why the book exists alongside the artifact.],
))
