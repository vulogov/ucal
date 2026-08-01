#import "../design.typ": *

#chapter(number: 23, title: "Latter-day Saint")

Rule K holds that reckoning is body-relative and that Earth is an instance rather than
a template. Chapter 14 presented that as a piece of software architecture with a
philosophical shape.

It is also a doctrinal proposition, stated in 1843, in a text that says it more
directly than this project's own specification manages.

#section("What the direction holds")

#claim("tradition")[
  *Reckoning is body-relative.* Doctrine and Covenants 130:4–5: "In answer to the
  question — Is not God's time, angel's time, prophet's time, and man's time, according
  to the planet on which they reside? I answer, Yes."

  *A declared bridge, with a tag on each side.* Abraham 3:4, of Kolob: "one revolution
  was a day unto the Lord, after his manner of reckoning, it being one thousand years
  according to the time appointed unto that whereon thou standest."

  *Law with bounds and conditions.* D&C 88:38: "unto every kingdom is given a law; and
  unto every law there are certain bounds also and conditions."

  *Organization rather than creation from nothing.* The King Follett discourse of 1844
  argues that the word rendered "created" means to organise — that matter is
  co-eternal with God and is arranged rather than brought from nothing.

  *What is numbered, and to whom.* Moses 1:37: "the heavens, they are many, and they
  cannot be numbered unto man; but they are numbered unto me." Alma 40:8: "all is as
  one day with God, and time only is measured unto men."
]

#section("Which rule it illuminates")

Rule K, one derivation mechanism with Earth as an instance. Rule Y, the declared
bridge. Rule C, validity windows. And the datum.

#section("The convergences")

#subsection("D&C 130:4–5 is Rule K's thesis, stated in 1843")

Rule K exists to prevent failure mode F9 — Earth becoming the template rather than an
instance. Chapter 14 described the drift it guards against and the test that catches it.

The 1843 text states the underlying claim without the software: time is *according to
the planet on which they reside*. Not that Earth's reckoning is primary and others
derive from it, and not that there is a true reckoning somewhere that local ones
approximate. Reckoning is indexed to a body, and the indexing is the whole story.

#claim("interpretation")[
  This is the most direct statement of Rule K's premise found in any direction
  surveyed, and it predates the software by 183 years.

  What it does not supply is the mechanism. "According to the planet on which they
  reside" is a proposition; `derive_leap_rule` is a procedure. The convergence is at the
  level of the claim, not the method, and the book should not inflate it past that.
]

#subsection("Abraham 3:4 is a bridge constant with a profile tag on each side")

This one is structurally precise enough to be worth laying out.

Read the verse as a conversion statement:

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    [*element*], [*what it corresponds to*],
    [one revolution = one day unto the Lord], [the source quantity, in the source body's units],
    ["after his manner of reckoning"], [the source profile tag],
    [one thousand years], [the target quantity],
    ["according to the time appointed unto that whereon thou standest"], [the target profile tag],
  )
]
#v(2mm)

That is a declared bridge constant with both sides tagged — which is exactly Rule Y's
requirement and exactly Rule P's. The verse does not say "a day is a thousand years"
and leave you to guess whose day and whose years. It names both frames.

#claim("interpretation")[
  Compare how often ordinary technical writing fails at this. "The timeout is 30" —
  thirty what, measured by whom? A ratio without both frames named is not a conversion;
  it is a number waiting to be misread.

  The verse names both. Whatever one thinks of its source, its *form* is the form this
  project's Rule Y exists to require.
]

#subsection("D&C 88:38 is a validity window")

"Unto every law there are certain bounds also and conditions."

Chapter 8 said every body parameter carries an epoch, a rate, and a window of validity,
and that evaluating outside the window warns rather than extrapolating. Chapter 20
found that posture to be Ghazālī's *ʿāda* with the theology removed.

Here it is again, in a different tradition, as a general proposition about law: a
regularity holds *within bounds and conditions*, and the bounds are part of the law
rather than a caveat on it.

