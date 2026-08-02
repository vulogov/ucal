// UCAL_INTRO — a short introduction to the Universe Calendar.
//
// Compile with:
//   typst compile Documentation/UCAL_INTRO.typ
//
// Twenty-odd pages, standing on its own. It borrows the book's design tokens
// and its voice, and uses none of its apparatus: no marked claim blocks, no
// conflict sections, no recaps. Those exist in the book because 251 pages of
// argument need them. An introduction needs to be clear.
//
// Every figure here is checkable against the source tree. Where a number
// appears, it came from running the thing.

#import "LIFE_UNIVERSE_AND_GOD/design.typ": (
  ink_black, ink_gray, ink_faint, ink_rule, ink_accent, ink_smoke, ink_paper,
  ink_term, ink_code_bg, ink_call_bg, ink_term_bg,
  body_family, mono_family,
  section, subsection, term, callout, terminal, figcap,
)

#let doc_title    = "A Software Engineer's Attempt to Measure Time in the Universe"
#let doc_subtitle = "An introduction to ucal"
#let doc_author   = "Vladimir Ulogov"

// ── Numbered part heading, lighter than the book's chapter opening ──
#let counter_part = counter("part")
#let part(title) = {
  pagebreak(weak: true)
  counter_part.step()
  hide(heading(level: 1, numbering: none, outlined: true, bookmarked: true, title))
  v(6mm)
  block(width: 100%, {
    text(font: body_family, size: 9pt, tracking: 2.5pt, fill: ink_gray,
      context [PART #counter_part.display()])
    v(2mm)
    text(font: body_family, size: 22pt, weight: "bold", fill: ink_black, title)
    v(3mm)
    line(length: 100%, stroke: 0.5pt + ink_rule)
  })
  v(6mm)
}

#let plate(path, width: 72%, cap) = {
  block(breakable: false, width: 100%, {
    v(3mm)
    align(center, image(path, width: width))
    v(1.5mm)
    align(center, block(width: 84%, breakable: false, {
      set par(justify: false)
      text(font: body_family, size: 8.5pt, style: "italic", fill: ink_gray, cap)
    }))
    v(2mm)
  })
}

// ── Document ────────────────────────────────────────────────────────
#set document(title: doc_title, author: doc_author)
#set text(font: body_family, size: 11pt, fill: ink_black, lang: "en")
#set par(leading: 0.72em, justify: true, first-line-indent: 1em)

#show raw.where(block: true): it => block(
  fill: ink_code_bg, stroke: 0.5pt + ink_rule, inset: 7pt, radius: 2pt,
  width: 100%, breakable: false,
  text(font: mono_family, size: 8.5pt, it))
#show raw.where(block: false): it => box(
  fill: ink_code_bg, inset: (x: 2pt, y: 0pt), outset: (y: 1.5pt), radius: 1pt,
  text(font: mono_family, size: 9.5pt, it))

// Title page
#set page(paper: "iso-b5", margin: 0pt, numbering: none, header: none,
  fill: ink_paper)
#block(width: 100%, height: 100%)[
  #place(top + left, dx: 12mm, dy: 12mm,
    rect(width: 100% - 24mm, height: 100% - 24mm, stroke: 0.8pt + ink_accent))
  #place(top + center, dy: 26mm,
    image("LIFE_UNIVERSE_AND_GOD/assets/images/dial-void.png", width: 52mm))
  #place(top + center, dy: 96mm, block(width: 74%)[
    #set par(justify: false)
    #align(center)[
      #text(font: body_family, size: 10pt, tracking: 3.5pt, fill: ink_smoke,
        upper("Counting from the first tick"))
      #v(9mm)
      #text(font: body_family, size: 20pt, weight: "bold", fill: ink_black,
        doc_title)
      #v(5mm)
      #line(length: 46%, stroke: 0.6pt + ink_accent)
      #v(5mm)
      #text(font: body_family, size: 12pt, style: "italic", fill: ink_smoke,
        doc_subtitle)
    ]
  ])
  #place(bottom + center, dy: -26mm, align(center)[
    #text(font: body_family, size: 10pt, fill: ink_smoke, doc_author)
    #v(2mm)
    #text(font: body_family, size: 8.5pt, fill: ink_smoke,
      "An introduction. The full argument is Life, the Universe, and God.")
  ])
]
#pagebreak()

#set page(paper: "iso-b5",
  margin: (inside: 24mm, outside: 19mm, top: 25mm, bottom: 22mm),
  fill: white, numbering: "1", number-align: center,
  header: context {
    if counter(page).get().first() > 1 {
      align(center, text(font: body_family, size: 8pt, fill: ink_faint,
        tracking: 1.5pt, upper("A Software Engineer's Attempt to Measure Time")))
    }
  })
#counter(page).update(1)

