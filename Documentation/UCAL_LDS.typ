// UCAL_LDS — the Universe Calendar, read from Latter-day Saint scripture.
//
//   typst compile Documentation/UCAL_LDS.typ
//
// Written for Latter-day Saint readers. It reports where the software and the
// doctrine converge and where they collide, at comparable length, and it argues
// neither true. The convergences are the easy half; the collisions are why the
// document exists.
//
// Nothing here tells a Latter-day Saint reader what their scripture means. It
// says what a piece of software does, quotes the text alongside, and marks the
// places where the two cannot both be right.

#import "LIFE_UNIVERSE_AND_GOD/design.typ": (
  ink_black, ink_gray, ink_faint, ink_rule, ink_accent, ink_smoke, ink_paper,
  ink_term, ink_code_bg, ink_call_bg, ink_term_bg, ink_conflict, ink_conflict_bg,
  body_family, mono_family,
  section, subsection, term, callout, terminal,
)

#let doc_title    = "Time Only Is Measured Unto Men"
#let doc_subtitle = "A calendar with no Earth in it, read from Latter-day Saint scripture"
#let doc_author   = "Vladimir Ulogov"

#let counter_part = counter("part")
#let part(title) = {
  pagebreak(weak: true)
  counter_part.step()
  hide(heading(level: 1, numbering: none, outlined: true, bookmarked: true, title))
  v(5mm)
  block(width: 100%, {
    text(font: body_family, size: 9pt, tracking: 2.5pt, fill: ink_gray,
      context [PART #counter_part.display()])
    v(2mm)
    text(font: body_family, size: 21pt, weight: "bold", fill: ink_black, title)
    v(3mm)
    line(length: 100%, stroke: 0.5pt + ink_rule)
  })
  v(6mm)
}

// Scripture, set apart from the prose that reads it.
#let scripture(ref, body) = {
  v(2.5mm)
  block(width: 100%, inset: (left: 10pt, right: 8pt),
    stroke: (left: 2pt + ink_term), breakable: false, {
      text(font: body_family, size: 8pt, weight: "bold", fill: ink_term,
        tracking: 1pt, upper(ref))
      v(1.8mm)
      set par(justify: false)
      text(size: 10.5pt, style: "italic", body)
    })
  v(2.5mm)
}

// Where the software and the doctrine cannot both be right.
#let tension(body) = {
  v(3mm)
  block(fill: ink_conflict_bg, stroke: (left: 3pt + ink_conflict),
    inset: (left: 10pt, right: 10pt, top: 8pt, bottom: 8pt),
    width: 100%, radius: 1pt, breakable: false, {
      text(font: body_family, size: 8.5pt, weight: "bold", fill: ink_conflict,
        tracking: 1.5pt, "WHERE THEY COLLIDE")
      v(2.5mm)
      body
    })
  v(3mm)
}

#let plate(path, width: 66%, cap) = block(breakable: false, width: 100%, {
  v(3mm)
  align(center, image(path, width: width))
  v(1.5mm)
  align(center, block(width: 84%, breakable: false, {
    set par(justify: false)
    text(font: body_family, size: 8.5pt, style: "italic", fill: ink_gray, cap)
  }))
  v(2mm)
})

#set document(title: doc_title, author: doc_author)
#set text(font: body_family, size: 11pt, fill: ink_black, lang: "en")
#set par(leading: 0.72em, justify: true, first-line-indent: 1em)

#show raw.where(block: true): it => block(
  fill: ink_code_bg, stroke: 0.5pt + ink_rule, inset: 7pt, radius: 2pt,
  width: 100%, breakable: false, text(font: mono_family, size: 8.5pt, it))
#show raw.where(block: false): it => box(
  fill: ink_code_bg, inset: (x: 2pt, y: 0pt), outset: (y: 1.5pt), radius: 1pt,
  text(font: mono_family, size: 9.5pt, it))

// ── title ───────────────────────────────────────────────────────────
#set page(paper: "iso-b5", margin: 0pt, numbering: none, header: none,
  fill: ink_paper)
