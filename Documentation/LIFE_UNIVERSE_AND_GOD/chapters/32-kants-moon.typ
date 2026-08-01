#import "../design.typ": *

#chapter(number: 32, title: "Kant's moon")

#claim("tradition")[
  Transcendental illusion is not a mistake. It is not carelessness, and it is not
  something a sufficiently careful thinker avoids.

  Kant's word is *natural and unavoidable*. It arises from the structure of reason
  itself, and — the part that matters here — it *persists after diagnosis*. His image is
  the moon at the horizon: the astronomer knows perfectly well that it is no larger
  there than overhead, and cannot prevent it from appearing larger. What diagnosis buys
  is not the disappearance of the appearance. It is only that one is no longer deceived
  by it.
]

#section("The 61-digit integer")

Here is the present epoch, at full precision:

#terminal(caption: "ucal now — ticks")[
```
8070205189128471254993117657693008777530466139316558837890625
```
]

Every reader who sees that number will read it, at least for a moment, as a fact about
being.

Not because they are careless. Because that is what a very precise number *looks like*.
Sixty-one digits with no rounding anywhere carries an unmistakable impression of having
been *found* — of corresponding to something, out there, that is exactly this many
ticks old. The impression arrives before any reasoning does and does not wait for
permission.

#claim("interpretation")[
  It is false. Chapter 3 said why in one line: the count is exact and the correspondence
  between tick zero and any physical event is a separate question with a separate
  answer.

  The number is exactly right about how many ticks have elapsed since a stipulated
  origin. It says nothing whatever about how long the universe has existed, because the
  origin was declared rather than discovered, and every digit of precision is precision
  *about the count*, not about the world.

  A reader who has finished Part IV knows this. They will still see the number as a
  fact about being, for the same reason the astronomer still sees the moon.
]

#section("What `UCAL-E0025` actually does")

Chapter 12 presented the compile-fail tests as the book's central exhibit, and they are.
It is worth being precise, at the end, about the scope of what they accomplish.

They stop the claim entering the arithmetic. `BIG_BANG_CLAIM` cannot be added to an
`Instant`, cannot be converted to a `Delta`, cannot reach an operand position by any
route the type system permits. Three programs that try are required to fail to build,
and they do.

That is a real thing to have stopped. Chapter 29 argued why: it is a discipline that
does not depend on the programmer having read anything, agreeing with anything, or
remembering anything.

#claim("interpretation")[
  And it is not the same as stopping the illusion.

  The type governs what may be *computed*. It has no reach at all over what may be
  *read*. A person looking at sixty-one digits and forming an impression about the age
  of the universe is not performing an operation the compiler can refuse. They are
  perceiving, and there is no type signature between a number and its reader.

  So the honest statement of what chapter 12 achieved is narrower than the chapter made
  it sound: `UCAL-E0025` *contains* the illusion. It does not cure it.

  It ensures that the false impression cannot propagate into a computed value — cannot
  become a timestamp that inherits an uncertainty it should not have, cannot become an
  answer someone downstream trusts. The impression itself survives intact, in every
  reader, permanently.
]

#section("Florensky, one last time")

Chapter 13 said the rule protects the arithmetic and does not protect the reading, and
promised the last chapter would be about the residue. This is it.

Florensky's §9 is a case of exactly this failure. He had the formal artifact — the
imaginary value of an expression outside its domain — and he read it as designating a
place. Chapter 25 established that this was not carelessness: on Losev's account, which
Florensky publicly defended, a formal structure *can* be constitutive of what it
describes, and so reading a mathematical artifact as designating something real is the
expected case rather than an error.

#claim("interpretation")[
  Which means the rule this project holds — that formal artifacts are not physical facts
  — is not a safeguard against a mistake. It is a *position*, contested by serious people,
  adopted here without argument, as chapter 25 recorded.

  And the position does not protect its holder from the illusion. It only forbids acting
  on it.

  I would like to be able to write that this project is safe from Florensky's error
  because it has the rule he lacked. It is not. The rule keeps the error out of the
  computation. The author of this book looks at a 61-digit integer and feels what
  everyone feels, and has felt it repeatedly while writing, and the type system was no
  help at all in that.
]

#section("What a specification can do")

So the book ends on a boundary rather than a conclusion.

#claim("interpretation")[
  A specification can say what a system computes with. It can enforce that boundary
  absolutely, at compile time, against everyone.

  It cannot say what a number means to someone reading it. There is no mechanism for
  that, there has never been one, and the two-and-a-half centuries since the First
  Critique have not produced one — Kant did not expect them to, which is the whole point
  of calling the illusion natural and unavoidable rather than remediable.

  `UCAL-E0025` is what a specification can do, done as thoroughly as the medium permits.
  It refuses to compute with a claim the artifact cannot help but suggest.

  That is the only thing a specification can do about a transcendental illusion, and
  doing it is not nothing. The astronomer who knows the moon is not larger still cannot
  see it correctly — but he does not adjust his instruments for it, and his tables are
  right.
]

#v(8mm)
#align(center, line(length: 30%, stroke: 0.5pt + ink_rule))
#v(6mm)

#align(center, block(width: 76%)[
  #set par(justify: false)
  #align(center, text(size: 10.5pt, style: "italic", fill: ink_gray)[
    Tick zero is a stipulated reference point, conventionally identified with the
    FLRW $t arrow.r 0$ limit. It is not a measurement and not an observed event.
  ])
])

#v(4mm)
#align(center, text(size: 9pt, fill: ink_faint, tracking: 1pt,
  "— the datum statement, printed by " + raw("ucal datum") + " on every run"))

#recap((
  [Transcendental illusion is natural, unavoidable, and persists after diagnosis. The astronomer knows the moon is not larger at the horizon and sees it larger anyway.],
  [Every reader of a 61-digit integer will read it as a fact about being, because that is what a very precise number looks like. The impression arrives before reasoning and does not wait for permission.],
  [`UCAL-E0025` stops the claim entering the arithmetic. It has no reach over what may be read — there is no type signature between a number and its reader.],
  [So it *contains* the illusion rather than curing it: the false impression cannot propagate into a computed value, and survives intact in every reader.],
  [The rule against reading formal artifacts as physical facts is a contested position, not a safeguard — and it does not protect its holder from the illusion, only from acting on it.],
  [That is the only thing a specification can do about a transcendental illusion, and doing it is not nothing.],
))