// ── 1 ───────────────────────────────────────────────────────────────
#part("The irritation")

Read almost any account of the early universe and you will find a sentence like
this one:

#align(center, block(width: 82%, breakable: false)[
  #set par(justify: false)
  #v(2mm)
  #text(size: 11.5pt, style: "italic")[
    Recombination occurred about 380,000 years after the Big Bang.
  ]
  #v(2mm)
])

It is a good sentence. It is doing real work. And if you stop and ask what its
units are, something uncomfortable happens.

A year is the time Earth takes to go around the Sun. Not approximately —
definitionally. The Julian year used in astronomy is exactly 365.25 days of
exactly 86,400 seconds, and those numbers are what they are because of how fast
one particular rock spins and how long it takes to circle one particular star.

So *380,000 years after the Big Bang* means 380,000 × 365.25 × 86,400 seconds,
which means it is a quantity expressed in units defined by the rotation and orbit
of a planet that would not exist for another nine billion years.

The number is correct. It carries a passenger.

#section("What the complaint is, precisely")

The problem is not that the units are arbitrary. All units are arbitrary. The
metre was a bar in a vault, then a wavelength, then a fraction of a light-second,
and none of that makes it a bad metre.

The problem is *provenance leaking into arithmetic*. When you compute with Earth
years, Earth's orbital period sits inside every intermediate value. Usually that
costs nothing. Occasionally it costs a rounding you did not intend, because
365.25 days is not a whole number of anything and the conversions do not close.
And structurally it means the description of an event 13.8 billion years old is
phrased in terms of a body that had no bearing on it.

#callout(label: "Not a criticism of cosmology")[
  Astronomers use Julian years because they are stable, agreed and unambiguous,
  which is exactly what a unit should be. The complaint here is narrow, and it is
  aesthetic before it is technical: a quantity ought to be expressible in units
  that do not smuggle in a planet.
]

#section("What would have to be true instead")

Suppose you wanted a time system with no Earth content in its arithmetic. Four
requirements fall out immediately.

/ A physical unit: #sym.dash.em something built from constants of nature rather
  than from a rotation.
/ A non-calendrical origin: #sym.dash.em because every calendar epoch is
  somebody's civil history.
/ A notation where writing fewer digits means saying *less precisely*: #sym.dash.em
  rather than meaning zero. At these magnitudes you are constantly quoting
  numbers you do not know to full precision, and a system that silently pads them
  is lying on your behalf.
/ One declared boundary with Earth: #sym.dash.em because the contact has to happen
  somewhere. The question is whether it happens *in the arithmetic* or at a point
  you can put your finger on.

That last question turned out to be the whole design. `ucal` is the answer to
those four requirements, and the rest of this document is what it cost.

// ── 2 ───────────────────────────────────────────────────────────────
#part("What was built")

#term("Tick")[
  The Planck time, $t_P = sqrt(planck G \/ c^5) approx 5.391 times 10^(-44)$
  seconds. The atomic unit: the smallest quantity representable, and the unit in
  which every other quantity is counted.
]

The Planck time is composed from three constants — the gravitational constant,
the reduced Planck constant, and the speed of light. Those bound gravitation,
quantum action and causality respectively, and there is exactly one combination
of them with the dimension of time.

That is the appeal. The tick is not defined by any body's motion. It does not
know about Earth, or the Solar System, or matter being organised into planets at
all.

#callout(label: "The tick is not a quantum of time")[
  It is the resolution floor of an *instrument*, not a structure discovered in
  the world. Nothing here asserts that time comes in discrete lumps or that there
  is a smallest possible interval. The system cannot represent a shorter
  duration; that is a fact about the system.

  The distinction matters because asserting otherwise would silently take a side
  in an argument running since Aristotle — and taking a metaphysical position as
  a side effect of choosing an integer type is not a respectable way to hold one.
]

#section("The ladder")

A tick is far too small to think in. The age of the universe is about
$8 times 10^(60)$ of them, and nobody reads a 61-digit number.

So ticks are grouped into *tiers*: the powers $5^(5k)$, each exactly five base-5
digits — 3125 of the tier below. The reference rung is the *beat*, $5^60$ ticks,
about 46.762 milliseconds, which the specification calls the *universe second*: a
unit of human-noticeable size with no Earth content whatever.

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 4.5pt),
    align: (center, left, left, right),
    [*tier*], [*name*], [*exponent*], [*≈*],
    [T5], [deep], [$5^85$], [441.6 Myr],
    [T4], [drift], [$5^80$], [141.3 kyr],
    [T3], [span], [$5^75$], [45.2 yr],
    [T2], [sweep], [$5^70$], [5.285 d],
    [T1], [arc], [$5^65$], [146.1 s],
    [T0], [*beat*], [$5^60$], [46.762 ms],
    [T−1], [flicker], [$5^55$], [14.96 µs],
  )
]
#v(2mm)

