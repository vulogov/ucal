# The road to 1.0

Three questions were asked at 0.4.0, and the answers are what this plan is built
from. They are answered first, with evidence, because a plan that starts from a
flattering assessment plans the wrong work.

---

## 1. Is the corpus correct, and aligned with the idea?

**Two different axes, and they are in different states.**

### Arithmetic correctness: strong, and mechanically supported

| evidence | what it covers |
|---|---|
| 486 tests, byte-identical on both integer backends | Rule W — the whole point of having two |
| `xtask` constants harness, two independent derivations | every declared constant, §2.4's invariants |
| conformance vectors re-deriving to a committed digest | that this tree produces what it says |
| differential tests against `hifitime` | the SI bridge |
| certified enclosure containing a float oracle | the quadrature |
| property tests over the full domain | codecs, ordering, tier decomposition |
| `no_std` + no-allocator build | GE-5 |

Nothing found this cycle was an arithmetic defect. Not one.

### Alignment with the idea: weaker, and the leaks say where

Every defect a human found in 0.3.0 and 0.4.0 was an *alignment* defect, and
almost all of them landed on the same rule:

| what was found | rule |
|---|---|
| twelve render sites with an undeclared rounding mode | **R** |
| "years" printed without saying which year | **R** |
| a tick's length in beats printed as `0.000000` | **R** |
| `cosmo age` reporting its widths in Earth years and nothing else | A.5, via **R**'s rendering path |
| foreign units appearing everywhere unasked | A.5, via **R** |

And here is the correlation that matters. Of twenty-four rules, **two were
enforced by convention alone**:

```
Rule F: convention          — the profile carries FRAME and datum prints it
Rule R: convention + review — to_decimal_string is the single rounding path
```

**Rule R is where every one of those defects lived.** The rules enforced by the
type system did not leak. The rule enforced by review did, repeatedly, in a
project whose central argument is that review is not enough — chapter 29 says
so in as many words.

That is not a failure of the idea. It is the idea, observed working on its own
author.

### What 0.4.0 changed about that

Rule R now has mechanism, not convention:

- `Value::quantity` is the only way to render a rational, and a test fails if any
  call site formats a decimal itself;
- `Certification` is *computed* from the value at render time, so a tag cannot
  drift from what the renderer did;
- `Value::Bridge` makes "this is a foreign unit" a property of the value, so
  omitting it is the default rather than something to remember;
- `no_earth_units.rs` asserts no non-Earth command prints a foreign unit unasked;
- `no-indent-in-literal` catches the defect that reached output twice.

**`spec/RULES.md` still describes Rule R as convention-enforced.** Correcting
that is the first deliverable below, and it is not paperwork: the rules file is
what a reader consults to know what is guaranteed.

---

## 2. Is the code stable?

**No, and the number that says so is not the test count.**

| release | breaking |
|---|---|
| 0.1.1 | yes |
| 0.2.0 | none |
| 0.3.0 | yes — three |
| 0.4.0 | yes — `--bridge`, D-A16, one renamed key |

Three of four releases broke something. That is *correct* behaviour for `0.x` and
it is evidence that the surface has not settled.

The second number matters more:

```
ucal-core: 125 downloads, 4 versions
reverse dependencies: 5 — all five are this workspace's own siblings
external users: none
```

**The API has never been in contact with anyone but its author.** Every breaking
change so far cost nothing, which means none of them tested whether the surface
is right. A 1.0 promise made in that state is a promise about a guess — the
release notes have said so since 0.2.0, and it is still true.

---

## 3. What must exist before 1.0

Not a wish list. Each item answers "what would make the 1.0 promise honest".

---

# The plan

Six cycles — five as first written, plus 0.9.0, added in 0.7.0 when the plan was
found to contradict its own exit criteria. Each has one goal, in the shape that
has worked: a stated aim every scope item serves, and a recorded outcome when
something fails.

## 0.5.0 — finish the mechanism Rule R started

**Goal: no rule is enforced by convention alone.**

