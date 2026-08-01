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
#set par(leading: 0.62em, justify: true)

#show raw.where(block: true): it => block(
  fill: rgb("#f3eee4"), stroke: 0.4pt + ink_rule, inset: 5pt, radius: 2pt,
  width: 100%, breakable: false, text(font: mono_family, size: 7.4pt, it))
#show raw.where(block: false): it => box(
  fill: rgb("#f3eee4"), inset: (x: 2pt), outset: (y: 1.4pt), radius: 1pt,
  text(font: mono_family, size: 8pt, it))

#let head(t) = block(sticky: true, above: 4mm, below: 1.6mm,
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
billion.

The number is correct. It carries a passenger.

Not a complaint about cosmology: Julian years are stable, agreed and
unambiguous, which is what a unit should be. The complaint is that *provenance
leaks into arithmetic*. Compute in Earth years and Earth's orbital period sits
inside every intermediate value.

#head("What was built")
`ucal` is six Rust crates. Time is an unsigned integer count of Planck-time
units — *ticks*, $5.391 times 10^(-44)$ s — since a stipulated datum.

Ticks group into tiers, the powers $5^(5k)$, each exactly five base-5 digits.
The reference rung is the *beat*, $5^60$ ticks ≈ 46.762 ms, which the
specification calls the universe second: human-noticeable, with no Earth content.
Base five because $5^5 = 3125$ is five base-5 digits — that is the whole reason.

A timestamp is the tick count written in base 5 and grouped in fives, so
truncation *is* rounding, prefix comparison *is* chronological comparison, and
writing every digit pinpoints one tick.

The cost is that nothing on the ladder is near anything you know. One second is
21.385061835 beats, and the two share a common measure only at the tick.

#head("Three engineering commitments")
*Unsigned.* Nothing precedes the datum. A result that would be earlier is
`UCAL-E0020` — an error, not a negative number. The refusal is the answer.

*No floating point, anywhere.* Not in a signature, a field, an intermediate, or
the rendering path; a lint fails the build on any float token. Cosmology is
therefore certified interval quadrature over exact rationals — orders of
magnitude slower, and it returns an *enclosure* with a proof rather than an
estimate with a guess.

*One declared boundary.* Earth enters through a single exact constant,
`SECOND` = 18 548 584 399 861 × $10^30$ ticks. Conversion in is multiplication
and never rounds; conversion out is division and is the only place a rounding
mode is chosen. The dependency is not eliminated — that is not available to
anyone — it is *localised* to somewhere you can point at and argue about.

#head("The problem with zero")
A count needs a zero, and the obvious one is not available.

The published age is 13.787 ± 0.020 Gyr. That uncertainty is 58 digits of ticks,
0.145% of the whole span. Define the datum *as* the measured age and every
timestamp inherits the wobble, making the exact arithmetic theatre.

So the datum is *stipulated*: declared, not discovered. And the physical claim
about it is kept — cited, with its exact magnitude — in a type that has *no
arithmetic operations at all*. Three tests exist whose job is to *fail to build*
if anyone ever computes with it.

#colbreak()

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