#block(width: 100%, height: 100%)[
  #place(top + left, dx: 12mm, dy: 12mm,
    rect(width: 100% - 24mm, height: 100% - 24mm, stroke: 0.8pt + ink_accent))
  #place(top + center, dy: 28mm,
    image("LIFE_UNIVERSE_AND_GOD/assets/images/dial-void.png", width: 46mm))
  #place(top + center, dy: 92mm, block(width: 78%)[
    #set par(justify: false)
    #align(center)[
      #text(size: 9.5pt, tracking: 3pt, fill: ink_smoke, upper("Alma 40:8"))
      #v(8mm)
      #text(size: 24pt, weight: "bold", fill: ink_black, doc_title)
      #v(5mm)
      #line(length: 44%, stroke: 0.6pt + ink_accent)
      #v(5mm)
      #text(size: 11.5pt, style: "italic", fill: ink_smoke, doc_subtitle)
    ]
  ])
  #place(bottom + center, dy: -26mm, align(center)[
    #text(size: 10pt, fill: ink_smoke, doc_author)
    #v(2mm)
    #text(size: 8.5pt, fill: ink_smoke,
      "No doctrine is argued true here, and none is argued false.")
  ])
]
#pagebreak()

#set page(paper: "iso-b5",
  margin: (inside: 24mm, outside: 19mm, top: 25mm, bottom: 22mm),
  fill: white, numbering: "1", number-align: center,
  header: context {
    if counter(page).get().first() > 1 {
      align(center, text(font: body_family, size: 8pt, fill: ink_faint,
        tracking: 1.5pt, upper("Time Only Is Measured Unto Men")))
    }
  })
#counter(page).update(1)

// ── 1 ───────────────────────────────────────────────────────────────
#part("Why this document has that title")

The title I first wrote for this was *My knowledge of God is limited, but I do
have a scientific timeline that can fit him.* It is a good sentence and I could
not use it, because the second half is exactly the claim this project spends two
hundred pages refusing.

No timeline fits God. Not this one, not a better one, not one built with more
care. The reason is not that the instrument is crude — it is that an
interval-measure measures interval, and the created order is where interval is.
Part IV puts the argument in the words of a fourth-century bishop who made it
better than I can.

So the title is Alma's instead.

#scripture("Alma 40:8")[
  All is as one day with God, and time only is measured unto men.
]

That sentence does two things at once, and both are what this project needs. It
says measuring time is a *creaturely* activity — the thing given to men to do.
And it says the divine mode is not that, and is not reached by doing more of it.

A calendar that counts ticks is doing the thing in the first half of that verse.
It has nothing whatever to say about the second, and this document is partly an
account of how the software was built so that it *cannot* say anything about the
second, even by accident.

#section("What this is and is not")

A technical document with scripture in it, written for readers who know the
scripture better than its author does. It reports where a piece of software and
Latter-day Saint doctrine converge and where they collide, at comparable length.
The convergences are the easy half; the collisions are why it exists.

#callout(label: "The rule this document works under")[
  No doctrine is argued true here, and none is argued false. The software is
  evidence for nothing about the world.

  Where the two agree, that is reported as a convergence — not as the doctrine
  being vindicated by a program, which would be worthless, and not as the program
  being vindicated by the doctrine, which would be worse.

  Where they collide, the collision is stated as the *project's* problem. It is
  the project that has to choose, and in every case in Part V it has chosen
  without arguing for the choice.
]

// ── 2 ───────────────────────────────────────────────────────────────
#part("The irritation")

Read almost any account of the early universe and you find a sentence like
*recombination occurred about 380,000 years after the Big Bang.*

A year is the time Earth takes to circle the Sun — definitionally, not
approximately. The Julian year of astronomy is exactly 365.25 days of exactly
86,400 seconds, and those numbers are what they are because of how fast one
particular rock spins and how long it takes to circle one particular star.

So an event 13.8 billion years old is described in units defined by the motion of
a planet that would not exist for another nine billion years. The number is
correct. It carries a passenger.

The complaint is not that the units are arbitrary — all units are — but that
*provenance leaks into arithmetic*. Compute in Earth years and Earth's orbital
period sits inside every intermediate value.

#section("Three things are declared; everything else is computed")

What follows is the whole foundation. The notation, the domain, the calendars,
the cosmology — none of them is a further decision. Each falls out of three
declared things, and this part is those three.

They are worth the space in a document written for this reader in particular,
because the first of them is what makes D&C 130:4–5 *expressible*.

// ── 1 ──────────────────────────────────────────────────────────────
#part("The first declared thing: the tick")