- **R1 — Rewrite Rule R's entry in `spec/RULES.md`** to record what 0.4.0 built:
  type system (`Value::Quantity` carries a rational, not a string), lint, and
  test. A reader consulting the rules must not be told a guarantee is weaker
  than it is — or stronger.
- **R2 — Sweep for the rest of Rule R.** `Value::quantity` covers the CLI crate.
  `ucal-core`, `ucal-civil`, `ucal-body` and `ucal-cosmo` also render, and
  nothing checks them. Extend the bypass property to every crate, or record why
  it cannot reach them.
- **R3 — Rule F.** The frame is declared and printed and nothing prevents a
  second profile declaring a different one, "which is the point of declaring
  it". Decide whether that is genuinely sufficient — if it is, say so in the
  rule rather than leaving `convention` to read as a gap; if not, give it a
  type.
- **R4 — D2, sign the conformance vectors.** Carried since 0.2.0.
  `verify-vectors` reports `UNSIGNED` honestly. The mechanics are twenty
  minutes; the content is key custody, and a signing key on one laptop with no
  rotation story is a weaker claim than it looks. Decide and record which claim
  is being made.

**Done when** no rule's enforcement line reads `convention` without a sentence
saying why that is the right answer for that rule.

## 0.6.0 — the surface a 1.0 would freeze

**Goal: know exactly what is being promised.**

- **S1 — API audit.** What is `pub` that need not be. `#[non_exhaustive]` on
  every enum and struct a future field could break. `Value` and `Names` got it
  in 0.3.0; nothing else has been reviewed.
- **S2 — Write down what 1.0 means here.** Semver is the floor. This project
  makes stronger claims than "the types will not change" — a released
  `BIG_BANG_CLAIM` cannot acquire operators, a certified enclosure cannot narrow,
  `ucal-json/1` fields cannot change meaning. Those are the promises worth
  freezing, and they are not the ones semver talks about.
- **S3 — MSRV and deprecation policy.** Currently `rust-version = "1.85"` with
  no stated policy for moving it.
- **S4 — Feature-flag audit.** `u512`/`bigint` are mutually exclusive with a
  `compile_error!`; `alloc`/`std`/`hifitime`/`civil`/`body`/`events`/`cosmo`
  interact in ways only the test matrix knows. Enumerate the combinations that
  are supported and fail the rest loudly.

**Done when** a document exists that a stranger could read to know what
upgrading within 1.x will and will not do to them.

## 0.7.0 — contact

**Goal: the surface meets someone who is not the author.**

This is the cycle that cannot be completed alone, and it is the one that decides
whether 1.0 is honest.

- **C1 — One external implementation of the conformance vectors.** Not a user of
  this crate: an independent implementation of UC-1 that reproduces the vectors.
  That is what the vectors and their signature are *for*, and until one exists
  the conformance apparatus has never been used.
- **C2 — GE-A4, the two-reader test.** Carried since 0.2.0. The book's
  dual-audience claim is untested; the protocol is written.
- **C3 — One person using the CLI for something.** Anything. The `--bridge`
  decision, the certification vocabulary and the table layout are all guesses
  about a reader who has never existed.
- **C4 — Record what contact changes.** Expect breaking changes here and treat
  them as the cycle's product rather than its cost. A 1.0 that ships without
  ever having been wrong about its surface has not learned anything.

**Done when** at least one of C1–C3 has happened and its findings are recorded —
including "nothing changed", if that is the finding.

## 0.8.0 — the last breaking window

**Goal: spend the breaking changes 0.7.0 earned, and stop.**

Whatever contact found. Nothing new invented; this cycle exists so that the
findings do not have to wait for 2.0.

**Done when** every finding from 0.7.0 that needs a breaking change has had one,
and nothing is left waiting for the next window. 0.8.0 itself *has* a `Breaking`
section — that is what it is for; the release that must not is 0.9.0.

## 0.9.0 — hold still

**Goal: one full cycle that breaks nothing, before the release that promises not
to.**

