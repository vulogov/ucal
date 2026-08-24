# The road to 2.0

Four limitations were frozen by 1.0 and have been carried, unchanged, in every
release note since. This decides what happens to them.

It is built differently from [`ROAD-TO-1.0.md`](ROAD-TO-1.0.md), and the reason
is written first because it is the most useful thing here.

---

## Why there is no contact gate

`ROAD-TO-1.0.md` made contact the gate on 1.0: an external implementation
reproducing the conformance vectors, and a surface that had survived meeting
someone. The gate was set, held for three cycles, and then **overridden**. 1.0.0
shipped with it open.

**That is a failed instrument, and the failure is instructive.** It did not
produce a decision. It produced three cycles of delay and then a reversal —
worse than either shipping on schedule or holding firm, because the delay bought
nothing and the reversal spent the credibility the gate was supposed to have.

The deeper fault is that a gate on something that will not happen at a
meaningful rate is not a decision procedure. This project's standard is that a
claim needs a mechanism; a gate nobody will ever trip is a claim whose mechanism
never runs. It reads as rigour and functions as deferral. A specification for
absolute time in Planck ticks has a slow adoption curve by nature, and gating a
version number on that curve conflates *is the design ready* with *did somebody
arrive*.

The concern underneath it was real — **do not freeze a surface nobody has
tested** — and it is not abandoned. What changed is the instrument. "Tested by a
user" is the wrong proxy when there are no users; and the project has spent five
cycles building better ones: the `ucal-json/1` surface baseline, `semver-checks`
against every published version, the property tests across both backends, the
hostile-input corpus, the fuzzer. Those answer *is this self-consistent*, which
is answerable. They do not answer *is this right*, which without users may be
unanswerable — and a gate does not answer it either.

The three asks in [`CONTACT.md`](../CONTACT.md) remain open and remain wanted.
They are not a condition on anything shipping.

---

## The decision rule

For each limitation, one question: **does it cause a defect, and is the defect
reachable?**

Not *is it inelegant*, and not *would a user complain*. A major version is
justified by a defect that cannot be fixed without one — which is answerable
today, by looking at the code.

---

## The four

### 1. The backends are mutually exclusive — **2.0**

`u512` and `bigint` cannot both be enabled, and a `compile_error!` says so
clearly. The consequence is not clear: two libraries that each chose a backend
cannot appear in one dependency graph. Cargo unifies the features, the guard
fires, and **nothing in the graph builds**.

Demonstrated in 0.9.0 rather than argued: two crates, one on each backend, and
an application depending on both fails with the guard's own message. Neither
library author can fix it; only the end user can, by making one of them change.
It also breaks tooling that assumes features are additive — `cargo semver-checks`
cannot build this crate without `--default-features`.

**Verdict: a reachable defect.** Latent today because there are no dependent
libraries, and permanent once there are.

**The fix, and it is the cheaper of two.** Not generics over the integer type,
which would put a parameter on every public type for a benefit nobody asked for.
**Withdraw `bigint` from the public feature surface.** It exists to verify
`u512` under Rule W — two independent implementations reaching the same answer —
and that is a property of the *test suite*, not a choice a caller should be
making. It offers a caller nothing: the domain is capped at 512 bits either way,
so `bigint` is strictly slower with no added range.

Kept as a build configuration the project's own CI selects, so Rule W is still
verified on every push and the digest in `properties.rs` still has two backends
to agree on. What goes away is a caller's ability to choose one — and with it,
the hazard.

**Cost to a caller:** a documented supported combination disappears.
`ucal-core` with `--features bigint` stops being buildable by an outside caller.
Anyone relying on it loses nothing but speed, and today that is nobody.

### 2. The `&'static str` data model — **2.0**

Every string in the body and citation model is a `&'static str`:
`Citation::new(source: &'static str, ..)` in `ucal-core`, `Body::new`,
`Satellite::new`, `Anchor::new`, `Measured`'s verbatim, unit and quantity —
thirty-four sites, and `Citation` alone appears in nineteen public API positions
across all five crates.

