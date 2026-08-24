# A3 — is there a second blind spot?

**Status: probed. Six categories, six agreements. The one known failure remains
the only one found.**

---

## Why this had to come with A2

A2 measured the 2.0 diff and its number rests on `cargo semver-checks` telling
the truth about what breaks. V1 Finding 6 established that it does not always:
a required method added to a public trait breaks every outside implementor and
the tool reports `no semver update required`.

X3 covered that one failure with four compile-pass fixtures and said plainly what
it did not:

> Whether there is a second exception is unmeasured, and finding out would mean
> building the same kind of external probe for every category of breaking change.

A2's number is only as good as the tool that produces it, so this is that.

## Method

The same one that found the first failure, and the only one that answers the
question: **an actual downstream crate**, compiled against the workspace, using
the surface a 2.0 would touch — a public field read, a `Copy` move out of a
shared reference, an exhaustive match on a closed vocabulary, a call at a
function's current arity.

For each category: apply the change, ask the **compiler** whether the outsider
still builds, ask **`cargo semver-checks`** what it thinks, and compare. A false
negative is *outsider breaks* paired with *no semver update required*.

Reading the tool's lint list would have been faster and would not have answered
the question, because the first failure was a lint that exists, runs, and reports
`PASS`.

## The results

| category | 2.0 item | outsider | `semver-checks` | verdict |
|---|---|---|---|---|
| baseline, no change | — | compiles | no update required | agree |
| a public **feature removed** | item 1 | compiles¹ | **requires new major** | agree |
| a **derive removed** (`Copy`) | item 2 | **breaks** | **requires new major** | agree |
| a public **field's type changed** | item 2 | **breaks** | **requires new major** | agree |
| an **argument added** to a public fn | item 3 | **breaks** | **requires new major** | agree |
| a variant added to a **`#[non_exhaustive]`** enum | — | compiles | no update required | agree |
| a variant added to a **closed** enum (`Kind`) | — | **breaks** | **requires new major** | agree |

¹ The probe crate does not select `bigint`, so it keeps compiling; the tool
flags the removal anyway, which is the correct and stricter answer.

**Six categories, six agreements. No second blind spot found.**

## What the probing cost, which is the useful part

Two of the probes could not be run naïvely, and both obstacles are worth
recording because they will recur:

**A feature cannot be removed from one crate.** Renaming `bigint` in `ucal-core`
alone made `cargo metadata` fail — sibling crates request `ucal-core/bigint`, so
resolution dies before any analysis runs. The probe only works when the feature
is withdrawn from all six manifests at once, which is what item 1 would actually
do. That is a fact about how item 1 must be executed, not just measured.

**A `Copy` removal does not compile in isolation.** Dropping `Copy` from
`Citation` breaks `Provenance` and `MeasuredValue`, which derive `Copy` and carry
one. `ucal-core` had to be made to compile again before the tool could produce a
verdict at all — the same wall A2 hit, and the reason A2's measurement is a
migration rather than a grep.

## What this does not establish

**Six categories are not all categories.** Lifetimes, generic bounds, trait
object safety, `#[repr]` changes, auto-trait leakage — `Send`/`Sync` — and
default type parameters are untested. The six chosen are the ones a 2.0 of *this*
crate would touch, which is the question A2 needed answered, not the general one.

**One agreement is not a guarantee.** The tool flagged all six correctly on
`ucal-core`; nothing here says it would on a differently-shaped crate.

So the honest position is unchanged in structure and narrower in doubt:
`cargo semver-checks` runs on every push and is trusted, with **one** known
failure — required trait methods — covered by four fixtures. The categories a 2.0
would touch have now been checked and it handles all of them.

## The one-line answer for A2

A2's number can be relied on. Nothing in the measured bundle falls into a
category the tool gets wrong.