#subsection("What cannot be numbered, and what can")

Moses 1:37 and Alma 40:8 together do something useful for this book's central worry.

Moses 1:37 says the *heavens* cannot be numbered unto man. It does not say durations
cannot. Alma 40:8 says time "is measured unto men" — measuring time is presented as
what creatures do, in contrast to the divine mode where all is as one day.

#claim("interpretation")[
  So this direction, like chapter 22's διάστημα, locates time-measurement firmly on the
  creaturely side and treats that as its proper place rather than as a limitation to be
  overcome.

  A calendar that counts ticks is doing the thing given to men to do. That is a modest
  claim and it is the one the book has been circling since the preface.
]

#conflict[
  *Two, and the second relocates rather than resolves.*

  *Revelation has no vocabulary in `Determination`.* The system records how a parameter
  was determined — measured, derived, cited to a published source with an uncertainty.
  A revealed ratio does not fit any of those categories.

  Abraham 3:4 can enter as a *declared bridge constant*, because Rule Y's requirement is
  that a bridge be declared and tagged, and the verse is. It cannot enter as a Rule J
  anchor, because anchors must be cited to a determination with a stated uncertainty
  window, and "revealed" is not a determination method the schema knows.

  That is a real exclusion and it is not neutral. The system is hospitable to this
  tradition's *ratios* and closed to its *epistemology*.

  *Kolob is a body where Rule K.5 permits only a grid.* Abraham 3 describes Kolob as
  the governing body — "nearest unto the throne of God", the one after whose reckoning
  the others are set. The hierarchy is real and it is centred on a *place*.

  Rule K.5 permits no privileged body. The universal ladder is the powers $5^(5k)$, and
  chapter 14 was explicit that nothing on it is any body's.
]

#section("What it changes")

#claim("interpretation")[
  The Kolob conflict is better stated as *the privilege is located differently*, and
  the restatement is not a softening — it is more accurate and less flattering.

  Chapter 14 declined the grand Copernican reading and said the privilege was
  *relocated, not abolished*: from a body to a power-of-5 grid. At the time that was a
  caution against overclaiming.

  This chapter shows what it means. Abraham 3 has a governing body at the centre of a
  hierarchy of reckonings. UC-1 has an abstract ladder. Both are hierarchies with a
  privileged referent; they differ in *what sort of thing* sits at the privileged
  point.

  And a grid is not neutral merely by being abstract. It was chosen — base five,
  anchored at $5^60$, with the beat placed where a human can notice a duration. Chapter
  4 admitted that last part as "a concession to the reader". Someone reading Abraham 3
  could fairly observe that a ladder whose reference rung is set to human perception is
  not obviously less parochial than a ladder whose reference is a named star; it is
  parochial about a different thing.

  I do not think that is right, and I think the reason is that a power ladder can be
  re-anchored without changing any arithmetic while a governing body cannot be
  replaced without changing the cosmology. But the objection is good enough that the
  book has to make the argument rather than assume it, and until this chapter it had
  assumed it.
]

#recap((
  [D&C 130:4–5 states Rule K's premise in 1843: reckoning is according to the planet on which one resides.],
  [Abraham 3:4 is a declared bridge constant with a profile tag on *both* sides — the form Rule Y requires and ordinary technical writing routinely omits.],
  [D&C 88:38's "bounds also and conditions" is a validity window as a general proposition about law.],
  [Moses 1:37 and Alma 40:8 locate time-measurement on the creaturely side and treat that as its proper place — the same move as chapter 22's διάστημα.],
  [*Conflict:* revelation has no vocabulary in `Determination`, so a revealed ratio may be a bridge constant and never an anchor. And Kolob is a governing *body* where Rule K.5 permits only a grid.],
  [*What changes:* the privilege was relocated, not abolished — and a ladder anchored where humans notice durations is parochial about something too. The book has to argue that, not assume it.],
))