**Verdict: a reachable defect, and the only conformance gap of the four.**
§15.1 requires a strict loader with body files and anchor files versioned
independently, and [D-A20](../../spec/SPEC-DELTAS.md) records that `ucal-body`
does not have one. It cannot: a runtime loader must produce `&'static str` from
owned data, which means `Box::leak`. Bounded and harmless in a process that
exits — which is why 1.4.0's loader lives in the binary — and **unbounded in a
library**, where a caller loading in a loop inherits a leak this crate chose for
them.

So a normative requirement is unmeetable without a breaking change. That is the
strongest case of the four, and it does not depend on anyone showing up: the
specification already says it.

**The fix:** `Cow<'static, str>` through the data model. `Cow` and not `String`
so that the compiled-in tables stay `const`-constructible and cost nothing —
§3.3 requires the profile constants to be `const`, and that must not regress to
buy a loader.

**Cost to a caller:** *measured in 1.8.0 and not what this said.* `Citation::new`
keeps its signature and its `const fn`, so no construction site changes at all.
What a caller loses is `Copy`, on seven types — see the measured diff below.

### 3. `Measured` cannot express a retrograde rotation — **rides along**

Venus rotates retrograde; the fact sheet prints `-5832.6` hours. `Measured`
carries an unsigned mantissa, so the value stored is the magnitude and the sign
lives in a comment.

**Verdict: not a defect on its own.** `venus-d` is correct — it is built from
the published solar day and year, both positive. What is missing is the
*explanation*: a reader asking why Venus's solar day is shorter than its
rotation finds no answer in the data. Measured in 0.8.0: the synodic formula run
with the magnitude gives 2980 days against a published 116.752, wrong by a
factor of twenty-five, and only the sign accounts for it.

This would not justify a major version by itself. It rides along because the
next section explains why nothing should be left behind.

### 4. The no-panic guarantee covers the binary, not the libraries — **probably not 2.0**

`crates/ucal/src` carries no panicking construct and the binary installs a
handler. The libraries beneath carry twenty-two `expect` calls on invariants
they have just established — thirteen of them in `ucal-civil`.

**Verdict: probably not a defect.** Each is a branch no input can reach, and
rewriting them into `Result` would give every caller an error case nothing can
produce, on paths where the alternative to stopping is not a wrong answer but no
answer. A caller who wants the guarantee can catch unwinds at their own
boundary, which is where that decision belongs.

What was actually wrong was the *claim*, and 0.9.0 fixed it: `STABILITY.md` now
says plainly that promise 5 is about the binary and states what a `1.x` release
will still not do — let a **reachable** condition arrive as a panic in any crate.

**This item is closed, not deferred.** It is listed so that a reader comparing
this document to the carried-forward lists can see it was decided rather than
dropped.

---

## If 2.0 happens, all of it happens at once

Items 1 and 2 both change `ucal-core`'s public types. A second major version for
the leftovers would be worse than one for everything — it would spend the same
disruption twice for less.

So 2.0 is a single go/no-go on a bundle: withdraw `bigint` from the public
surface, move the data model to `Cow`, give `Measured` a sign, implement §15.1
in `ucal-body`, and move the body-file loader out of the binary where it now
lives.

Nothing else. A major version is not an invitation to redesign; it is the
narrowest possible way to fix four things that cannot be fixed otherwise.

## When

**When the bundle is worth a release, which is a question about the diff and not
about the calendar.** Two of the four are reachable defects with known fixes, so
the work is justified now. What is not yet decided is whether it is *urgent*,
and the honest answer is no: nothing depends on this library, so the defects are
latent, and the cost of waiting is zero until something does.

That is not a gate. It is the observation that the release can be made whenever
the work is done, and that nothing is made worse by doing the work in a cycle
that has room for it.