#term("Tick")[
  The Planck time, $t_P = sqrt(planck G \/ c^5) approx 5.391247 times 10^(-44)$
  seconds. Every quantity in the system is an unsigned integer count of these,
  and nothing else is primitive.
]

#section("Why this unit and not another")

The Planck time is composed from three constants: the gravitational constant $G$,
the reduced Planck constant $planck$, and the speed of light $c$. Those bound
gravitation, quantum action and the propagation of causality respectively, and
there is exactly one combination of them with the dimension of time.

What matters here is what is *absent* from that list. No rotation. No orbit. No
body. The tick does not know about Earth, or the Solar System, or matter being
organised into planets at all. It is the interval you get when you ask the three
limiting constants of physics what a duration would be.

#section("What the tick makes possible for this reader")

Here is the point specific to this document, and it took writing the document to
see it.

#scripture("Doctrine and Covenants 130:4–5")[
  In answer to the question — Is not the reckoning of God's time, angel's time,
  prophet's time, and man's time, according to the planet on which they reside?
  I answer, Yes.
]

That proposition needs something to be true before it can even be *stated
precisely*. If every available unit were derived from some body's motion, then
"reckoning is according to the planet" could be asserted but not *compared*: you
would have Earth's reckoning and Kolob's reckoning and no common measure in which
to say how they differ, except by picking one of them as the standard — which is
the thing the verse denies.

A body-independent substrate is what makes body-relative reckoning expressible
rather than merely assertable. The tick is that substrate. Every body's periods
enter the system as exact rational multiples of it, and two bodies' calendars can
then be set side by side without either being privileged.

#callout(label: "What this convergence is not")[
  It is not a claim that the verse anticipated Planck units, or that it needs
  them. The proposition stands on its own and predates the physics by seventy
  years.

  What can be said is narrower and still worth saying: a system that wanted to
  *implement* the proposition rather than assert it would need a unit of this
  kind, and would have to go looking for one.
]

#section("What the tick is not")

*The tick is not a quantum of time.* It is the resolution floor of an instrument,
not a structure discovered in the world. Nothing in this project asserts that
time comes in indivisible parts, or that asking about a shorter duration is
meaningless. The system cannot *represent* a shorter duration; that is a fact
about the system.

The distinction matters more than it looks. Temporal atomism is a serious
position with a long history — Epicurus held it, and so did a whole medieval
school — and so is its denial. Asserting either as a side effect of having chosen
an integer type is not a respectable way to hold a metaphysical position, so the
specification declines to hold one.

#section("The one concession")

There is a place where the tick is not free of Earth, and it should be admitted
here rather than discovered later.

The Planck time's *numerical value* requires a unit of time to state it in, and
that unit is the SI second — defined by a caesium transition counted by
instruments on this planet. So the tick's length is fixed by convention against
the second, recorded, and used as declared.

What that concedes is *metrology* and nothing else. The arithmetic contains no
Earth content: ticks are counted, never converted. What is Earth-flavoured is the
sentence saying how long a tick is in seconds — a statement about translating
between two systems, not a fact used inside either.

// ── 2 ──────────────────────────────────────────────────────────────
#part("The second declared thing: the universe second")

A tick is far too small to think in. The present epoch is about
$8 times 10^60$ of them, and nobody reads a sixty-one-digit number.

#term("Beat")[
  $5^60$ ticks — 867 361 737 988 403 547 205 962 240 695 953 369 140 625 of
  them, about 46.762 milliseconds. The specification calls it the *universe
  second*.
]

#section("Two choices, each with exactly one reason")

*Base five*, because $5^5 = 3125$ is a number of exactly five base-5 digits. A
tier is therefore a clean five-digit group, and a timestamp is the tick count
written in base 5 and cut into fives. That is the entire reason for the five, and
the specification contains a rule forbidding any constant from acquiring
significance by resembling a number in a tradition. The rule was written before
any of this reading began.

*Exponent sixty*, because it places the reference rung at the scale where a human
being can notice a duration. Forty-seven milliseconds is about the resolution of
the perceptual present — the interval below which events stop being separable.

That second choice is the one concession to the reader in the whole design, and
it changes no arithmetic: the ladder could be re-anchored at any exponent without
altering a single computed value.

#section("What the ladder is")

Every rung is $5^(5k)$ ticks — 3125 of the rung below, five base-5 digits wide.

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, auto, auto, auto),
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