Why base five? Because $5^5 = 3125$ is a number of exactly five base-5 digits.
That is the entire reason, and it is worth saying plainly before anyone reaches
the later parts of this document and starts looking for significance in the five.

#subsection("What the ladder buys")

A timestamp is the tick count written in base 5 and grouped in fives. Three
things follow, and all three are consequences of that one fact.

*Truncation is rounding.* Write fewer groups and you have said the same thing
less precisely. The digits you dropped are exactly the precision you gave up.

*Prefix comparison is chronological comparison.* Positional notation is monotone,
so sorting the text sorts the times.

*Writing all the digits pinpoints one tick.* The coarse view and the fine view
are the same integer read at different widths, so there is no accumulated
conversion error between them.

#subsection("What the ladder costs")

Nothing on it is near anything you know. A second is 21.385 beats — not a whole
number, and not close to one. An hour is 24.6 arcs. A day is 0.189 sweeps.

The two seconds do not divide, and the system says so on its own output: `BEAT`
carries $5^60$ while `SECOND` carries only $5^30$, so they share a common measure
only at the tick. That is exactly why the tick is primitive rather than either of
them.

If you want units with no planetary content, you do not get to keep the hour. The
hour *is* planetary content.

#section("The domain")

The tick count is a 512-bit unsigned integer, and the ceiling is about
$2.29 times 10^103$ years.

To scale that: proton decay, if it happens, is expected around $10^34$ years; the
last black holes evaporate around $10^100$. The domain outlasts both. The present
epoch sits at $6.0 times 10^(-94)$ of the range — the counter is, for all
practical purposes, still at zero.

#plate("LIFE_UNIVERSE_AND_GOD/assets/images/scale-plate.png", width: 30%)[
  The domain as a scale. From a burst at the Planck tick, through an atom's
  vibration, a heartbeat, the day, a human life, recorded history, the
  stratigraphic record and a galaxy's turning, to a single point of light in
  emptiness. Logarithmic; the present epoch sits nowhere near the top.
]

The width was not chosen to impress. It was chosen so that the width never has to
*change*: the canonical binary encoding is 64 bytes because the domain is 512
bits, and widening it later would invalidate every stored timestamp in existence.
Sixty-four bytes, paid once, against a class of migration software normally pays
repeatedly.

#section("Three refusals")

Time here is *unsigned*. There is no tick −1, and a request for an instant before
the datum does not return a negative number — it returns an error. There is a
difference between answering a question with a negative value and refusing the
question, and the system takes the second option deliberately.

That refusal is the first of three, and they are the same move at three scales:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 5pt),
    [*code*], [*rather than*],
    [`UCAL-E0020`], [returning a negative tick count before the datum],
    [`UCAL-E0031`], [truncating an identifier beyond its declared range],
    [`UCAL-E0043`], [rounding an input finer than the bridge constant permits],
  )
]
#v(2mm)

A system that answered all three would be more convenient, and would be lying in
three different ways.

#section("No floating point")

There is no floating-point value in any shipped crate of this workspace. Not in a
signature, a field, a constant, an intermediate, or the rendering path that
produces the human-readable output. A lint scans the source and fails the build
on any float token.

The cost is concentrated in one place. Computing the age of the universe at a
given redshift means evaluating an integral; in any normal library that is a call
to a quadrature routine over `f64`. Here there is none, so it is certified
interval quadrature over exact rationals — thousands of panels, each bounded
above and below, with directed integer square roots underneath. It is slower by
orders of magnitude.

What it buys is an *enclosure* rather than an estimate. The float routine returns
a number and a well-informed guess about accumulated error. The interval routine
returns two numbers and a proof that the true value lies between them.

#callout(label: "Two widths, never merged")[
  Every cosmological result carries the quadrature's error and the model's own
  measured uncertainty *separately*.

  At recombination the quadrature width is about 251 years and the parameter
  width about 10,900. Merged into one number the answer reads "about eleven
  thousand years of uncertainty", and the fact that forty-three forty-fourths of
  it comes from the measurement rather than the computation becomes invisible.

  That distinction is what tells you more computing power would buy nothing here.
]

#section("The bridge")

Earth enters the system through exactly one number.

#terminal(caption: "the bridge constant")[
```
SECOND = 18 548 584 399 861 000 000 000 000 000 000 000 000 000 000 ticks
```
]

It is an exact integer, declared rather than computed, and it is the only place in
the workspace where a foreign unit is named at all — a lint fails the build if any
identifier in the core crate mentions one elsewhere.

Converting *into* absolute time is multiplication by an integer, which is exact:
a whole number of seconds becomes a whole number of ticks with no rounding, ever.
Converting *out* is division, which is where rounding lives — and so the rendering
path is the only place a rounding mode is chosen, always explicitly.

