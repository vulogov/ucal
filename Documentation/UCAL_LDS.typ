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
    stroke: (left: 2pt + ink_term), {
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
  align(center, block(width: 84%, {
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

This is a technical document with scripture in it, written for readers who know
the scripture better than the author does.

It reports where a piece of software and Latter-day Saint doctrine converge, and
where they collide. Both halves are here at comparable length. The convergences
are the easy half and would make a more comfortable document; the collisions are
why it exists.

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
#part("The irritation, and what was built")

Read almost any account of the early universe and you find a sentence like
*recombination occurred about 380,000 years after the Big Bang.*

A year is the time Earth takes to circle the Sun — definitionally, not
approximately. So an event 13.8 billion years old is described in units defined
by the motion of a planet that would not exist for another nine billion years.

The number is correct. It carries a passenger. `ucal` is what you get if you
refuse the passenger.

#section("The tick and the ladder")

#term("Tick")[
  The Planck time, about $5.391 times 10^(-44)$ seconds — built from the
  gravitational constant, the reduced Planck constant and the speed of light.
  Every quantity in the system is an unsigned integer count of these.
]

A tick is too small to think in, so ticks group into *tiers*: the powers
$5^(5k)$, each exactly five base-5 digits. The reference rung is the *beat*,
$5^60$ ticks, about 46.762 milliseconds.

Base five because $5^5 = 3125$ is a number of five base-5 digits. That is the
whole reason, and it is worth saying before anyone reaches Part III and starts
looking for significance in the five. There is none, and the specification
forbids finding any.

The cost of leaving Earth units behind is that nothing on the ladder is near
anything familiar. One second is 21.385061835 beats — not a whole number, and
not close to one.

#plate("LIFE_UNIVERSE_AND_GOD/assets/images/scale-plate.png", width: 27%)[
  The domain, logarithmic: from the Planck tick through an atom's vibration, a
  heartbeat, the day, a human life, recorded history, the stratigraphic record
  and a galaxy's turning. The ceiling is about $2.29 times 10^103$ years; the
  present epoch sits at $6 times 10^(-94)$ of it.
]

#section("Three commitments")

*Time is unsigned.* Nothing precedes the datum. A result that would be earlier is
an error — `UCAL-E0020` — not a negative number. The refusal is the answer.

*No floating point, anywhere.* Not in a signature, a field, an intermediate, or
the printed output. A lint fails the build on any float token. Cosmology is
therefore done by certified interval arithmetic: it returns two numbers and a
proof the answer lies between them, rather than one number and a guess.

*Earth enters at one declared boundary.* A single exact constant, `SECOND`,
converts between ticks and seconds. Conversion in is multiplication and never
rounds. The dependency is not eliminated — that is not available to anyone — it
is *localised* somewhere you can point at.

// ── 3 ───────────────────────────────────────────────────────────────
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

Five, and the first is the largest. In every case the collision is the project's
problem rather than the doctrine's, and in every case the project has chosen
without arguing for the choice.

#section("1. The datum assumes a beginning this doctrine does not")

`ucal`'s built profile, UC-1, stipulates tick 0 and conventionally identifies it
with the FLRW $t arrow.r 0$ limit — the point a standard cosmological model
extrapolates back to. The unsigned domain follows: nothing precedes the datum,
and asking for an earlier instant is an error rather than a negative number.

That arrangement sits comfortably with a cosmology in which time itself began.

#scripture("Doctrine and Covenants 93:29")[
  Man was also in the beginning with God. Intelligence, or the light of truth, was
  not created or made, neither indeed can be.
]

Taken with the King Follett discourse of 1844 — in which the word rendered
*created* is argued to mean *organise*, and matter is held co-eternal with God —
the picture is not one in which time began. It is one in which a particular
organisation began, out of materials that did not.

#tension[
  If matter and intelligence are co-eternal, then tick 0 is not the beginning of
  time. At most it is the beginning of *this* organisation, and there is a
  "before" that is not nothing.

  UC-1's unsigned domain then stops being a principled refusal and becomes a
  *misdescription*: the system says no time exists before the datum, and this
  doctrine says otherwise.
]

The specification anticipates this. There is a second profile, called UC-Θ, in
which the datum is placed at organisation rather than at a physical origin, and
in which the interval between the two is representable rather than refused. Under
it the unsigned domain becomes *room* instead of a limit.

UC-Θ has never been built.

#callout(label: "An inversion worth stating plainly")[
  The book that this document summarises reads nine traditions. Judged by the
  Patristic and Latin material — Augustine, Basil, Aquinas, all holding creation
  *ex nihilo* including time — UC-Θ is the heterodox profile and UC-1 is the
  orthodox one.

  Read from Latter-day Saint scripture, that is exactly reversed. UC-Θ is the
  natural profile and UC-1 is the foreign one.

  So the two profiles are not two configurations of one system. They are two
  cosmologies, and the project ships the one that does not fit this reader.
]

#section("2. Kolob is a body; the ladder is a grid")

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

#section("3. Revelation has no place in the schema")

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

#section("4. A revealed ratio does not land on the grid")

A smaller finding, reported because an attentive reader would otherwise wonder,
and because it says nothing at all about the text.

Take Abraham 3:4 as a bridge constant and convert it. One Kolob revolution is
1,000 Julian years, which is 585,348,807,057,053,493,600 followed by
thirty-three zeros of ticks — and that is *not* a whole number of beats. The
remainder is 35,632,189,513,851,911,760,866,641,998,291,015,625,000 ticks.

The reason is arithmetic and it has nothing to do with Kolob. A thousand Julian
years is a whole number of *seconds*; the second carries thirty factors of five,
while the beat carries sixty. Any quantity stated in years misses the tier grid
by construction, from any source whatever.

#callout(label: "Why this is in the document at all")[
  Because it *looks* like a finding about scripture and is not, and a reader
  skimming could take it for one.

  It is the same incommensurability that makes one second 21.385 beats rather
  than a round number, showing up somewhere nobody was looking for it.
]

#section("5. Scripture chronology is classified as declared, not derived")

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

#align(center, block(width: 86%)[
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
#align(center, block(width: 76%)[
  #set par(justify: false)
  #align(center, text(size: 10.5pt, style: "italic", fill: ink_gray)[
    Tick zero is a stipulated reference point, conventionally identified with the
    FLRW $t arrow.r 0$ limit. It is not a measurement and not an observed event.
  ])
])
#v(3mm)
#align(center, text(size: 8.5pt, fill: ink_faint, tracking: 1pt,
  "— printed by " + raw("ucal datum") + " on every run"))