Because a timestamp is that count in base 5, three things follow at once and none
of them is a separate feature. Truncation *is* rounding — drop groups and you
have said the same thing less precisely, with the dropped digits being exactly
the precision surrendered. Prefix comparison *is* chronological comparison.
And writing every digit pinpoints one tick, with no accumulated error between the
coarse view and the fine one, because they are the same integer read at different
widths.

#section("The beat is not a second in disguise")

This is worth stating flatly, because it is the ladder's real cost.

One SI second is 21.385061835 beats. Not a whole number, not close to one, and no
rescaling would make it whole. `SECOND` carries thirty factors of five and `BEAT`
carries sixty, so the two share a common measure *only at the tick*.

#callout(label: "Which is why the tick is primitive")[
  Not the second, and not the beat. Two units that agree only at the tick cannot
  both be fundamental, and the one they agree at is the one that is.

  An hour is 24.6 arcs. A day is 0.189 sweeps. A year is 0.699 spans. If you want
  units with no planetary content, you do not get to keep the hour — the hour
  *is* planetary content.
]

#section("What this means for a stated ratio")

Take Abraham 3:4's ratio — one Kolob revolution to one thousand Earth years — and
put it into the system. Two things happen, and the difference between them is the
whole point of this section.

*The ratio converts exactly.* One thousand Julian years is a whole number of
seconds; a second is a whole number of ticks; so the revolution is a whole,
exact, unrounded number of ticks. Nothing is lost and nothing is approximated.

*It does not land on a round number of beats.* Expressed in the human-facing unit
it comes out as 674 861 227 352 beats and a remainder of
35 632 189 513 851 911 760 866 641 998 291 015 625 000 ticks.

#callout(label: "Why that is worth a paragraph")[
  Because it is easy to read the second fact as though it said something about
  the verse, and it does not.

  A reader might reasonably expect that a stated ratio ought to come out *clean*
  in a well-built system, and that its failing to do so is a mark against one or
  the other. It is neither. "Clean in Earth years" and "clean in powers of five
  above the tick" are two different tidinesses, and no unit system is tidy in
  another's terms.

  The same arithmetic makes one SI second 21.385061835 beats rather than a round
  number, for exactly the same reason: seconds carry thirty factors of five, the
  beat carries sixty, and the two meet only at the tick.

  So *any* quantity stated in years misses the beat grid, from any source
  whatever — a papal bull, an IAU resolution, or this verse. What survives is
  what matters: the ratio is exactly representable, in the unit the system treats
  as primitive.
]

That is the practical form of the earlier claim that the tick is primitive and
the beat is a convenience. Exactness lives at the tick. Tidiness lives at the
beat, and tidiness was never promised.

// ── 3 ──────────────────────────────────────────────────────────────
#part("The third declared thing: the datum")

A count needs somewhere to start counting from, and the obvious place is not
available.

#section("Why the origin cannot be measured")

The published age of the universe is 13.787 billion years, plus or minus 0.020
billion. That is an excellent measurement and, like every measurement, an
interval rather than a point.

Convert the uncertainty to ticks and it is a number with fifty-eight digits —
about 0.145% of the entire span from the datum to now. Define the datum *as* the
measured age and every timestamp in the system inherits that wobble, and the
exact integer arithmetic becomes decoration over a guess.

You cannot get an exact origin from an inexact measurement. So the origin is not
taken from the measurement.

#section("What was declared instead")

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

Tick zero is *stipulated*: declared, not discovered, and the specification says in
as many words that it is not a measurement and not an observed event.

`ORIGIN_OFFSET` — the distance from the datum to the SI epoch — is 9 304 311 741
502 590 385 beats. A whole number of them, deliberately, so that the sub-beat
digits of the bridge epoch are all zero.

And the rounding to a whole beat is *reported* rather than absorbed: −0.017190364
seconds, printed on every run. A citation says where a number came from; this
chain says where it came from and what happened to it on the way in.

#section("What follows, and what does not")

Two things follow from the datum. The domain is *unsigned*: it begins there, and
no earlier instant is representable — a result that would be earlier is
`UCAL-E0020`, an error rather than a negative number.

And the physical claim about where the origin falls is held *separately*: cited,
with its exact magnitude, in a type with no arithmetic operations at all.