The design does not eliminate the Earth dependency; that is not available to
anyone. What it does is *localise* it. There is one constant, in one declared
place, with a lint that fails the build if a second appears. The arithmetic is
clean; the boundary is dirty; and the boundary is visible.

A dependency you can point at can be argued about. One dissolved into the
arithmetic cannot.

// ── 3 ───────────────────────────────────────────────────────────────
#part("The problem with zero")

A count needs somewhere to start counting from. If time is a count since some
zero, the zero has to be somewhere — and the obvious place, the beginning, is not
available.

#section("Why it cannot be measured")

Four reasons, and they compound.

*First: exactness cannot come from measurement.* The published age of the universe
is 13.787 billion years, plus or minus 0.020 billion. That uncertainty, converted
to ticks, is a number with fifty-eight digits — about 0.145% of the whole span
from the datum to now. If the datum were defined *as* the measured age, every
timestamp would inherit that wobble, and the exact integer arithmetic would be
theatre over a guess.

*Second: the limit is not an event.* What is measured is the *age* — how long the
universe has been expanding under a model — and the model is known to stop
describing anything real well before you reach the limit.

*Third: the extrapolation is model-dependent.* The figure 13.787 is not read off
an instrument; it falls out of a parameter fit. Change the model and it changes.
A datum defined from it would shift with each data release.

*Fourth — and this is Kant's — the question may be malformed.* More on that in
Part IV.

#section("What was done instead")

Three moves. Stipulate the datum. Declare the physical claim separately. Make the
claim impossible to compute with.

The first two are ordinary good practice. The third is why this project turned
into a book.

`BIG_BANG_CLAIM` is a value whose type has *two fields and nothing else*: no
arithmetic operators of any kind, no conversion to any computable type, no method
returning one. You can read its bounds, render it, print its citation. You cannot
do arithmetic with it.

An absence is hard to test — you cannot assert that a method does not exist,
because naming it would fail to compile. So the project uses *compile-fail tests*:
programs required to fail to build, checked on every run.

#terminal(caption: "tests/compile_fail/signed_window_as_operand.rs")[
```rust
use ucal_core::{Instant, Profile, UC1};

fn main() {
    let t: Instant<UC1> = Instant::zero();
    let claim = UC1::big_bang_claim();
    // A SignedWindow is not a Delta and must never become one.
    let _ = t.checked_add(&claim);
}
```
]

That is a person trying, in the most natural way available, to use the uncertainty
in the age of the universe as a number. It fails to compile. The test suite passes
precisely because it does.

#section("The chain that re-executes")

Declaring the claim separately would be worth little if the declaration were
prose. It is data, and it runs.

#terminal(caption: "ucal datum — the provenance chain")[
```
datum_provenance:
  input     13.787 Gyr +/- 0.020 Gyr (age_of_universe)
  citation  Planck 2018 results VI, A&A 641, A6 (2020)
  chain:
    AGE_s     = 13 787 000 000 x 31 557 600
              = 435 084 631 200 000 000 s        (exact)
    AGE_ticks = AGE_s x SECOND                   (exact)
    beats     = round_half_even(AGE_ticks / BEAT)
              = 9 304 311 741 502 590 385
    ORIGIN_OFFSET
              = beats x BEAT
  residual_rendered  -0.017190364 s
```
]

Every step is there: the cited input, each exact multiplication, the rounding to a
whole beat, and — the part that matters most — *the residual the rounding
discarded*. Seventeen milliseconds. The datum is not the published age; it is the
published age rounded, and the system prints the difference rather than absorbing
it.

A citation says where a number came from. This says where it came from *and what
happened to it on the way in*.

// ── 4 ───────────────────────────────────────────────────────────────
#part("The philosophical background")

This part is short on purpose. The full treatment is nine chapters of the book;
what follows is the minimum needed to see that the design decisions above are not
novelties.

#section("The distinction, and why it does not stay put")

There is a line between what a measurement *establishes* and what it merely
*points at*. Everyone agrees it exists. Aristotle has a version, Basil of Caesarea
has a version, Kant spent a large part of the first *Critique* on it — and every
one of them observed that it does not stay where you put it.

Kant was the bluntest. Transcendental illusion, he says, is *natural and
unavoidable*, and it *persists after diagnosis*. His image is the moon at the
horizon: the astronomer knows perfectly well it is no larger there than overhead,
and cannot stop seeing it larger.

#plate("LIFE_UNIVERSE_AND_GOD/assets/images/kants-moon.png", width: 74%)[
  The same moon, twice, drawn at the same diameter — measured on the plate the two
  discs differ by under one per cent. Everything else is the surrounding.
]

The usual remedy for this is vigilance: argue carefully, mark the boundary, and
hope the next reader is paying attention. It works for as long as attention lasts.

