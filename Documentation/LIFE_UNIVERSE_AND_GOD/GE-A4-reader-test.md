# GE-A4 — the two-reader test

**Status: NOT RUN.**

This is the one experiment the author cannot run. It needs two people who are
not him, and it has been deferred at every stage of production, which is exactly
why it is written down here rather than left as an intention.

Chapter 28 records it as not run. This file is the instrument, so that running
it is a matter of finding two readers rather than of designing a protocol.

---

## The question

Can both intended audiences state the book's thesis after a single pass?

The preface claims two paths through the book — an engineer's and a reader's —
and the whole dual-audience structure rests on that claim being true. It has
never been tested.

## The kill criterion

> If neither can, the dual-audience goal fails and the book picks one:
> engineer-primary, with Part VI as an extended appendix.

Stated in RFC UCAL-A1 §21. It is a real criterion with a real consequence, and
a book that quietly declined to run the test would be keeping a claim it had
chosen not to check.

## Who

**Reader A — the engineer.** Someone who writes software professionally and has
no particular background in philosophy or theology. Rust familiarity is helpful
and not required; the book's code listings are short and explained.

**Reader B — the non-engineer.** Someone who reads seriously in philosophy,
theology, history of ideas, or a related field, and who does not program.

Neither should have discussed the project with the author beforehand. Both
should know only what the cover and preface tell them.

## What each reads

| reader | path | chapters |
|---|---|---|
| A — engineer | Parts I, II, III, V | preface, 1–10, 14–17 |
| B — non-engineer | Parts I, IV, VI, VII, VIII | preface, 1–4, 11–13, 18–32 |

One pass. No re-reading, no notes, no looking things up. The point is what
survives an ordinary reading, not what a careful study would recover.

## What to ask, in this order

Ask these after the reading, without showing the questions in advance.

**1. In your own words, what is this book claiming?**

Open-ended. Do not prompt, do not narrow, do not accept "it's about a
calendar" — ask "and what does it claim about that?" once, then stop.

*Record the answer verbatim.* A paraphrase by the person administering the test
is a paraphrase by someone who knows the answer.

**2. What is the book *not* claiming?**

Chapter 31 is an explicit negative inventory. A reader who took the book's
central discipline should be able to produce at least two items unprompted.

**3. What is tick zero?**

The one factual question. The answer that counts is some form of *a stipulated
reference point, not a measurement*. "The Big Bang" is a fail, and it is the
fail the whole of Part IV exists to prevent.

**4. Was there anything the book got wrong, or admitted getting wrong?**

Chapters 9, 10 and 28 are the material. This tests whether the corrections
registered as content or read as throat-clearing.

**5. Who is this book for?**

Tests whether the dual-audience structure was visible or whether the reader
felt they were reading someone else's book with sections skipped.

## Scoring

The thesis, for comparison — do not show it to the reader before question 1:

> A measuring instrument may legitimately point at what it cannot describe,
> provided it declares that it is only pointing — and that declaration can be
> enforced mechanically rather than left to the author's discipline.

A **pass** on question 1 is an answer containing both clauses in any wording:
that the instrument points at something it cannot measure *and* declares so,
and that the declaring is enforced by the machine rather than by the author's
care.

Half the thesis is not a pass. "It's about a calendar that admits it can't
measure the beginning" is the first clause only, and the second clause is the
book's entire contribution.

| outcome | consequence |
|---|---|
| both readers pass Q1 | GE-A4 passes; the dual-audience claim stands |
| one passes | partial. Record which, and which clause the other missed |
| neither passes | **kill criterion fires.** The book becomes engineer-primary, Part VI becomes an extended appendix, and the preface's two-paths section is rewritten |

Questions 2–5 do not gate the experiment. They are diagnostic: they say *where*
a failure happened, which is what a revision would need.

## Recording the result

Append the outcome to this file — including a failure, and especially a
partial. Then update:

- chapter 28's gated-experiment table, which currently reads **not run**
- the book's README
- `Documentation/Release_Notes/0.2.0.md`

## A note on who should not administer this

Not the author.

The questions are open-ended, and an author who knows the intended answer will
hear it in a reply that does not contain it. That is not dishonesty; it is what
knowing the answer does to listening. If the author must be present, they should
record verbatim and score afterwards, or better, have someone else ask.

---

## Result

*Not yet run. Record it here.*

```
date:
reader A (engineer):
  Q1 verbatim:
  Q1 verdict:  pass / half / fail
  Q2:
  Q3:
  Q4:
  Q5:
reader B (non-engineer):
  Q1 verbatim:
  Q1 verdict:  pass / half / fail
  Q2:
  Q3:
  Q4:
  Q5:
outcome:
consequence:
```