What does *not* follow is any claim that nothing preceded it. That distinction is
the subject of Part VI, and it is the reason this document is called what it is.

#callout(label: "The three, and what they generate")[
  / The tick: #sym.dash.em fixes what is counted, and makes body-relative
    reckoning comparable rather than merely assertable.
  / The beat: #sym.dash.em fixes the notation, and with it the tier ladder, the
    meaning of truncation, and the 64-byte binary form.
  / The datum: #sym.dash.em fixes where counting starts, and with it the unsigned
    domain and the separation of the origin claim from the arithmetic.

  Calendars, the SI bridge, the cosmology and the 512-bit domain are consequences
  of these three. Nothing below them is a further decision.
]

#part("Where the doctrine got there first")

Four places, and the first is not a resemblance.

#section("Reckoning is according to the planet")

The mechanism that produces calendars in this system has one rule above the
others: there is *one* derivation path, every body goes through it, and Earth is
an ordinary instance rather than the template. No Earth branch, no crate named
after a body, and a test that builds Earth's calendar and Mars's by the identical
code from data alone.

That rule exists to prevent a specific failure: Earth quietly becoming the
template, one convenience at a time, until the mechanism is an Earth calendar
with parameters.

#scripture("Doctrine and Covenants 130:4–5")[
  In answer to the question — Is not the reckoning of God's time, angel's time,
  prophet's time, and man's time, according to the planet on which they reside?
  I answer, Yes.
]

That is the rule's premise, stated in 1843, without the software.

Not that Earth's reckoning is primary and others approximate it. Not that there
is a true reckoning somewhere that local ones are shadows of. *Reckoning is
indexed to a body*, and the indexing is the whole story.

#callout(label: "What the convergence is worth")[
  It is the most direct statement of this premise found in any tradition
  surveyed, and it predates the software by 183 years.

  What it does not supply is the mechanism. "According to the planet on which
  they reside" is a proposition; expanding a continued fraction over two orbital
  periods is a procedure. The convergence is at the level of the claim, and the
  document should not inflate it past that.
]

#section("A conversion with both frames named")

Every foreign unit in this system crosses one declared boundary, and a bridge
constant is required to say what it converts *between* — both sides, tagged.

#scripture("Abraham 3:4")[
  And the Lord said unto me, by the Urim and Thummim, that Kolob was after the
  manner of the Lord, according to its times and seasons in the revolutions
  thereof; that one revolution was a day unto the Lord, after his manner of
  reckoning, it being one thousand years according to the time appointed unto
  that whereon thou standest.
]

Read as a conversion statement, it has the shape the specification requires:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 5pt),
    [*in the verse*], [*what it corresponds to*],
    [one revolution = one day unto the Lord], [the source quantity, in source units],
    ["after his manner of reckoning"], [the source profile tag],
    [one thousand years], [the target quantity],
    ["according to the time appointed unto that whereon thou standest"],
      [the target profile tag],
  )
]
#v(2mm)

The verse does not say "a day is a thousand years" and leave you to work out
whose day and whose years. It names both frames.

That is worth pausing on, because ordinary technical writing routinely fails at
it. *The timeout is 30* — thirty what, measured by whom? A ratio with only one
frame named is not a conversion; it is a number waiting to be misread. Whatever
one thinks of the source, the *form* here is the form the specification demands.

#section("Law with bounds and conditions")

Every body parameter in the system carries an epoch, a rate of secular change,
and a *window of validity*. Inside the window it computes; outside it warns
rather than extrapolating confidently.

Earth's rotation is lengthening by about 1.8 milliseconds per century, and its
tropical year shortening by about half a second. A parameter treated as a
constant is wrong the moment you leave the epoch it was measured at — which
matters most at exactly the deep-time scale this project targets.

#scripture("Doctrine and Covenants 88:38")[
  And unto every kingdom is given a law; and unto every law there are certain
  bounds also and conditions.
]

A regularity holds *within bounds and conditions*, and the bounds are part of the
law rather than a caveat attached to it. That is the design decision, stated as a
general proposition about law.

#section("What can be numbered, and by whom")

This is the one that gave the document its title, and it does the most work.

#scripture("Moses 1:37")[
  And the Lord God spake unto Moses, saying: The heavens, they are many, and they
  cannot be numbered unto man; but they are numbered unto me, for they are mine.
]

