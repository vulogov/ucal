#import "../design.typ": *
#import "@preview/cetz:0.4.2"

#chapter(number: 27, title: "Six samples")

#section("What a program can do that an argument cannot")

An argument is addressed. It is made by someone, to someone, about something, and its
force depends on the person receiving it being willing to follow. That is not a defect
— it is what arguments are — but it has two consequences that matter here.

An argument cannot be *run against material its author did not choose*. It can be
extended to new cases, but the extending is done by the author, who selects the cases.

And an argument cannot *return an answer its author did not want*. It can fail to
persuade; it can be refuted by someone else. It cannot, on its own, produce a
conclusion its maker was not looking for.

A program can do both. Chapter 10 is the book's evidence for the second — the
mechanism contradicted a claim its author had published twice. This chapter is the
evidence for the first: six samples, run against material the project was not built
for, each producing an artifact you can open.

#callout(label: "Everything here is generated")[
  Every number in this chapter comes from a file under `assets/output/`, produced by
  `samples/run-samples.py` against the commit in `PINNED.md`. Nothing is illustrative
  and nothing was typed by hand.

  Where a sample produced a weaker result than hoped, chapter 28 says so.
]

#section("S1 — comparative chronology on one axis")

Four declared epochs, each converted to absolute ticks.

#terminal(caption: "assets/output/S1-chronologies.txt")[
```
epoch                         civil (Julian)    ticks since datum
--------------------------------------------------------------------------
Seder Olam (Masoretic AM 1)   3761 BC-10-07     807020180242960413984034848...
Byzantine AM 1                5509 BC-09-01     807020077918219588719351401...
Ussher                        4004 BC-10-23     807020166021508493862768038...
Julian Day epoch              4713 BC-01-01     807020136627266382332510054...

Spread between the widest pair:
  1,748.1 years
```
]

The four differ by 1,748 years, and the difference is not error. Byzantine reckoning
follows the Septuagint's longer genealogies; the Masoretic text gives shorter ones.
Ussher and Scaliger are working from the same materials with different methods.

#claim("interpretation")[
  What the sample demonstrates is not that one is right. It is that *putting them on
  one axis at all* requires a mechanism that keeps them apart.

  Each row is a different profile. The arithmetic within a row is meaningful; the
  arithmetic across rows is not, because the rows are not measuring from the same
  declared origin. Rule P is what makes the table legitimate — every value carries its
  profile, so the comparison is visible rather than accidental.

  Without that, a chronological table is a set of numbers that look comparable and are
  not. This is the ordinary condition of chronological tables.
]

#section("S2 — calendar audit by convergent")

The sample with a gated experiment attached, and the one worth publishing on its own.

Six historical intercalation rules, tested against Earth's convergent ladder.

#terminal(caption: "assets/output/S2-calendar-audit.txt")[
```
rule                       value   convergent?    1 day slips in
--------------------------------------------------------------------------
Julian                       1/4      yes — #1            128 yr
Gregorian                 97/400            NO          3,226 yr
Revised Julian           218/900            NO         31,034 yr
Persian (Jalali)            8/33      yes — #3          4,269 yr
Medler                    31/128      yes — #4        400,000 yr
Coptic / Ethiopian           1/4      yes — #1            128 yr

FINDING
  derived (a convergent): Coptic / Ethiopian, Julian, Medler, Persian
  declared (not one):     Gregorian, Revised Julian
```
]

#claim("interpretation")[
  The split is real and it does not follow accuracy, which is what makes it
  interesting.

  The Persian calendar of 1079, worked out by a commission including Omar Khayyam, uses
  8/33 — the third convergent. It is *derived* in this system's exact sense: it is
  where the arithmetic says the good approximation is at that denominator.

  The Gregorian reform of 1582 uses 97/400, which is not a convergent at any depth and
  is beaten by 8/33 with a denominator twelve times smaller.

  The Revised Julian rule of 1923 is more accurate than the Gregorian — 31,034 years
  against 3,226 — and is also not a convergent. So accuracy and derivedness come apart
  in both directions, and neither implies the other.
]