#section("Three borrowings")

*Aristotle*, *Physics* IV: time is "the number of motion with respect to before
and after" — not motion itself, and not a container, but the countable aspect of
change. And in IV.14 he asks whether time could exist with no soul to count it,
and answers carefully that there would be the substrate but not time *as number*,
since number requires a numberer.

That is uncomfortable for this project rather than supportive. The datum is where
a counter decided to start, and Rule Q concedes that rather than answering it.

*Kant's First Antinomy* supplies the fourth reason the origin cannot be measured.
Thesis: the world has a beginning in time. Antithesis: it does not. Both proofs
valid; both conclusions false — because both assume the world-series is a
completed totality, given as a whole, about which the question has an answer
waiting. If that is right, the search for the true origin is not a hard empirical
problem awaiting better instruments. There is nothing of the appropriate kind to
be true *of*.

Which means stipulating is not a retreat from a better method. There is no better
method.

*Cusanus*, *De docta ignorantia*, 1440: no proportion holds between the finite and
the infinite, so no increase in finite precision constitutes approach. That is
this project's thesis, five and a half centuries early and stated more cleanly.

#section("Constitutive and regulative")

Kant distinguishes principles that tell you what objects *are* from principles
that tell you how to go on *investigating*. Transcendental illusion is the slide
from the second to the first — taking a rule for how to proceed as a description
of an object.

The datum is a regulative posit: *count from here*. It does not say *here is where
time began*. The physical claim is the constitutive-looking statement, and it is
the one that has been rendered inert.

Kant had no mechanism for this. He had argument, vigilance, and the expectation
that a careful reader would hold the distinction under pressure — and he says
himself the illusion does not go away once diagnosed.

What this project has is not a better argument. It is the same distinction
maintained by something that does not get tired, does not skim, and does not want
the flattering answer.

#section("A cautionary tale")

In 1922 Pavel Florensky — mathematician, electrical engineer, Orthodox priest —
published *Мнимости в геометрии*, a study of imaginary quantities in geometry.

Most of it is exactly what it says. In one part he observes that Dante's journey
in the *Comedy* only *closes* — descending through the Earth, passing the centre,
emerging at the Mount of Purgatory, ascending through the spheres and arriving
back — if the cosmos is finite and non-orientable. That is genuine mathematical
reading: it takes a structural feature of a poem seriously enough to ask what
geometry the poem presupposes.

#plate("LIFE_UNIVERSE_AND_GOD/assets/images/florensky-cosmos.png", width: 78%)[
  The cosmos Florensky's reading requires. The spheres twist so that the outermost
  rejoins the innermost; the traced line is Dante's route, in from the Empyrean,
  down through the infernal circles, out at the Mount of Purgatory. An
  illustration made for this project — not a reproduction; his book contains no
  plate like it.
]

Then, a few pages later, he considers a rigidly rotating cosmos, notes that beyond
some radius the co-rotating velocity would exceed $c$, computes the radius at
which the Lorentz factor turns imaginary — and identifies that surface with the
Empyrean.

The distance between the two is exactly one move. In the first, a formal structure
is used to *interpret* a text. In the second, a formal artifact is treated as
*designating a place*, licensed by nothing except that the number came out at a
suggestive magnitude.

He had no rule against the second move. That is not a moral observation: he had a
serious apparatus and enormous facility, and nothing in his method said *here is
where the mathematics stops describing and starts being read*.

This project has such a rule, and it is implemented rather than stated. It does
not make the project safe — a reader looking at a 61-digit integer will still read
it as a fact about being, and no type system reaches that. What the rule protects
is the *arithmetic*.

// ── 5 ───────────────────────────────────────────────────────────────
#part("The theological background")

Also short, and included for a specific reason: several of the design's decisions
have precedents that are older, better named, and arrived at for entirely
different purposes. Reporting them is not an argument that any tradition is
correct. It is an argument that the decisions are not eccentric.

#section("The beginning is not a member of the series")

Basil of Caesarea, *Hexaemeron*, fourth century: the beginning of a road is not
the road, and the beginning of a house is not the house. A beginning is not a
member of the series it begins.

Applied to time: the beginning of time is not a time. Not a very short duration,
not the first instant of a sequence, not a moment you could point at.

That is the datum's content, stated with no arithmetic whatsoever. Tick 0 is not
the first tick of the universe; it is where counting starts, and those are
different sorts of thing.

Augustine gives the operational form. Asked what God was doing before creating
heaven and earth, he declines the joke and answers that there was no *then* —
time is among the created things, and the question presupposes a container that
does not exist. That is `UCAL-E0020` exactly: an error, not a negative number.

#section("Interval as the mark of the created")

