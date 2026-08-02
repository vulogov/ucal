#import "../design.typ": *

#chapter(number: 28, title: "Null results")

Chapter 27 reported what the six samples produced. This chapter reports what they did
not, because a demonstration reports only the first and this book is trying not to be
one.

It runs eight pages against chapter 27's ten. That is stated rather than corrected: the
remedy for a short chapter is not padding, and a book that criticises the gap between
*this ran* and *this established something* should not close a page gap by writing more
words about it. The proportion is what it is.

Some of these are weaker versions of results already claimed. Some are things that
came out and mean less than they appear to. One is a sample that essentially failed.

#section("S1 — the axis is a picture, not a comparison")

The chronology table looks like a comparison and is not one.

Four epochs sit on one axis. Rule P is what makes that legitimate, and Rule P is also
what makes it *useless for the obvious purpose*: you cannot subtract Byzantine AM from
Masoretic AM and get a meaningful number, because they are not offsets from a shared
origin.

#claim("interpretation")[
  What the sample actually produced is a rendering. Four values, each correct in its
  own profile, displayed together — and the display is exactly as informative as
  putting four rulers side by side with different zeros marked.

  The 1,748-year figure in chapter 27 is the difference between two tick counts, which
  is a real number about two declared epochs. It is not a disagreement about when
  anything happened, because the two traditions are not both dating the same event by
  different methods; they are reading different manuscript families.

  I let that figure carry more weight in chapter 27 than it should. It is the width of
  the picture, not a measured discrepancy.
]

#section("S2 — what the audit cannot tell you")

The strongest sample, and three limits on it.

*It only tests solar intercalation.* The audit compares leap-day rules against the
tropical year. It says nothing about the lunisolar structure of the Hebrew or Islamic
calendars, nothing about month lengths, nothing about week structure. A calendar is
more than its leap rule, and this tests the leap rule.

*"Not a convergent" is not a criticism.* Chapter 10 made this point about the Gregorian
and it needs making again about the Revised Julian, which is *more accurate* than the
Gregorian and equally not a convergent. Derivedness and accuracy are independent, which
means neither is a ranking.

*Six rules is a small audit.* The RFC calls S2 publishable independently. On six data
points, with two negatives, that is a method demonstration rather than a survey. A real
audit would cover the Islamic, Chinese, Indian, and Mayan systems, and would have to
handle calendars whose intercalation is not expressible as a single fraction at all —
which several are.

#claim("interpretation")[
  The last is the serious one. The mechanism can only audit a rule that *is* a
  fraction. An observational calendar — one that intercalates when someone sights the
  new moon — has no fraction to test, and the audit has nothing to say about it.

  That excludes a large share of the historical calendars a survey would want. So the
  method's reach is narrower than "calendar audit" suggests: it audits *arithmetic*
  calendars, and a calendar being arithmetic is itself a historical development that
  not every tradition made.
]

#section("S3 — the sample that essentially failed")

This is the weakest of the six and it should be said plainly.

S3 was specified as an uncertainty audit with *Seder Olam*'s missing years as the
worked case. What it produced is an audit of UC-1's own datum, and a paragraph saying
that the *Seder Olam* audit is computable.

#claim("interpretation")[
  Computable is not computed. The sample does not contain the analysis it was specified
  to contain.

  Chapter 19 gives the reason, and the reason is good: performing the audit and
  publishing the result would put a mechanically-derived finding about a religious
  tradition's chronology into a book whose rules forbid its author from concluding
  anything from it. The restraint is deliberate.

  But there is a difference between *declining to draw a conclusion* and *not doing the
  work*, and S3 does the second while chapter 19 argues for the first. Those are not
  the same position, and the book has been sliding between them.

  The honest statement: the audit was not run. Not because it is impossible, and not
  only because the conclusion would be forbidden, but because running it would have
  required a level of engagement with rabbinic chronology that the author does not
  have and that a footnote cannot supply. Chapter 19's principled restraint is real and
  it is also, in this instance, standing in for an absence of competence.
]

#section("S4 — a rendering, not a synchronisation")

The cross-body sample shows one instant in three calendars, and chapter 27 called it a
demonstration that "now" is not a shared object.

That is right about the *rendering* and says nothing about the physics.

#claim("interpretation")[
  Two observers on Earth and Mars do not share a "now" for reasons this sample does not
  touch: relativity of simultaneity, light-travel delay of four to twenty-four minutes,
  differing gravitational potentials.

  S4's point is narrower — that a tick count renders differently in different local
  calendars, and that the renderings carry their kinds and anchor revisions so they are
  not confused.

  Chapter 16 listed relativistic environments as out of scope. This sample is where a
  reader is most likely to forget that, because "cross-body simultaneity" sounds like a
  claim about simultaneity and is a claim about formatting.
]

#section("S5 — avoiding Earth units is not avoiding Earth")

S5 displays the datum-to-present interval with no second, day or year in it, and
chapter 22 gives that a name: διάστημα.

The null result is that the display is Earth-free and the *number* is not.