Notice precisely what is said to be beyond numbering: the *heavens*. Not
durations. Not intervals. Not the count of ticks between two moments in the
created order.

And beside it, Alma:

#scripture("Alma 40:8")[
  Now whether there is more than one time appointed for men to rise it mattereth
  not; for all do not die at once, and this mattereth not; all is as one day with
  God, and time only is measured unto men.
]

Measuring time is presented as what creatures do, in contrast to a divine mode
where all is as one day. So this tradition, like several others surveyed, locates
time-measurement firmly on the creaturely side — and treats that as its proper
place rather than as a limitation to be transcended.

A calendar that counts ticks is doing the thing that is given to men to do. That
is a modest claim, and it is the one this project has been circling since its
first page.

// ── 4 ───────────────────────────────────────────────────────────────
#part("The instrument's own limit")

Before the collisions, the thing the two agree about most deeply — and the reason
the title had to change.

#section("What the instrument measures")

Gregory of Nyssa, in the fourth century, argued against Eunomius that *διάστημα*
— interval, extension, the spread between before and after — is the mark of
createdness. Everything created is διαστηματικός; what is uncreated is
ἀδιάστατος, without interval. The gap is not one of degree along a scale. It is a
difference of *kind*: there is no interval there for a measure to be a measure of.

Every quantity in this system is an interval. The tick count is the interval from
the datum. A duration is an interval. An uncertainty window is an interval with
its bounds carried. The whole apparatus, from the 512-bit integer to the
cosmological enclosures, measures διάστημα and nothing else.

#callout(label: "Which is why no timeline can fit God")[
  Not because the timeline is imprecise, and not because a better one might
  manage it. Adding digits does not approach what has no interval, in the same
  way that no finite magnitude approaches the infinite by growing.

  The instrument reaches exactly as far as interval reaches. That is not a
  confession of failure. It is a statement about what an interval-measure *is* —
  and it is the same point Alma makes in five words.
]

#section("And what it refuses to compute with")

The design's central move follows from that. The datum — tick 0 — is
*stipulated*: declared, not measured. The published age of the universe is
13.787 ± 0.020 Gyr, and that uncertainty is a fifty-eight-digit number of ticks,
about 0.145% of the whole span. An exact count cannot inherit it.

So the physical claim about where the origin falls is kept *separately*: cited,
with its exact magnitude, in a type that has *no arithmetic operations at all*.
You can read it, print it, cite it. You cannot compute with it — and three tests
exist whose job is to *fail to build* if anyone ever does.

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

That is the whole of the project's contribution: not the observation that an
instrument should declare what it cannot reach — which is old, and better said by
Cusanus in 1440 — but that the declaration can be *enforced by a machine* rather
than left to the author remembering.

// ── 5 ───────────────────────────────────────────────────────────────
#part("Where they collide")

Three, and one that looks like a collision and is not. In every case the problem
is the project's rather than the doctrine's, and in every case the project has
chosen without arguing for the choice.

#section("The one that is not a collision: the datum")

This is where a reader would most expect the two to come apart, and the title of
this document is why they do not.

`ucal`'s profile UC-1 stipulates tick 0 and conventionally identifies it with the
FLRW $t arrow.r 0$ limit. The domain is unsigned: no earlier instant is
representable, and asking for one is an error rather than a negative number.

It is easy to read that as *nothing existed before* — and I wrote an earlier
draft of this document that read it exactly so, and built a collision on top of
it. That was wrong, and the specification says so plainly. `BIG_BANG_CLAIM` is a
*signed* window precisely because the limit "may lie before the datum, which is
not representable as a tick." A system asserting that nothing precedes the datum
would have no use for a type that can express something preceding it.

#callout(label: "What the unsigned domain actually says")[
  The datum is the best available starting point for time *as this system can
  date it*, and everything it dates is dated from there. What may lie earlier is
  a question the instrument declines rather than answers.

  It is a limit on *range*, not a claim about *existence*. Alma's verse is the
  whole of it: time is measured unto men, and a measuring instrument reaches as
  far as measuring reaches.
]

#scripture("Doctrine and Covenants 93:29")[
  Man was also in the beginning with God. Intelligence, or the light of truth, was
  not created or made, neither indeed can be.
]

Taken with the King Follett discourse of 1844 — in which the word rendered
*created* is argued to mean *organise*, and matter is held co-eternal with God —
the picture is not one in which time began, but one in which a particular
organisation began, out of materials that did not.