Gregory of Nyssa, against Eunomius, makes διάστημα — interval, extension, the
spread between before and after — the mark of createdness. Everything created is
διαστηματικός. God is ἀδιάστατος, without interval. The gap is not one of degree
along a scale but of *kind*: there is no interval there for a measure to measure.

Every quantity in this system is an interval. The tick count is the interval from
the datum. A delta is an interval. A window is an interval with its uncertainty
carried. The whole apparatus measures διάστημα and nothing else.

#callout(label: "Why that matters for the title")[
  An instrument for the immeasurable reads as a paradox — an instrument that
  cannot measure its subject is a failed instrument.

  Nyssa turns it into a statement about categories. The instrument reaches
  exactly as far as interval reaches, and adding digits does not approach what has
  no interval. The limit is not imprecision. It is what an interval-measure *is*.
]

#section("A stipulated epoch, eleven centuries early")

The Hebrew calendar's computational epoch is *molad tohu* — "the new moon of
chaos" — fixed by the mnemonic BaHaRaD, and falling by rabbinic reckoning roughly
a year *before* the creation it anchors.

It is computational. It is placed before the event it anchors. It is not claimed
as an observation. And it is *named for its own emptiness*: tohu is the word from
Genesis 1:2, formlessness, the void before ordering.

Compare the two namings. This project calls its epoch "the datum" and then spends
four chapters explaining that it is stipulated rather than observed. The Hebrew
calendar calls its epoch the new moon of chaos, and the explanation is in the name.

The same tradition supplies a second precedent. Time is subdivided by the *ḥelek*,
1/1080 of an hour — and 1080 was chosen because it divides cleanly, by 2, 3, 4, 5,
6, 8, 9, 10, 12 and more. That is exactly why `SECOND` is a multiple of $10^30$:
pick a constant with many factors and the subdivisions come out exact. A designer
looking at a unit and asking not *how big* but *what must it divide by*.

#section("Reliable here, not thereby necessary")

Every body parameter in the system carries an epoch, a rate, and a *validity
window*. Inside the window it computes confidently; outside it warns rather than
extrapolating.

The epistemic content of that is: the regularity has been observed here, I will
assert it here, and I decline to assert it beyond where it was observed.

Al-Ghazālī's term for the reliable-but-not-necessary regularity is *ʿāda* — divine
custom. Practical certainty within the range of custom; no claim of necessity.
Whatever the author believed about causation while writing it, the posture the
code takes is the one argued for in Baghdad in the eleventh century, for entirely
different reasons.

#callout(label: "And where it cuts the other way")[
  Ibn Rushd's reply is that to deny that things have natures from which their
  effects follow is to deny the possibility of knowledge.

  The mechanism's central promise — *choose the intercalation rule that guarantees
  this accuracy for this long* — presupposes that an orbital period has a nature
  from which its future follows. So the system computes as though Ibn Rushd were
  right and reports as though Ghazālī were, and it took that side silently, as an
  implementation consequence.

  That is a metaphysical commitment the specification never declares. It was found
  by reading the artifact against a tradition, which is the sort of thing this
  part is for.
]

// ── 6 ───────────────────────────────────────────────────────────────
#part("What came out of it")

A calendar with no Earth in its arithmetic turns out to be a good instrument for
examining calendars that have Earth in theirs.

Every calendar in the system has the same shape: a *body*, an *anchor*, a *leap
rule*, and *cycles*. Three of the four are computed from the body's own periods.
The intercalation rule in particular is *derived*, by expanding the fractional day
count as a continued fraction and reading off the convergents — the best rational
approximations, in the precise sense that no fraction with a smaller denominator
comes closer.

#section("The Julian rule is derivable")

#terminal(caption: "ucal cal show earth-d — the walk")[
```
intercalation:
  whole_days_per_year  365
  rule                 31/128
  bound                1 local day per 10000 local years
  walked:
    1: 1/4        — 1 day slips in 128 local years
    2: 7/29       — 1 day slips in 1234 local years
    3: 8/33       — 1 day slips in 4269 local years
    4: 31/128     — 1 day slips in 400000 local years   <- chosen
    5: 752/3105   — 1 day slips in 62100000 local years
    ...
```
]

Convergent 1 is `1/4`: one leap day every four years. That is the Julian calendar,
derived from Earth's rotation and orbit with no knowledge of Rome.

#section("The Gregorian rule is not")

Now look for `97/400` in that walk.

It is not there. Not at depth 4, not at depth 8 where the expansion terminates,
*not at any depth* — and there is a check in the build whose only job is to assert
its absence.

This is not an accident of where the walk stopped; it is a theorem. Convergents
are the best rational approximation at their denominator, so if `97/400` were one,
nothing with a smaller denominator could beat it. Two things do:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 5pt),
    align: (left, right, right, left),
    [*rule*], [*denominator*], [*1 day slips in*], [*against 97/400*],
    [8/33], [33], [4,269 yr], [1.32× more accurate, denominator 12.1× smaller],
    [31/128], [128], [400,000 yr], [*124× more accurate*, denominator 3.1× smaller],
    [97/400], [400], [3,226 yr], [— the Gregorian rule],
  )
]
#v(2mm)

