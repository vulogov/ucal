// UCAL_SHORT_INTRO — two pages.
//
//   typst compile Documentation/UCAL_SHORT_INTRO.typ
//
// The shortest honest account: why the project exists, what it does, what it
// borrows, what it refuses. Two pages is not room for argument, so nothing here
// is argued — every claim is stated with the number that supports it and a
// pointer to where it is defended.
//
// Longer: UCAL_INTRO.typ (29 pp). Full: LIFE_UNIVERSE_AND_GOD (251 pp).

#import "LIFE_UNIVERSE_AND_GOD/design.typ": (
  ink_black, ink_gray, ink_faint, ink_rule, ink_accent, ink_smoke,
  body_family, mono_family,
)

#set document(title: "Attempt to Measure the Unmeasurable",
  author: "Vladimir Ulogov")
// The colophon is a page footer rather than trailing content: as flowing
// content after a two-column block it was pushed onto a third page whenever the
// columns filled page two, which is exactly what happened.
#set page(paper: "a4", margin: (x: 17mm, y: 15mm, bottom: 20mm), fill: white,
  footer: context {
    if here().page() == counter(page).final().first() {
      line(length: 100%, stroke: 0.4pt + rgb("#c6c0b5"))
      v(1.2mm)
      grid(columns: (1fr, auto), align: horizon,
        text(size: 7.6pt, style: "italic", fill: rgb("#5d5d5d"))[
          Tick zero is a stipulated reference point, conventionally identified
          with the FLRW $t arrow.r 0$ limit. It is not a measurement and not an
          observed event. #h(0.6em) — printed by #raw("ucal datum") on every run.
        ],
        text(size: 7.6pt, fill: rgb("#9a9a9a"))[Vladimir Ulogov · MPL-2.0])
    }
  })
#set text(font: body_family, size: 9pt, fill: ink_black, lang: "en")
#set par(leading: 0.62em, spacing: 1.15em, justify: true)

#show raw.where(block: true): it => block(
  fill: rgb("#f3eee4"), stroke: 0.4pt + ink_rule, inset: 5pt, radius: 2pt,
  width: 100%, breakable: false, text(font: mono_family, size: 7.4pt, it))
#show raw.where(block: false): it => box(
  fill: rgb("#f3eee4"), inset: (x: 2pt), outset: (y: 1.4pt), radius: 1pt,
  text(font: mono_family, size: 8pt, it))

// A heading at 11pt over 9pt text needs real air beneath it, or the descenders
// of the heading sit on the ascenders of the first line.
#let head(t) = block(sticky: true, above: 5mm, below: 2.8mm,
  text(size: 11pt, weight: "bold", fill: ink_black, t))
#let note(t) = block(width: 100%, inset: (left: 7pt),
  stroke: (left: 1.6pt + ink_accent),
  text(size: 8.4pt, fill: ink_gray, t))

// ── masthead ────────────────────────────────────────────────────────
#grid(columns: (26mm, 1fr), gutter: 6mm, align: horizon,
  image("LIFE_UNIVERSE_AND_GOD/assets/images/dial-void.png", width: 100%),
  [
    #text(size: 8pt, tracking: 2.6pt, fill: ink_smoke,
      upper("Counting from the first tick"))
    #v(2mm)
    #text(size: 19pt, weight: "bold", fill: ink_black,
      "Attempt to Measure the Unmeasurable")
    #v(1.6mm)
    #text(size: 10pt, style: "italic", fill: ink_smoke,
      "ucal — the Universe Calendar, in two pages")
  ]
)
#v(2mm)
#line(length: 100%, stroke: 0.6pt + ink_accent)
#v(3mm)

#columns(2, gutter: 7mm)[

#head("The irritation")
*Recombination occurred about 380,000 years after the Big Bang.* A good sentence,
and its units are a problem. A year is the time Earth takes to circle the Sun —
definitionally, not approximately. So an event 13.8 billion years old is
described in units defined by a planet that would not exist for another nine
billion. The number is correct; it carries a passenger.

Not a complaint about cosmology. The complaint is that *provenance leaks into
arithmetic*: compute in Earth years and Earth's orbital period sits inside every
intermediate value.

