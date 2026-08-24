# A2 — the 2.0 diff, measured

**Status: measured. The numbers below come from running the migration far enough
to count and then reverting it, not from reading the code.**

---

## Why measure now

`ROAD-TO-2.0.md` decided all four frozen limitations and then said when:

> **When the bundle is worth a release, which is a question about the diff and
> not about the calendar.** [...] That is not a gate. It is the observation that
> the release can be made whenever the work is done.

Five cycles later nobody had looked at the diff. "A question about the diff" with
nobody counting the diff is a claim with no mechanism, in the document that
schedules the project's only planned breaking change.

## Method

For item 2 — the substantial one — the migration was **performed**: `Citation`'s
fields changed to `Cow<'static, str>`, `ucal-core` brought back to compiling, and
the resulting errors counted across the workspace. Then reverted.

A count of `grep` hits would have been easier and would have missed the finding
below entirely.

---

## Item 1 — withdraw `bigint` from the public feature surface

| | |
|---|---|
| feature declarations to edit | **7** (one per crate, plus the optional deps) |
| `cfg(feature = "bigint")` sites in source | **7** |
| CI lines naming it | **11** |
| documentation files mentioning it | **29** |
| **caller-visible breakage** | one documented feature disappears |

The code cost is trivial: the feature stays, as a build configuration CI selects,
so Rule W still has two backends to agree on. What goes away is a *caller's*
ability to select it — and today no caller exists.

**This is the cheapest of the four and the one with the most evidence behind it.**
Three tools now want something the feature surface cannot give them:
`cargo semver-checks` needs `--default-features`, the MSRV job needs
`--features tui` spelled out, and 1.6.0's audit had to write around
`--all-features` entirely.

---

## Item 2 — the `&'static str` data model

The one the whole bundle is really about, and **the measurement changed what it
is.**

### What was expected

Thirty-four construction sites, `Citation` in nineteen public API positions. A
large but mechanical rename.

### What it actually is

`Citation` **derives `Copy`**, and `Cow` is not `Copy`. That is not a rename; it
is a change in how the type moves.

| | |
|---|---|
| `Citation::new` call sites needing an edit | **0** |
| struct literals needing an edit | **22** |
| types that lose `Copy` transitively | **7** |
| workspace errors after `ucal-core` compiles again | **14** |
| of those, `cannot move out of ... behind a shared reference` | **6** |

The seven types that lose `Copy`: `Citation`, `Provenance`, `MeasuredValue`,
`Measured`, `Determination`, `Meridian`, `PhaseDefinition`. (Six was the first
count written here, from the five found by grep plus `Citation`; `Provenance`
turned up only when the compiler reached it, which is the argument for running
the migration rather than reading it.)

### The good news, and it is substantial

**`Citation::new` does not have to change at all.** It can keep taking
`&'static str` and keep being `const fn`, because `Cow::Borrowed` is a `const`
constructor:

```rust
pub const fn new(source: &'static str, locator: Option<&'static str>) -> Citation {
    Citation { source: Cow::Borrowed(source), locator: /* ... */ }
}
```

So §3.3's const-constructible profile constants survive, and **every one of the
twenty-nine `Citation::new` call sites compiles unchanged** — verified, not
assumed. What breaks is the twenty-two struct literals, which are internal, and
the `Copy`.

### The real cost is `Copy`, and it lands on callers

A downstream crate that today writes

```rust
let c = anchor.citation();     // Copy: a move out of a shared reference
```

writes `.clone()` after 2.0. Six such sites exist inside this workspace; a
caller's count depends on how they hold these types, and there is no way to
measure that from here.

**That is a different kind of breakage from a rename.** A rename is found by the
compiler and fixed mechanically. Losing `Copy` changes the ergonomics of every
type that carries a citation — which, in a project whose central rule is that
every measured quantity carries its provenance, is most of the data model.

### The option nobody has considered

The 14 errors are small enough that a **third path** is worth naming: keep
`Citation` as it is, and give the *loader* an arena. §15.1's problem is that a
runtime loader must produce `&'static str` and therefore leaks; an arena owned by
the loader and dropped with it would bound the leak without touching a single
public type.

That is not on `ROAD-TO-2.0`'s list, and it would resolve item 4 without item 2.
Recorded here rather than argued: it is a design proposal and this page is a
measurement.

---

## Item 3 — `Measured` cannot express a retrograde rotation

| | |
|---|---|
| `Measured::new` call sites | **17** |
| public methods on `Measured` | **5** |
| bodies affected today | **1** (Venus) |

Adding a sign is a field and a constructor argument. `Measured` is one of the seven
types that lose `Copy` in item 2, so if the two ship together this costs almost
nothing beyond what item 2 already costs.

ROAD-TO-2.0 says this "rides along" and the number supports that.

---

## Item 4 — §15.1 in the library

| | |
|---|---|
| lines to move | **627** (`body_file.rs` 400, `anchor_file.rs` 227) |
| public items added to `ucal-body` | 2 loaders |
| depends on | item 2, or on the arena above |

The move itself is mechanical. What it depends on is the `&'static str` problem,
which is item 2 — or the arena, which is neither.

---

## The total

| item | caller-visible cost | internal cost |
|---|---|---|
| 1 — withdraw `bigint` | one feature disappears | ~25 lines |
| 2 — `Cow` data model | **`Copy` on 7 types**; construction unchanged | 22 literals + 14 errors |
| 3 — `Measured` sign | one constructor argument | 17 sites |
| 4 — §15.1 in the library | none (additive) | 627 lines moved |

**The bundle is smaller than five cycles of deferral implied**, which is itself
the finding. A2's stop condition anticipated it:

> **Stop if** the measurement shows the bundle is smaller than expected — under a
> few dozen sites — in which case the interesting question becomes why it was
> deferred five times.

It is under a few dozen sites. So: why?

## Why it was deferred five times

Not because it is large. Because **nothing forced anyone to look.**
`ROAD-TO-2.0`'s "when the work is done" reads like a schedule and functions as a
deferral — the same failure the contact gate had, and the same one the document
itself diagnosed about *that*:

> a gate on something that will not happen at a meaningful rate is not a decision
> procedure [...] It reads as rigour and functions as deferral.

The document diagnosed the pattern and then reproduced it one section later. That
is worth recording plainly, because it is the third time this project has found
the same shape — a claim with no mechanism, a mechanism with no wire, and now a
schedule with no date.

## What this page does not decide

Whether to cut 2.0. That needs the number and now has it; the decision is the
author's and belongs in `ROAD-TO-2.0.md`, not here.

The one thing the measurement argues for on its own is **sequencing**: item 1 is
nearly free and has three independent tools asking for it, and it does not depend
on item 2. If 2.0 happens for one reason, that is the reason.