This is not a criticism of the Gregorian reform, which solved the problem it was
aimed at and solved it well enough to still be in use. What it means is precise:
the Gregorian rule is a *declared table*, not a derivation. Its authority comes
from a decision somebody made, not from arithmetic on cited parameters.

#callout(label: "Why this is the project's best evidence")[
  The specification claimed, in two published revisions, that the mechanism
  reproduces the Julian *and* Gregorian rules as convergents. Half of that is
  true.

  The error ran *in the direction of its author's thesis* — a machinery that
  rediscovers both historical calendars is a better story — and it survived two
  revisions because nobody checked the flattering half.

  A formal system built by someone with a thesis is exactly the sort of thing that
  can be tuned until it produces the desired result. That charge is nine hundred
  years old, and it admits only one kind of answer: a case where the system
  contradicted its builder on something already published. This is that case, and
  a build-time assertion now stops the correction being quietly reversed.
]

#section("Two more that came free")

*The Metonic cycle.* Earth names the Moon as its grouping satellite, and the same
continued-fraction machinery applied to the year and the synodic month produces
`235/19` as its sixth convergent — 235 lunar months in nineteen years. Known to
Babylon, named for an Athenian, still fixing the date of Easter and the Hebrew
calendar's leap years. It falls out of two numbers with nothing else supplied.

*Mars has no month.* Mars has two satellites; neither is anything a person would
call a month. The mechanism returns nothing rather than synthesising one — because
a month-shaped structure imposed on a body that has no month is Earth structure
leaking through a mechanism built specifically to keep it out. Whether a satellite
is "month-like" is not derivable, because *month-like* is an Earth predicate.

The absence is not a gap in the implementation. It is the implementation working.

// ── 7 ───────────────────────────────────────────────────────────────
#part("Where it stops")

A section on capability that is longer than the section on limits is a sales
document. Here are the limits.

#section("The anchor cannot be derived")

The mechanism derives units, intercalation and cycles from a body's periods. It
cannot derive *phase*. Knowing exactly how long Earth takes to rotate tells you
nothing about whether it is currently noon, and no amount of further tick counting
will supply the missing fact.

So every calendar carries one declared, cited, interval-valued constant. The
mechanism is not self-sufficient, and one component of every calendar it produces
is a measurement it did not make.

Titan has no such anchor and will not be given one by invention: no published
convention exists to cite. Its calendar is therefore complete in units,
intercalation and cycles, and incomplete in phase — and asking it for a local date
is an error rather than a guess.

#section("Four more, briefly")

/ A rogue planet has no year: #sym.dash.em no orbital period, no fraction to
  absorb, nothing to derive. Three of the four calendar components depend on a
  relationship between the body and something else; only rotation is the body's
  own.
/ A tidally locked body collapses two components into one: #sym.dash.em Titan's
  solar day is its orbit about Saturn, and its year is Saturn's about the Sun.
  Handled with no special case, and thinner for it.
/ Relativistic environments are out of scope: #sym.dash.em no time dilation, no
  worldline, no frame transformation. For a body deep in a gravity well this
  system is the wrong instrument, and it would give an answer that is wrong in a
  way it cannot detect.
/ Parameters are wrong outside their validity windows: #sym.dash.em Earth's
  rotation is lengthening by about 1.8 ms per century. The domain reaches
  $10^103$ years; the parameters are good near J2000 and degrade from there. The
  reach and the accuracy point in opposite directions, and the system warns rather
  than pretending otherwise.

#section("What the limits have in common")

Two of them are *structural* — the mechanism gives less because the body has less
to give, and correct output for an unusual body is unusual output.

The other four are *epistemic*: the mechanism needs something it cannot compute.
And in every one of those the response is the same — return an error or a warning,
and never a default.

That consistency is the actual claim. Not that the approach works everywhere,
because it does not, but that where it stops working it *says so* rather than
producing a confident number nobody can audit.

A calendar that silently defaulted Titan's anchor to Earth's would work. Every
function would return a value, no test would fail, and every Titanian date it
produced would be wrong by an unknown amount with nothing anywhere to indicate it.

// ── 8 ───────────────────────────────────────────────────────────────
#part("The claim, and what is not claimed")

#align(center, block(width: 86%, breakable: false)[
  #set par(justify: false)
  #v(2mm)
  #text(size: 11.5pt, style: "italic")[
    A measuring instrument may legitimately point at what it cannot describe,
    provided it declares that it is only pointing — and that declaration can be
    enforced mechanically rather than left to the author's discipline.
  ]
  #v(2mm)
])