Added in 0.7.0 because the plan and its own exit criteria disagreed. The
criteria ask for **two consecutive releases with no breaking change**; the four
cycles above supply at most one, since 0.8.0 is defined as the cycle that
*spends* 0.7.0's breaks. Reading "0.8.0 → 1.0.0" as satisfying it was an
arithmetic error in the gate, and a gate that cannot be met is a gate that will
be waived.

There is nothing to build here, which is the point. A cycle whose diff contains
no `Breaking` section is evidence that the surface has stopped moving, and that
evidence cannot be manufactured faster by working harder — it is exactly the
kind of claim this project insists must have a mechanism, and the mechanism is
elapsed time with people using it.

What may happen in 0.9.0: bug fixes, documentation, added fields, added
commands, performance. What may not: anything that changes a name, a shape, a
meaning, or a default.

**Done when** 0.9.0 has shipped with an empty `Breaking` section, and nothing
found since 0.8.0 is waiting for a breaking change to fix. If something is
waiting, it is spent here and the counter restarts — which costs one cycle and
is the whole reason the counter is two rather than one.

## 1.0.0 — the promise

**Goal: say what is guaranteed, and mean it for as long as 1.x lasts.**

Exit criteria, in the shape §20's phases use:

| criterion | met when |
|---|---|
| no rule enforced by convention alone | 0.5.0 |
| every promise 1.0 makes is written down | 0.6.0 |
| the conformance apparatus has been used by someone else | 0.7.0 C1 |
| the surface has survived contact | 0.7.0 |
| the specification and the source agree, checked | already: `check-docs`, 116 citations |
| both backends byte-identical | already: Rule W, every release |
| two consecutive releases with no breaking change | 0.9.0 → 1.0.0 |
| CI green on every push, with no known-failing job | 0.7.0 — **not 0.5.0, as this table claimed until then.** The `features` workflow added in 0.6.0 failed on every push it ever ran, including the 0.6.0 release tag, because an assertion read colourised cargo output through an anchored `^error`. The criterion was written, nothing checked whether it held, and this document asserted it did — which is the failure this table exists to keep out, occurring in the table itself |

---

## What this plan refuses

Recorded now, while it is cheap.

**1.0 as a marketing event.** The version number is a promise about compatibility
and nothing else. Reaching it because the project feels finished would be the
same category error as reporting a midpoint because an interval is inconvenient.

**Skipping 0.7.0.** It is the only cycle whose success is not in the author's
control, and therefore the only one that can be quietly dropped. If contact
proves impossible, the honest outcome is **1.0 does not ship** and the crates
stay `0.x` — which costs nothing, since `0.x` already permits everything 1.0
would and promises less.

### If contact never comes

Recorded because "never" is a likely outcome for a project of this kind and an
open-ended wait is not a plan. A specification for absolute time in Planck ticks
has a small audience, and nothing in this repository can enlarge it.

Two things follow, and they are separate.

**The contact gate stays shut.** Not softened, not reinterpreted, not satisfied
by the author using the API and reporting that it felt fine. `0.x` forever is an
honest resting state and this project loses nothing by occupying it. A version
number is not a reward for effort.

**The cycles do not stop.** 0.7.0 closes and ships like any other, with its
finding recorded as *no contact yet*; the gate is a condition on **1.0**, not on
releasing. Holding the branch open until a stranger arrives would make the
repository look abandoned to exactly the stranger it is waiting for, which is
the one outcome that makes the wait self-defeating. 0.8.0 and 0.9.0 can run on
their own merits — there is always a defect to fix and a document to correct,
and this cycle found several without any help.

So: releases continue, the 1.0 gate holds, and if contact arrives in 0.9.0 or in
five years, 0.8.0's breaking window reopens then. Nothing about that ordering
expires.

**A 1.0 that freezes the specification.** UCAL-1 is superseded, not finished.
`spec/SPEC-DELTAS.md` has sixteen entries and D-A16 was written this cycle. 1.0
freezes the *API*; the specification keeps its amendment procedure.