### That paragraph was a deferral, and it lasted five cycles

Written above and left standing since 1.4.0. It reads like a schedule and
functions as one only if somebody counts the diff, which nobody did until 1.8.0
— five releases later. [`A2-the-2.0-diff.md`](A2-the-2.0-diff.md) says the rest
plainly, and the shape is the one this document already diagnosed one section
earlier about the contact gate: *"It reads as rigour and functions as deferral."*

The lesson is not that the decision was wrong. It is that **"when the work is
done" needs somebody to have measured the work**, and this document did not ask
anyone to.

### The measured diff (1.8.0)

| item | caller-visible cost | internal cost |
|---|---|---|
| 1 — withdraw `bigint` | one documented feature disappears | ~25 lines across 6 manifests |
| 2 — `Cow` data model | **`Copy` lost on 7 types**; construction unchanged | 22 struct literals, 14 compile errors |
| 3 — `Measured` sign | one constructor argument | 17 sites |
| 4 — §15.1 in the library | none (additive) | 627 lines moved |

**Item 2 is not the rename this document assumed.** `Citation::new` can keep
taking `&'static str` and keep being `const fn`, because `Cow::Borrowed` is a
`const` constructor — so §3.3's const-constructible profile constants survive and
**all twenty-nine call sites compile unchanged**, verified by doing it. What
breaks is `Copy`, on seven types, and that lands on callers rather than here.

The paragraph above says the fix's "cost to a caller" is that "every construction
site changes". That is now known to be false, and the real cost is one this
document did not name.

**Item 1 has three tools asking for it.** `cargo semver-checks` needs
`--default-features`, the MSRV job cannot say `--all-features`, and 1.6.0's audit
wrote around the same wall. 1.8.0 added a `full` feature as the name they were
reaching for, which relieves the symptom without touching the hazard: two
libraries on different backends still cannot appear in one dependency graph.

And item 1 must be executed as one change across all six manifests. Withdrawing
the feature from `ucal-core` alone makes `cargo metadata` fail outright, because
siblings request `ucal-core/bigint` — found while probing, in A3.

### A path this document does not list

A2 surfaced it: **give the loader an arena or an interner** rather than changing
the data model. §15.1's obstacle is that a runtime loader must produce
`&'static str` and therefore leaks; interning bounds the leak by *distinct
content* instead of by call count. That would address item 4 without item 2, and
it is additive.

Whether it is enough to move D-A20 from `UNIMPLEMENTED` is a separate question —
interning does not make a loaded `Body` droppable, so it weakens the objection
rather than removing it.

### And the tool can be relied on

A2's number depends on `cargo semver-checks` being right about what breaks, and
V1 Finding 6 established that it is not always.
[`A3-semver-probes.md`](A3-semver-probes.md) probed the six categories a 2.0 of
this crate would touch — a removed feature, a removed derive, a changed field
type, an added argument, and a variant added to each of a `#[non_exhaustive]` and
a closed enum — against what a real downstream crate does. **Six agreements, no
second blind spot.**

## What 2.0 does not do

**It does not freeze the specification.** UCAL-1 keeps its amendment procedure,
as 1.0 did. Twenty standing deltas and counting.

**It does not revisit the six promises.** `STABILITY.md`'s promises survive a
major version; what changes is the shape of the types they are made about.

**It does not become an occasion.** Everything not on the list above stays as it
is, including the things that are merely inelegant.

## If 2.0 never happens

Then the four items become permanent features of `1.x`, `STABILITY.md` carries
them as the shape of the thing, and §15.1 stays `UNIMPLEMENTED` with `D-A20`
saying so.

That costs little and should be said without drama: `1.x` already does
everything `2.x` would, minus four caveats, two of which bind only callers who
do not exist. It is an honest resting state, in the way `0.x` forever was, and
naming it here means the choice to stay is a choice rather than a drift.
