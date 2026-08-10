# W4 — two ladders, aligned, with a local unit ladder for the body

**Status: research proposal. Not built, and the kill criterion is written first.**

A gated experiment in the shape the RFC uses for GE-1…GE-6 and this repository
has used for GE-U4 and D5: a question, a thing that would answer it, and a
condition under which the answer is *no* and the work is deleted rather than
defended.

---

## The question

Absolute time is body-independent by construction. A local calendar is derived
from one body's periods. **Nothing in this program shows the two side by side**,
and the relationship between them is the whole content of Rule K: a local
calendar is a *consequence* of a body, not a translation of Earth's.

`ucal cal show earth-d <t>` prints local fields. `ucal explain <t>` prints the
universal decomposition. A reader who wants to see that the second causes the
first has to hold two outputs in their head, and the causation is what the
project is arguing.

So: **can a two-column alignment make the derivation visible, where two separate
commands have not?**

## What is proposed

One view, two ladders, one instant.

```
  UNIVERSAL                            LOCAL — mars-d
  ─────────────────────────            ─────────────────────────
  T5  deep      31                     year        2 084
  T4  drift     687                    month       — (no grouping satellite)
  T3  span      2 481                  sol         412
  T2  sweep     2 999      ◀── here ──▶ local hour  17
  T1  arc       3 108                  local min    3
  T0  beat      2 437                  local sec   44
  T-1 flicker   1 104                  ⋯
```

The left column is fixed for every body: the same forty-five rungs, the same
instant. The right column is derived from the body's own periods, and **its rows
are different for every body** — Mars has no month, Mercury's day outlasts its
year, Jupiter has no surface for its rotation to belong to.

Zooming moves both columns together. Zoom in and the left descends a tier while
the right descends whatever the *local* unit ladder's next step is; zoom out and
both climb. The alignment is the point: at each step the reader sees which
universal tier the local unit currently sits near, and watches the two drift
apart as they go, because they are incommensurable and only meet at the tick.

## The part that does not exist yet: a local second

The right-hand column above assumes a body has an hour, a minute and a second.
**It does not.** A derived calendar in this program has a year, a day, a
day-fraction, and cycles where a satellite provides them. Below the day there is
nothing.

That gap is the research, and it is not obviously fillable. Three routes, with
what each costs:

### Route A — divide the local day the way Earth's is divided

`local_second = solar_day / 86 400`.

**Cheap, and wrong in the way this whole project objects to.** 86 400 is
24 × 60 × 60, and 24, 60 and 60 are Babylonian and Earth-historical. Dividing
Titan's day into 86 400 parts imports Earth's clock into a body-relative
calendar wearing a local name — failure mode F9, and precisely what Rule K
exists to prevent. It would work, it would look reasonable, and it would be the
substitution the project was built to refuse. Route A is listed to be rejected.

### Route B — derive the subdivision from the body, as Rule K derives intercalation

Rule K does not choose a leap rule; it takes the ratio of two periods and reads
the answer out of a continued fraction. The same machinery could ask: **is there
a subdivision of this body's day that a continued fraction picks out?**

There is a candidate. A body's day and its *rotation* differ (§8.3), and its day
and its year differ; both ratios have convergents. But a convergent of
`year / solar_day` is a *calendar* structure — it says how many days in a year —
and there is no second period that would make `solar_day / X` fall out. The
subdivision has no second measurement to be a ratio *of*.

**That is a finding and probably the answer.** Intercalation is derivable
because two independent periods exist. Sub-day structure is not, because nothing
below the day is a period of the body at all. A "local hour" is not a fact about
Mars; it is a convenience, and Rule K's whole discipline is that a convenience
must not be dressed as a derivation.

### Route C — base-5, the same as everywhere else

`local_beat = solar_day / 5^k`, choosing `k` so the result is near some target.

This has the merit of using the project's own base rather than Earth's, and the
demerit of the target: near *what*? Any answer names a duration a human finds
comfortable, and human comfort is calibrated on Earth's rotation. The Earth
enters through the target instead of through the divisor, which is the same
import with an extra step.

Route C is honest only if it declines to pick `k` and shows the whole column —
every power of five that divides the local day — leaving the reader to say which
one they find useful. That is not a local second. It is the universal ladder
again, expressed in local days, which may be the correct answer to the whole
question: **there is no local second, and the universal ladder is what a body has
instead.**

## The kill criterion

**A reader given the two-column view can state, without being told, that the
local calendar is derived from the body and not translated from Earth's — and a
reader given `ucal cal show` and `ucal explain` separately cannot.**

Both halves matter, as in GE-U4. The first alone is satisfied by a reader who
already knew; the second is what distinguishes the view from the two commands it
would sit beside.

And a second criterion, on the sub-day question specifically:

**If Route B's finding holds — that no subdivision below the day is derivable —
then the right-hand ladder stops at the day, and the view must show it stopping
rather than filling the space.** A two-column display whose right column runs
out is a better outcome than one that invents six rows to look symmetrical. If
the implementation cannot bear an asymmetric view, it is the wrong
implementation.

## Why this is a proposal and not a branch

Three reasons, in order of weight.

**It needs a reader.** Like GE-U4's navigator and GE-A4's two-reader test, the
criterion cannot be run by the author, who already knows the answer to the
question being asked. Seven cycles of asking have produced nobody
([`CONTACT.md`](../CONTACT.md)), and this would be the third gated experiment
waiting on the same missing person.

**The cheap version comes first.** GE-U4 learned this: the recorded walk cost
sixty lines of shell against a TUI's dependency tree, and building it found a
defect in the first ten minutes. The cheap version here is a **static two-column
rendering at one instant, no zoom** — which is a `Doc` with two `Rows` values
and could ship in a minor release. If a reader cannot see the derivation in the
static view, they will not see it in an animated one, and the zoom is the
expensive half.

**The sub-day question may dissolve the feature.** If Route B's finding is
right, the right-hand ladder has four rows and the left has forty-five, and the
"alignment" is a column of blanks. That is worth knowing before building a
zoom for it — and it is worth knowing *anyway*, because it is a fact about the
model rather than about a view.

## What would make this not worth building

- **If `ucal show` already conveys it.** That command puts one instant in
  several calendars at once. It does not show the universal ladder beside them,
  but a reader might supply that themselves, and finding out costs nothing.
- **If the answer is a diagram rather than a program.** The book's scale plate
  covers adjacent ground. A second plate showing one instant on both ladders
  would cost an afternoon and reach every reader of the PDF.
- **If the right column stops at the day.** Then the honest artefact is a
  paragraph in `Documentation/CLI.md` saying that a body has a year, a day and
  cycles, and nothing below — not a view built to display an absence.

## Prior art in this repository

Listed so a future reader knows what was tried, not consulted for design:
`ucal show` (one instant, several calendars), `ucal cal show` (one calendar in
full, with its derivation), `ucal explain` (the universal decomposition),
`ucal ladder` (the grid), `ucal between` (a duration on the grid), and
[`GE-U4-walk.sh`](GE-U4-walk.sh) (the grid, travelled). Each shows one side.
None shows both at once, which is either the gap this proposal names or evidence
that nobody needed it.
