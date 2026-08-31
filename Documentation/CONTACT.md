# Three things this project needs from someone who is not its author

These three need someone who is not the author. Everything else was a matter of
sitting down.

**1.0 was gated on them and shipped without them.** The asks below went three
cycles unanswered and 1.0.0 was released anyway, by the author's decision; the
argument against doing that is left standing in
[`STABILITY.md`](STABILITY.md) and [`ROAD-TO-1.0.md`](Proposals/ROAD-TO-1.0.md)
rather than revised to agree with the outcome.

That makes these **more** valuable now, not less. Until 1.0 a finding cost a
minor version; now it costs a major one — and it is still much better to know.
If something here is wrong, the useful moment to say so has not passed, it has
only become more expensive to act on.

The asks below are specific on purpose. "Contributions welcome" gets nothing,
and deserves to.

---

## Why this is asked at all

The API has never been used by anyone but its author. Every breaking change so
far cost nothing, which means **none of them tested whether the surface is
right** — only that it could be changed. Six releases have refined a design
against a single set of assumptions.

The project's own argument is that what nobody checks, drifts. That applies to
this repository's design as much as to its arithmetic, and it is the one form of
checking that cannot be automated.

---

## C1 — Reproduce the constants in another language

**Time: about thirty minutes. No Rust required. Nothing to install.**

This is the most valuable of the three, and the cheapest.

`fixtures/vectors.json` contains every declared constant of profile UC-1, its
derivation, and 13 worked fixtures. It is deliberately a set of *derivations*
rather than outputs: the provenance chain carries every intermediate, so a
mismatch localises to the step that diverged instead of reporting only that a
final number differs.

### The whole task

Compute these three in whatever language you like — Python, Java, Haskell, a
calculator with bignums — and compare the decimal strings.

```
BEAT          = 5^60
              = 867361737988403547205962240695953369140625

SECOND        = 18548584399861 × 10^30
              = 18548584399861000000000000000000000000000000

ORIGIN_OFFSET = round_half_even(AGE_ticks / BEAT) × BEAT
  where AGE_ticks = 13787000000 × 31557600 × SECOND
              = 8070204002895596515944343085635637180530466139316558837890625
```

Every quantity is an exact integer. There is no floating point anywhere in the
definition and there must not be any in your check — that is the point of the
exercise.

If those three match, `vectors.json` has 45 tier values, 13 fixtures and 4
continued-fraction tables you can go on to, and `spec/CONFORMANCE.md` explains
the file's shape.

### If you would rather see the expected values first

Since 0.8.0 the binary prints them itself, so you need neither this repository
nor the vector file to know what you are aiming at:

```
cargo install ucal
ucal verify
```

It re-derives all three from their definitions and shows what it got. Read the
last field before you trust it: that command is a **self-check**, so its
agreeing with itself is not the confirmation being asked for here. It exists to
make the target cheap to see, not to stand in for C1.

### What to report

Either outcome is a result, and **a mismatch is worth more than a match**.

- **A match**, with the language you used. That is the first independent
  confirmation this project has ever had, and it goes in the release notes.
- **A mismatch**, with which quantity and what you got. That is a defect in one
  of the two implementations and finding out which is exactly what the vectors
  exist for.
- **"I could not tell what to compute from the documents."** Also a result, and
  a more useful one than either, because it says the specification is not
  self-contained and this repository cannot discover that from the inside.

### Verifying you have what was published

```
minisign -Vm fixtures/SHA256SUMS \
  -P RWTgVaXr8eTV6+dsVwvMkwZglwUJS69tF+78i2MFUi5LBaUXPf66M+FV
```

The same key is printed on **crates.io** and **docs.rs** in the READMEs of
`ucal` and `ucal-core`, where a published version cannot be edited afterwards.

**Compare against a crate published after 2026-08-31.** This key replaced
`D0E4E5A9439E54CC` on that date, because the passphrase to the old secret key
was lost, so every version through 1.11.0 carries the retired key on crates.io
permanently. Until a version ships with the new one, the repository and the
registry disagree — and the sentence that stood here said such a disagreement
means something is wrong and you found it.

That instruction was right, and this is the case it did not cover: a mismatch is
evidence of *a change*, not of an attack, and nothing in a rotation announced by
the same repository that was rotated can tell you which. The retired key is in
`fixtures/ucal-retired.pub` and still verifies everything it signed. Nothing
signs the replacement, because the key that could have is the one that was
lost.

Do not read that as five independent confirmations. One person put it in all
five places, so against a forgery you are no better off than with one copy —
what the copies buy is that a *change* to the key here can be contradicted by an
artefact nobody can quietly edit. `spec/CONFORMANCE.md` states the key's custody
plainly, including what a signature here does *not* establish and what would
actually improve it.

---

## C2 — Read part of the book and answer three questions

**Time: about an hour. No technical background required for one of the two
roles.**

*Life, the Universe, and God* claims to work for two different readers: someone
who came for the software and someone who came for the argument. That claim has
never been tested, and the author is the one person who cannot test it.

The protocol is written out in
[`GE-A4-reader-test.md`](LIFE_UNIVERSE_AND_GOD/GE-A4-reader-test.md) — it needs
two people, one from each direction, and it has a stated kill criterion, so a
negative result is a real outcome rather than a disappointment.

---

## C3 — Use the command line for something, then say what was confusing

**Time: ten minutes.**

```
cargo install ucal
ucal datum
ucal explain 8070205189123984864657505252035637180530466139316558837890625
ucal cal list
ucal between 0 8070205189123984864657505252035637180530466139316558837890625
```

Then tell me what you expected and did not get.

Several decisions were guesses about a reader who has never existed: that
foreign units should be off by default and behind `--bridge`; that a
certification block belongs in the output at all; that promoting a wide column
beneath its row reads better than truncating it; that a duration is better
stated as a walk down the tier ladder than as a number of seconds. Each is
defensible and none has met anyone.

`ucal cal list` is the one to look at if you only look at one. Seven derived
calendars, five of them with no anchor — which is the project's whole argument
sitting in a table, and the place a reader most likely to think it is broken.

---

## What happens to what you send

It is recorded, including if it says the design is wrong. This project's release
notes carry four gated experiments whose kill criteria fired and were written
down rather than worked around; a fifth was dropped in 0.4.0 after a measurement
contradicted the reason for doing it.

Contact findings are expected to break things. Before 1.0 there was a cycle set
aside to spend them cheaply; that window has closed, and a finding now waits for
`2.0`. This is exactly the cost the ordering was meant to avoid, and it was
accepted knowingly — which is a reason to report something, not a reason to
assume it is too late.

## Where to send it

An issue on [github.com/vulogov/ucal](https://github.com/vulogov/ucal), or a
patch, or an email — whichever is least trouble. A rough note is worth more than
a polished one that never gets written.