#claim("interpretation")[
  The tick count reached that value through `ORIGIN_OFFSET`, which came from 13.787 Gyr
  — Julian years — times `SECOND`, the declared bridge constant. Chapter 2 conceded
  that the tick's length is fixed by convention against the SI second.

  So S5 demonstrates that Earth units can be kept out of the *presentation*. It does
  not demonstrate that they are out of the value, and they are not.

  This is the honest form of what chapter 7 called localisation: the dependency was
  moved to one declared place, not eliminated. S5 is the place where that distinction
  is easiest to lose, because a display with no Earth units in it looks like a quantity
  with no Earth in it.
]

#section("S6 — the sample answered a question nobody asked")

S6 accepted Abraham 3:4 as a bridge constant and refused it as an anchor, and produced
one unanticipated fact: a Kolob revolution is not a whole number of beats.

#claim("interpretation")[
  That fact is real and it is about arithmetic, not about the text. Any quantity stated
  in Julian years misses the tier grid, because years reduce to seconds and seconds
  carry $5^30$ against the beat's $5^60$.

  So the sample's most striking output is a restatement of chapter 7's incommensurability
  in a context where it happens to look like a finding about scripture. It is not one,
  and a reader skimming could easily take it for one.

  What the sample did *not* do is anything with the ratio. It converted a number and
  classified it against a schema. It did not test the ratio against anything, could not
  have, and the "template for any tradition's stated ratio" framing oversells what a
  unit conversion plus a schema check amounts to.
]

#section("The gated experiments, reported")

Four of the six §21 experiments in the software's specification hit their kill
criteria, and chapter 9 said so. The book's own experiments deserve the same treatment.

#v(2mm)
#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 5pt),
    [*id*], [*question*], [*outcome*],
    [GE-A1], [Does the code diverge from the spec enough to carry Part III?], [passed — Part III stands],
    [GE-A2], [Does S2 produce a non-trivial result?], [passed — 4 of 6, split real],
    [GE-A3], [Does the deletion test pass?], [passed — green throughout],
    [GE-A4], [Can both audiences state the thesis?], [*not run*],
    [GE-A5], [Does Part VI argue rather than summarise?], [passed at the book band],
    [GE-A6], [Can anchors be determined usefully?], [Earth and Mars yes, Titan no],
  )
]
#v(2mm)

#claim("interpretation")[
  GE-A4 has not been run and the book is nearly finished. Its criterion is that two
  readers — one engineer, one not — each state the thesis after a single pass, and its
  kill condition is that the book picks one audience and demotes Part VI to an appendix.

  That experiment requires two people who are not the author. It is the only one of the
  six that cannot be run by the person writing, and it is therefore the only one that
  has been deferred at every stage.

  Recording it as *not run* rather than quietly dropping it is the minimum. The next
  thing after the minimum is to make running it cheap, so the protocol is written out
  in `GE-A4-reader-test.md` — who reads what, the five questions in order, the scoring,
  and the consequence if it fails. Designing the test was the part the author could do.

  The dual-audience claim in the preface is, as of this chapter, untested.
]

#section("What the null results have in common")

#claim("interpretation")[
  Five of the six entries above are the same failure in different clothes: *the sample
  demonstrated a mechanism and the chapter described it as demonstrating a finding.*

  S1 renders four epochs and was described as comparing them. S4 formats one instant
  three ways and was described as showing something about simultaneity. S5 suppresses
  Earth units in a display and was described as producing an Earth-free quantity. S6
  converts a number and was described as a template for evaluating revealed ratios.

  The pattern is not carelessness in any single case. It is that a working mechanism
  producing real output is genuinely impressive to its author, and the gap between
  *this ran* and *this established something* is exactly where enthusiasm lives.

  Chapter 10 identified the same failure at the level of a specification claim: the
  error ran in the direction of the thesis, and survived because nobody checked the
  flattering half. Here it is at the level of six samples, caught only because a chapter
  was reserved in advance for catching it.

  That is the argument for reserving the chapter. Not honesty as a virtue — honesty as
  a *scheduled step*, because the author who wrote chapter 27 was not in a position to
  see what chapter 28 says.
]

#recap((
  [S1 renders four epochs on one axis; Rule P makes the display legitimate and the subtraction meaningless. The 1,748-year figure is the width of a picture, not a discrepancy.],
  [S2 tests solar intercalation only, on six rules, and cannot audit an observational calendar at all — which excludes much of what a survey would want.],
  [S3 essentially failed: the *Seder Olam* audit was specified and not run, and chapter 19's principled restraint is standing in for an absence of competence.],
  [S4 is about formatting, not simultaneity; the physics chapter 16 excluded is exactly what a reader will assume it addresses.],
  [S5 keeps Earth out of the presentation and not out of the value — the dependency was localised, not eliminated.],
  [S6's most striking output is chapter 7's incommensurability wearing scriptural clothes, and the sample tested nothing.],
  [GE-A4 has not been run. The dual-audience claim is untested, and it is the one experiment the author cannot run alone.],
  [Five of six failures are one failure: the sample demonstrated a mechanism and the chapter called it a finding.],
))