The instrument does not contradict that. It is *silent* about it, and silence is
not contradiction. Where the doctrine speaks of what precedes organisation, the
instrument has nothing to say, because there is nothing there it can date.

#subsection("What remains, which is a limitation and not a disagreement")

If the doctrine holds there is a dateable "before" — an eternity of organised
matter preceding this arrangement — then the instrument's *reach* falls short of
the doctrine's *scope*. That is a real limitation and worth naming as one.

The specification anticipates it. A second profile, UC-Θ, would place the datum
at organisation rather than at a physical origin and make the interval before it
representable. Under UC-Θ the unsigned domain becomes *room* rather than an edge.
It has never been built.

So the honest statement is narrow: UC-1 cannot date what precedes its datum, and
UC-Θ would extend the range rather than correct a false claim. Whether extending
it is desirable is a separate question, and one this document does not settle.

#callout(label: "An inversion worth stating anyway")[
  The book this document summarises reads nine traditions. Judged by the Patristic
  and Latin material — Augustine, Basil, Aquinas, all holding creation *ex nihilo*
  including time — UC-Θ is the *heterodox* profile: it posits pre-existing
  material shaped rather than made.

  Read from Latter-day Saint scripture, that is reversed. UC-Θ is the natural
  profile and UC-1 the foreign one.

  Two profiles here are not two configurations of one system. They are two
  cosmologies — and the project ships the one that reaches less far for this
  reader, without having argued that it should.
]

#section("1. Kolob is a body; the ladder is a grid")

The mechanism permits no privileged body. The universal ladder is the powers
$5^(5k)$ — an abstract grid — and the specification is explicit that no body's
period may occupy a special place in it.

#scripture("Abraham 3:9")[
  And thus there shall be the reckoning of the time of one planet above another,
  until thou come nigh unto Kolob … which Kolob is set nigh unto the throne of
  God, to govern all those planets which belong to the same order as that upon
  which thou standest.
]

Abraham 3 describes a hierarchy of reckonings, and the hierarchy has a *place* at
its centre. The governing referent is a body.

#tension[
  Both arrangements are hierarchies with a privileged referent. They differ in
  what sort of thing sits at the privileged point: a named body, or a power-of-5
  grid.

  So the honest description of what this project did is not that it abolished the
  privilege. It *relocated* it — from a body to a grid.

  And a grid is not neutral merely by being abstract. It was chosen: base five,
  anchored at $5^60$, with the reference rung placed where a human can notice a
  duration. A reader of Abraham 3 could fairly say that a ladder anchored to human
  perception is parochial about something too — just about a different thing.
]

I think the relocation is defensible, and the reason is that a power ladder can be
re-anchored without changing any arithmetic, while a governing body cannot be
replaced without changing the cosmology. But that is an argument the project
should make rather than assume, and until it was written down here it had assumed
it.

#section("2. Revelation has no place in the schema")

Every body parameter in the system records *how it was determined*: measured to a
stated precision, derived from other parameters, or cited to a published source
with an uncertainty window. The field is closed; those are the values it admits.

A revealed ratio fits none of them.

The consequence is precise and worth stating exactly, because it cuts one way and
not the other. Abraham 3:4 *can* enter the system as a declared bridge constant,
because a bridge is required to be declared and tagged on both sides and the verse
is. It *cannot* enter as an anchor — the constant that fixes where a calendar's
count begins in phase — because an anchor must carry a determination method and a
stated uncertainty, and no reading of the text supplies either without inventing
them.

#tension[
  The system is hospitable to this tradition's *ratios* and closed to its
  *epistemology*.

  That is a real exclusion and it is not neutral. It is not that revelation is
  judged unreliable; the schema has no way to represent the category at all, and
  a schema that cannot represent a category has taken a position on it by
  omission.
]

#section("3. Scripture chronology is classified as declared, not derived")

The system distinguishes calendars whose rules it *derives* from a body's motion
from calendars whose rules are *declared tables*. That distinction is factual
rather than evaluative — it records where a rule's authority comes from — but it
does not flatter, and it is applied without exception.

Applied to the Gregorian calendar, it produces an uncomfortable result: the leap
rule of 1582, `97/400`, is not derivable from Earth's motion at any depth, and a
rule twelve times simpler is more accurate. The Julian rule *is* derivable — it is
the first thing the mechanism produces.