#callout(label: "GE-A2 — the kill criterion does not fire")[
  The experiment asked whether this audit produces a non-trivial result. Its kill
  criterion: if every historical calendar turns out to be a convergent, the finding is
  empty and the sample becomes a footnote.

  Four of six are convergents and two are not, and the two that are not include the
  calendar most of the world uses. The result stands, and the method is reusable on any
  body and any rule.
]

#section("S3 — uncertainty audit")

For any chronology, separate the *cited* from the *stipulated*.

The worked case is UC-1's own datum, because that is the one this project is
answerable for:

#terminal(caption: "assets/output/S3-uncertainty-audit.txt")[
```
CITED      13.787 Gyr +/- 0.020 Gyr, Planck 2018 VI
           31 557 600 s per Julian year (definitional)
           SECOND, as the declared bridge constant
STIPULATED that tick 0 is the origin of the count
           that the datum is rounded to a whole beat
DISCARDED  0.017190364 s, reported rather than absorbed
```
]

Three categories, and the third is the one most systems do not have. A rounding that
is *discarded and reported* is different from one that is absorbed, and the difference
is auditable.

#claim("interpretation")[
  The same audit applied to *Seder Olam* would separate the scriptural genealogies from
  the compression of the Persian period. That audit is computable, and chapter 19 is
  four pages on why this book computes it and does not conclude from it.

  Turning the audit on the project's own datum first is not modesty. It is the only
  order in which the sample is not an accusation.
]

#section("S4 — cross-body simultaneity")

One instant, three calendars — the demonstration that "now" is not a shared object.

#terminal(caption: "assets/output/S4-simultaneity.txt")[
```
earth-d:      earth-d/1: 0027-213.7987 c328    derived (Rule K)
mars-d:       mars-d/1:  0082-086.1665         derived (Rule K)
earth-civil:  2026-07-31T19:11:12 TT           legacy — UCAL-W0005
```
]

Each rendering carries its kind. Each derived rendering carries its anchor revision, so
values computed under different anchor determinations are never silently compared. The
legacy rendering carries a warning, every time.

#claim("interpretation")[
  The Martian date is not a translation of the Earth date. Both are renderings of one
  tick count, and neither is prior.

  That is what Rule K buys, and it is easier to see here than in any amount of prose
  about Earth being an instance rather than a template. There is no conversion between
  the two rows. There is a value, and there are three ways of writing it.
]

#section("S5 — measuring diastema")

The datum-to-present interval, stated with no Earth content in the units.

#terminal(caption: "assets/output/S5-diastema.txt")[
```
ticks since the datum:
  8070205189128471254993117657693008777530466139316558837890625

on the tier ladder:
  T5 deep   31        T2 sweep  3000
  T4 drift  687       T1 arc    1638
  T3 span   2481      T0 beat   3018

in beats — the universe second:
  whole            9304313109135981143
  remainder_ticks  216453223532315359877956032752990722656250
```
]

No second, no day, no year appears. Every quantity is a count of ticks or of powers of
five of them.

#claim("interpretation")[
  Chapter 22 supplied the word for what this measures. Διάστημα — interval, extension,
  the spread between before and after — is Gregory of Nyssa's mark of createdness, and
  everything in the display above is an instance of it.

  So the sample is not merely a demonstration that Earth units can be avoided. It is the
  concrete form of the chapter 22 finding: the instrument measures interval, interval is
  what created things have, and that is both what it can reach and why it reaches no
  further.
]

#section("S6 — a revealed ratio, evaluated")

Abraham 3:4 states a conversion with a profile tag on each side. The sample takes it
seriously as a bridge constant and reports what it does and does not determine.

#terminal(caption: "assets/output/S6-revealed-ratio.txt")[
```
WHAT IT IMPLIES
  1 Kolob revolution = 5853488070570534936000...000 ticks
                     = 674861227352 beats
                     = 0.007076 drifts

  It is NOT a whole number of beats. The remainder is
    35632189513851911760866641998291015625000 ticks
  which is 5^60-relative, not a rounding artifact.

WHAT IT LEAVES UNDETERMINED
  the phase — Rule J requires an anchor; the text supplies a period
  the uncertainty — Rule C requires epoch, rate and window; none given
  the determination method — 'revealed' is not a Determination value

VERDICT
  ACCEPTED as a declared bridge constant (Rule Y)
  REFUSED  as a Rule J anchor
```
]