The first clause is old and this document has said whose it is: Cusanus has it in
1440, Gregory of Nyssa has the reason a century earlier, and Semyon Frank reaches
it independently in 1939.

The second clause is the contribution, and it amounts to one thing. A comment
addresses a reader who is paying attention. A runtime check addresses a program
that reaches a particular line. A *type* addresses everyone who ever writes code
against the library, on every path, including the ones nobody thought about — and
requires nothing of them at all.

Someone who thinks the distinction between a stipulated datum and a measured
origin is pedantic sits down, writes the line that ignores it, and is stopped in
under a second by something with no interest in whether they agree. They have not
been persuaded. They have been refused.

#section("What the medium costs")

The argument would be cheap without this.

*A program hides its premises.* The Ghazālī/Ibn Rushd finding in Part V is one of
three metaphysical commitments the artifact makes and never declares, all found by
reading it against traditions rather than by inspecting it. An essay wears its
premises; a program buries them in type signatures where they function silently.

*A program's audience is smaller.* The argument is fully available to people who
read Rust. To everyone else it is a claim about a piece of code.

*A program can be wrong with more authority.* A false claim in an essay is
contestable by anyone who reads it. A false claim compiled into a type is enforced
against everyone downstream, and the enforcement carries no indication of whether
the rule is good.

So: mechanical enforcement is a *stronger* way to maintain a distinction and a
*worse* way to argue for one. That asymmetry is why there is a book as well as an
artifact.

#section("Not claimed")

/ Not that time began: #sym.dash.em the system stipulates a datum and counts from
  it. It asserts nothing about whether anything started.
/ Not that the Big Bang happened at tick 0: #sym.dash.em that identification is
  recorded as cited metadata, and made impossible to compute with.
/ Not that the tick is time's smallest unit: #sym.dash.em it is an instrument's
  resolution floor.
/ Not that any tradition is correct: #sym.dash.em or that any anticipated this
  work. There is one good algorithm for continued fractions and one obvious
  solution to needing an exact origin.
/ Not that base 5 is meaningful: #sym.dash.em $5^5 = 3125$ is five base-5 digits,
  and that is the whole reason.
/ Not that the system is useful: #sym.dash.em no task you have today needs a
  Planck-tick count. That is not an apology; it is the second of three facts, and
  the third depends on it.
/ Not that anyone should adopt it: #sym.dash.em there is one qualification about
  timekeeping across two or more bodies, it is a claim about coherence rather than
  a recommendation, and it is one page long in the book.

#section("What the artifact actually is")

Six crates on crates.io. 381 tests on two interchangeable integer backends that
accept and reject exactly the same values. A specification vendored, corrected in
place, and cited by the source about a thousand times, with a build step that
fails if a citation resolves to nothing. A tier table generated from the library
so it cannot drift. A lint that reports every exemption it honours.

Verification found the specification wrong in fourteen places, and one further
claim was raised and *withdrawn* — an error alleged in the specification that
turned out to be an error in the checking oracle. The withdrawal is kept in the
record, because a process that only ever finds things is indistinguishable from
one that invents them.

// ── close ───────────────────────────────────────────────────────────
#part("Where to go from here")

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 5.5pt),
    [*if you want*], [*read*],
    [the software], [`cargo add ucal-core`, or `cargo install ucal` for the command line],
    [the specification], [`spec/UCAL-1.1.md` — normative, with the fourteen corrections applied in place],
    [what the rules mean], [`spec/RULES.md` — all twenty-four, with what enforces each],
    [why it differs from the RFC], [`spec/SPEC-DELTAS.md` — the record, one entry withdrawn],
    [the full argument], [*Life, the Universe, and God* — 251 pages, eight parts, nine traditions],
  )
]
#v(4mm)

The book is what this document is an introduction to. It contains the nine
readings in full, the six samples run against real material, a chapter reserved
for what those samples *failed* to establish, and the conflicts — four of which
cut at the project rather than at the tradition being read.

It also ends where this document does not, on the part that the mechanism does not
reach. `UCAL-E0025` stops the claim entering the arithmetic. It has no reach over
what may be *read*, and there is no type signature between a number and its
reader. The instrument contains the illusion; it does not cure it.

#v(10mm)
#align(center, line(length: 28%, stroke: 0.5pt + ink_rule))
#v(6mm)
#align(center, block(width: 76%, breakable: false)[
  #set par(justify: false)
  #align(center, text(size: 10.5pt, style: "italic", fill: ink_gray)[
    Tick zero is a stipulated reference point, conventionally identified with the
    FLRW $t arrow.r 0$ limit. It is not a measurement and not an observed event.
  ])
])
#v(3mm)
#align(center, text(size: 8.5pt, fill: ink_faint, tracking: 1pt,
  "— printed by " + raw("ucal datum") + " on every run"))