#scripture("Doctrine and Covenants 77:12")[
  … as God made the world in six days, and on the seventh day he finished his
  work, and sanctified it … even so, in the beginning of the seventh thousand
  years will the Lord God sanctify the earth …
]

#tension[
  A seven-thousand-year structure is a declared period. The mechanism does not
  derive it from any body's motion, because no body's motion yields it.

  So the instrument would classify it exactly as it classifies the Gregorian leap
  rule: *declared, not derived*.

  That has to be said, because a system that called the Gregorian declared and a
  scriptural period something else would not be classifying. It would be
  flattering, and a classification that bends for the author's sympathies is not a
  classification.
]

The classification is not a verdict. Chapter after chapter of the book makes the
same point about the Gregorian, and the strongest defence of *declared* calendars
in the whole survey comes from a different tradition entirely — the Orthodox
churches that keep the Julian calendar knowingly, on the crudest rule available,
for reasons of communion that have nothing to do with accuracy.

Derivation is not a calendar's only criterion. It is the only one this instrument
can measure, which is a fact about the instrument.

// ── 6 ───────────────────────────────────────────────────────────────
#part("What is not claimed")

A negative inventory, because a document with scripture in it will be quoted and
the quoting will not always be careful.

/ Not that any doctrine is true: #sym.dash.em or false. The software is evidence
  for nothing about the world, and its agreements are not endorsements.
/ Not that the scripture anticipated this work: #sym.dash.em D&C 130:4–5 states a
  proposition that this project also holds. Body-relative reckoning is a thought
  available to anyone who considers another planet, and there is one good
  algorithm for continued fractions.
/ Not that the instrument reaches God: #sym.dash.em it measures interval, and
  interval is the mark of the created order. That is Part IV, and it is why the
  title changed.
/ Not that time began: #sym.dash.em the system stipulates a datum and counts from
  it. Whether anything began is a question it declines, and Part V is honest that
  its unsigned domain nevertheless leans one way.
/ Not that base 5 is meaningful: #sym.dash.em $5^5 = 3125$ is five base-5 digits.
  The specification explicitly forbids any constant acquiring significance from
  resembling a number in a tradition, and that rule was written before the survey
  began.
/ Not that the seven thousand years are wrong: #sym.dash.em the instrument
  classifies the period as declared. It has no opinion about whether a declared
  period is correct, and neither does its author in this document.
/ Not that the system is useful: #sym.dash.em no task you have needs a
  Planck-tick count. That is stated in the project's own preface as a fact rather
  than an apology.

#section("What is claimed")

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

The first clause is old. Cusanus has it in 1440; Gregory of Nyssa supplies the
reason a millennium before that; Alma says it in eleven words.

The second is the contribution, and it is small and specific. A comment addresses
a reader who is paying attention. A runtime check addresses a program that
reaches a particular line. A *type* addresses everyone who ever writes code
against the library, on every path, including paths nobody thought about — and it
requires nothing of them at all.

Someone who thinks the distinction between a stipulated datum and a measured
origin is pedantic sits down, writes the line that ignores it, and is stopped in
under a second by something with no interest in whether they agree.

// ── 7 ───────────────────────────────────────────────────────────────
#part("Where to go from here")

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(210)) },
    inset: (x: 5pt, y: 5.5pt),
    [*if you want*], [*read*],
    [the software], [`cargo install ucal`, or `cargo add ucal-core`],
    [the chapter this document draws on],
      [*Life, the Universe, and God*, chapter 23 — the same material at greater
       length, alongside eight other traditions],
    [the datum argument in full], [the same book, Part IV — four chapters],
    [what the rules mean], [`spec/RULES.md` — all twenty-four, and what enforces each],
    [a shorter general account], [`UCAL_INTRO` — 29 pages, no scripture],
  )
]
#v(4mm)

One last note, and it is the note the book itself ends on.

The type system stops the claim about the origin from entering the arithmetic. It
has no reach whatever over what a person *reads*. A reader who looks at a
sixty-one-digit integer will take it for a fact about being, because that is what
a very precise number looks like — and no type signature stands between a number
and its reader.

The instrument contains that; it does not cure it. Which is, in the end, a
narrower version of the thing Alma said: the measuring is ours to do, and what it
does not reach is not reached by measuring more carefully.

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