The non-exactness was not anticipated and is the sample's most interesting output.

#claim("interpretation")[
  A thousand Julian years is a whole number of seconds. Seconds carry $5^30$; the beat
  carries $5^60$. So a quantity defined in years cannot land on a whole beat unless it
  happens to supply the missing thirty factors of five, and a thousand years does not.

  That is chapter 7's incommensurability — the two seconds share a measure only at the
  tick — arriving in a place nobody was looking for it. Any ratio stated in Earth
  years, from any source, will miss the tier grid the same way.

  It says nothing whatever about Abraham 3:4. It is a fact about the arithmetic of
  expressing year-based quantities on a base-5 ladder, and the sample surfaced it only
  because it computed the conversion instead of describing it.
]

#claim("interpretation")[
  The verdict is a statement about the schema, not about the source, and the distinction
  is the whole value of the sample.

  The system does not refuse the ratio because it is revealed. It accepts it — Rule Y
  requires a bridge to be declared and tagged on both sides, and Abraham 3:4 is tagged
  more explicitly than most engineering documentation manages.

  It refuses it as an *anchor* because anchors carry a determination method and an
  uncertainty window, and the text supplies neither. The same refusal would fall on a
  ratio from any source that gave a period without a phase.

  This is a template. Any tradition's stated ratio gets the same treatment, and the
  treatment is legible enough that someone who holds the text as scripture and someone
  who does not can agree on what the instrument did.
]

#section("The pattern across the six")

#v(2mm)
#block(breakable: false, width: 100%)[
#align(center, cetz.canvas({
  import cetz.draw: *
  let rows = (
    ("S1", "chronologies", "Rule P keeps four profiles apart"),
    ("S2", "calendar audit", "derived and declared come apart from accuracy"),
    ("S3", "uncertainty audit", "cited / stipulated / discarded"),
    ("S4", "simultaneity", "three renderings, no conversion"),
    ("S5", "diastema", "interval, with no Earth in the units"),
    ("S6", "revealed ratio", "accepted as bridge, refused as anchor"),
  )
  let y = 0
  for (id, name, finding) in rows {
    rect((0, y), (0.72, y + 0.5), fill: luma(235), stroke: 0.4pt, radius: 0.05)
    content((0.36, y + 0.25), text(size: 8.5pt, weight: "bold", id))
    content((0.95, y + 0.25), anchor: "west", text(size: 8.5pt, name))
    content((3.3, y + 0.25), anchor: "west", text(size: 8pt, fill: luma(90), finding))
    y = y - 0.62
  }
}))
#v(1mm)
#figcap[8][
  The six samples and what each returned. Every row is a file under
  `assets/output/`, regenerated by one script.
]
]

#claim("interpretation")[
  Read together, the six do one thing: they take material the project was not built for
  — rabbinic chronology, a Persian reform, a scriptural ratio — and put it through a
  mechanism that has no opinion about any of it.

  That is the sense in which a program can be run against material its author did not
  choose. The author chose the six inputs, obviously. What he did not choose was what
  came out, and S2 in particular came out differently from what a person who likes the
  Gregorian calendar would have guessed.
]

#recap((
  [An argument cannot be run against material its author did not choose, and cannot return an answer he did not want. A program can do both.],
  [S1: four chronologies on one axis, legitimate only because Rule P keeps their profiles apart.],
  [S2: four of six historical rules are convergents; the Gregorian and the Revised Julian are not. GE-A2's kill criterion does not fire, and accuracy and derivedness come apart in both directions.],
  [S3: cited, stipulated, and *discarded* — the third category is what makes a provenance chain auditable, and the audit is turned on this project's own datum first.],
  [S4: three renderings of one value with no conversion between them — Rule K, visible.],
  [S5: the datum-to-present interval with no Earth content, which chapter 22 names as διάστημα.],
  [S6: a revealed ratio accepted as a bridge constant and refused as an anchor — a statement about the schema, not the source.],
))