#v(1.5mm)
#note[
  Three things are declared. Everything else in the system is *computed* from
  them — the notation, the ladder, the domain, the calendars, the cosmology.
  They are worth the space.
]

#head("1 — The tick")
The Planck time, $t_P = sqrt(planck G \/ c^5) approx 5.391247 times 10^(-44)$ s.
It is the one combination of the gravitational constant, the reduced Planck
constant and the speed of light with the dimension of time — so it is built from
the bounds on gravitation, quantum action and causality, and from nothing else.
It does not know about Earth, or planets, or matter being organised at all.

Every quantity in the system is an unsigned integer count of ticks. Nothing else
is primitive — not the second, not the beat, not the day.

*It is not a quantum of time.* It is the resolution floor of an instrument, not a
structure found in the world. The system cannot represent a shorter duration;
that is a fact about the system. Asserting otherwise would take a side in a
twenty-four-century argument as a side effect of choosing an integer type.

*The one concession.* The tick's length *in seconds* is fixed by convention
against SI, because stating it at all needs a unit to state it in. That concedes
metrology and nothing else: ticks are counted, never converted.

#head("2 — The beat")
$5^60$ ticks — 867 361 737 988 403 547 205 962 240 695 953 369 140 625 of them,
about 46.762 ms. The specification calls it the *universe second*.

Two choices, each with one reason. *Base five*, because $5^5 = 3125$ is a number
of exactly five base-5 digits, so a tier is a clean five-digit group. *Exponent
60*, because it puts the reference rung at the scale a human can notice a
duration — the one concession to the reader in the whole design, and it changes
no arithmetic.

Everything above and below is $5^(5k)$: deep, drift, span, sweep, arc, beat,
flicker. A uniform ladder, no ragged fields at either end.

#note[
  The beat is not a second in disguise. One second is 21.385061835 beats, and no
  rescaling would make it whole: `SECOND` carries thirty factors of five and
  `BEAT` carries sixty, so the two share a common measure only *at the tick*.

  That is the whole reason the tick is primitive rather than either of them.
]

#head("3 — The datum")
Tick 0. *Stipulated* — declared, not measured, and not an observed event.

It cannot be measured, and the reason is arithmetic before it is philosophy. The
published age is 13.787 ± 0.020 Gyr; that uncertainty is a fifty-eight-digit
number of ticks, 0.145% of the whole span. Define the datum *as* the measured age
and every timestamp inherits the wobble, making the exact arithmetic theatre.

So it is declared: `ORIGIN_OFFSET` = 9 304 311 741 502 590 385 beats, a whole
number of them. The published age rounded to a whole beat — and the system prints
what the rounding discarded, −0.017190364 s, rather than absorbing it.

Two things follow. The domain is *unsigned*: it begins at the datum, and an
earlier instant is not representable — `UCAL-E0020`, an error rather than a
negative number. A limit on what can be *dated*, not a claim about what exists;
the specification keeps the origin claim in a *signed* window precisely because
the limit may lie earlier. And the
physical claim about where the origin falls is kept separately — cited, with its
exact magnitude, in a type with *no arithmetic operations at all*. Three tests
exist whose job is to *fail to build* if anyone computes with it.

#colbreak()

#head("What is computed from them")
Nothing below is a further decision. Each follows from the three above.

*The notation.* A timestamp is the tick count written in base 5 and grouped in
fives. So truncation *is* rounding, prefix comparison *is* chronological
comparison, and writing every digit pinpoints one tick — no separate rounding
step, no scaling, no drift between the coarse view and the fine one.

*The domain.* 512 bits, reaching $2.29 times 10^103$ years — past proton decay
and black-hole evaporation. The present epoch is $6 times 10^(-94)$ of it. The
width is fixed so that it never has to change: the canonical binary form is 64
bytes because the domain is 512 bits.

*The bridge.* Earth enters through one exact constant,
`SECOND` = 18 548 584 399 861 × $10^30$ ticks. In is multiplication and never
rounds; out is division and is the only place a rounding mode is chosen. The
dependency is not eliminated — that is not available to anyone — it is
*localised* to somewhere you can point at.

*Calendars.* A body's periods enter as exact rationals of ticks; intercalation is
*derived* by continued fraction, never declared.

*Cosmology.* No floating point anywhere, so ages are certified interval
quadrature over exact rationals: two numbers and a proof the answer lies between
them, rather than one number and a guess.

#head("The philosophical background")
The line between what a measurement *establishes* and what it merely *points at*
is old, and it does not stay where you put it. Kant was bluntest: the illusion
that erodes it is natural, unavoidable, and *persists after diagnosis* — the
astronomer knows the moon is no larger at the horizon and goes on seeing it so.

The usual remedy is vigilance, which lasts as long as attention does. In 1922
Florensky — who had the distinction available in his own tradition, fifteen
centuries deep — computed a radius from the Lorentz factor outside its domain of
validity and identified the resulting surface with the Empyrean. One move, and
nothing in his method forbade it.

Kant's First Antinomy supplies the reason the origin cannot be measured at all:
both *the world has a beginning* and *it does not* are demonstrable, and both
fail, because both assume a completed totality about which the question has an
answer waiting. Stipulating is not a retreat from a better method. There is no
better method.

Cusanus, 1440: no proportion holds between finite and infinite, so precision does
not constitute approach.

#head("The theological background")
Basil of Caesarea, fourth century: the beginning of a road is not the road. A
beginning is not a member of the series it begins — which is the datum's content,
with no arithmetic.

Gregory of Nyssa makes *διάστημα*, interval, the mark of createdness: everything
created has it, and what lies beyond has none. Every quantity in this system is
an interval, so the title stops being a paradox and becomes a statement about
categories.

The Hebrew calendar's epoch is *molad tohu*, "the new moon of chaos" —
computational, placed before the event it anchors, and named for its own
emptiness. This project spent four chapters explaining that its epoch is
stipulated; that one puts it in the name. Its *ḥelek* was chosen for
divisibility exactly as `SECOND` was.

And a validity window — assert a parameter where it was measured, warn beyond —
is al-Ghazālī's *ʿāda* with the theology removed.

#note[
  None of this argues that any tradition is correct. The decisions have older,
  better-named precedents reached for other purposes; reporting that is an
  argument that the decisions are not eccentric, and nothing more.
]

#head("What came out of it")
Give the mechanism nothing but Earth's rotation and orbit and its first answer is
`1/4` — the Julian calendar, with no knowledge of Rome. `97/400` never appears:
the Gregorian rule is not a convergent at any depth, and `31/128` is *124× more
accurate* with a denominator three times smaller.

The specification claimed twice, in print, that the mechanism reproduces both. It
reproduces one. That correction is published, and a build-time assertion now
stops it being reversed.

Two more fell out. The Metonic cycle — 235 months in 19 years, known to Babylon —
appears from two numbers unaided. And Mars has *no month*: neither moon is one,
and the mechanism returns nothing rather than inventing one, because *month-like*
is an Earth predicate.

#head("What is claimed, and what is not")
#note[
  A measuring instrument may legitimately point at what it cannot describe,
  provided it declares that it is only pointing — and that declaration can be
  enforced mechanically rather than left to the author's discipline.
]
#v(1mm)
The first clause is Cusanus'. The second is the contribution: a comment addresses
a reader who is paying attention, a runtime check addresses a program that
reaches a line, and a *type* addresses everyone who ever writes against the
library — including someone who disagrees. They are not persuaded. They are
refused, in under a second.

Not claimed: that time began; that the Big Bang happened at tick 0; that the tick
is time's smallest unit; that base 5 means anything; that any tradition is right;
or that the system is *useful* — no task you have needs a Planck-tick count, and
that is the second of three facts rather than an apology.

#v(1.5mm)
#line(length: 100%, stroke: 0.4pt + ink_rule)
#v(1.5mm)
#text(size: 8pt, fill: ink_gray)[
  *Where next.* `cargo install ucal`, or `cargo add ucal-core`. The normative
  specification is `spec/UCAL-1.1.md`; the twenty-four rules and what enforces
  each are in `spec/RULES.md`. A 29-page account is `UCAL_INTRO`. The full
  argument — nine traditions, six samples, and a chapter on what they failed to
  establish — is *Life, the Universe, and God*, 251 pages.
]
]
