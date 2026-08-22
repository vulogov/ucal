# X1 — a defect corpus: one recorded mutation per check

**Status: built, run, and wired. `cargo run -p xtask -- corpus`, in CI since
1.7.0. Twenty mutations, twenty caught — the one survivor X1 found was fixed by
X2, below.**

---

## The question

1.6.0 established that every check in this repository **reads its subject**.
Fourteen could previously report success having examined nothing; none can now.
The audit closed by naming the property it had not established:

> Whether a check is *correct* — whether it would catch the defect it exists for
> — is not answerable by pointing it at an empty directory.

A check that provably reads its subject is not a check that would object to a
wrong one. Those are different properties, and only the first had a mechanism.

## Why a corpus rather than more hand-checking

Six checks were verified strict by hand across 1.5.0 and 1.6.0: inject the
defect, watch it fire, revert. It works and it is convincing, and it is a
**procedure** — done when someone remembers, on whichever check they are
editing, and the evidence gone when the terminal closes.

This project's argument about every other claim applies to its own verification
habits. A procedure is not a mechanism.

## How a mutation runs

Every check takes a `root`. That is what makes this possible: the corpus builds a
**sandbox** — a copy of the files the checks read, and nothing else — mutates one
file in it, and calls the check against that root. The real tree is never edited,
so an interrupted run cannot leave the repository mutated.

Four verdicts, and three of them are failures:

| | |
|---|---|
| `ok` | the check rejected the mutated tree |
| `SURV` | it accepted it — the check passes on a tree wrong in exactly the way it exists to prevent |
| `UNAPP` | `find` did not occur, so nothing was mutated and nothing was tested |
| `DIRTY` | the check was already failing on the *unmutated* sandbox, so its verdict means nothing |

`UNAPP` and `DIRTY` exist because a corpus that silently tests nothing is the
defect this whole line of work is about. Both fire in practice — see below.

## The finding: a real survivor

**`citations` does not read the documentation.**

The check is announced as `citations resolve against spec/ (126 distinct)`, and a
reader takes that to mean every `§` and `Rule` reference in the project resolves.
It does not. `cited()` walks `crates/**/*.rs` and `xtask/src/**/*.rs` and reads
**no Markdown at all** — so a dangling section reference in `Documentation/`,
`spec/`, or a release note is invisible to it.

Demonstrated: a section reference to a number the specification does not define,
planted in `Documentation/CLI.md`, passes. The same reference planted in
`crates/ucal-core/src/tier.rs` is caught.

(Written without the marker on purpose. Spelled out, this paragraph would itself
be a dangling citation once the scan is widened — which is what happened to the
corpus's own source, one section above.)

This matters more than it might look. The documentation is where citations are
*dense* — CLI.md alone carries dozens — and it is the surface a reader checks
against the spec. The check covers the place where a citation is least likely to
be written by hand.

### Fixed by X2

`cited()` now walks `Documentation/`, `spec/` and `docs/` as well, and reads
`.md` alongside `.rs`. The check went from 126 distinct citations to **136**, and
found **two dangling references that had never been seen**:

- **A reference to section 29, cited twice**, in `spec/RULES.md` and in the 0.5.0
  release note. There is no section 29; the specification ends at §24. It had stood for four
  releases — in a sentence whose entire point is *that review is not enough, and
  the rule left to review is the one that failed*. The argument it was reaching
  for is §21.3's, and it now cites that.
- **A section reference in this file**, planted by the paragraph above describing
  the survivor.

**The fix has a cost worth naming: a broken citation can no longer be quoted.**
Writing one out makes the page carrying the explanation into a second instance of
the defect. That happened **four** times while fixing this — in the corpus
source, in `RULES.md`, in this file's survivor paragraph, and then once more in
the paragraph immediately above, written after the cost had been described and
in the sentence describing it.

Prose about citations now has to name them rather than show them. The check
caught every one of the four, which is the argument for it in miniature.

That is small against what it buys. Forty-three Markdown files carrying 502
citation sites had never been checked, and the documentation is where a citation
is written by hand rather than by someone with the spec open.

## Four mistakes the corpus made about itself

Worth recording, because three of the four were mine and the fourth is a finding.

**1. The corpus planted a defect in the tree under test.** The first citation
mutation contained a literal section marker in its `replace` string. `xtask/src/`
is scanned by the citation check, so the corpus's own source became a dangling
citation and `check-docs` failed on the real repository. The marker is now written
as a `\u{a7}` escape — an escape in the source, the right character at runtime.

**2. A mutation aimed at the wrong check.** `cli-docs` was given a renamed
*field*; it checks the *command* surface. Renamed a command instead.

**3. A mutation that matched nothing.** `public-type-is-classified` reads
`pub struct ` as a line prefix at column zero, and the injection was indented
inside an `impl`. Reported `UNAPP` rather than passing, which is why that verdict
exists.

**4. `version-lockstep` does not read member manifests.** A mutation that
hardcoded `version = "1.6.0"` into `crates/ucal-core/Cargo.toml` survived, and
the lint is not at fault in the way it first appeared: it reads the root
manifest's `[workspace.dependencies]` and nothing else. That is a narrower scope
than the name suggests, and a member crate that stopped inheriting
`version.workspace = true` would not be caught.

### Also fixed by X2

The lint now reads each member's `[package]` section and requires
`version.workspace = true`, not merely a matching number.

**Which case matters is not the obvious one.** A member pinned to a *stale*
version — `1.6.0` while the workspace is on `1.7.0` — never reaches any tool:
`ucal-civil` requires `1.7.0`, cargo's resolution fails, and nothing builds. The
case only this lint can see is a member pinned to the version the workspace is
**already on**. That builds perfectly, publishes correctly, and drifts silently
at the next bump.

The corpus carries both paths — the root manifest's dependency table and a member
`[package]` — because a corpus covering one of them would report the check as
tested while half of it was not, which is the shape of claim this exercise exists
to refuse.

## What is not in the corpus, and why

**The UC-P0 constants harness.** Its ninety-six checks are hardcoded calls, not a
reading of the tree, so a mutation is a source edit to `xtask` itself rather than
a reversible edit to a data file. X1's stop condition anticipated this. It is
hand-verified: raising its floor above the live count makes it fail, which was
run in 1.6.0.

**`check-links`.** It makes network requests and is deliberately not in CI. A
mutation would be an edit to a cited URL, and the verdict would depend on
somebody else's web server.

**`cargo semver-checks`.** Not ours, and V1 Finding 6 already records what it
misses. X3 covers it.

**`verify-vectors`.** Reads its manifest from `workspace_root()` rather than a
parameter, so it cannot be pointed at a sandbox without a refactor. Listed as
hand-verified; the refactor is a candidate for X2.

## The count

| | |
|---|---|
| mutations | 21 |
| caught | 21 |
| survivors | 0 (the one X1 found was fixed by X2) |
| checks with a corpus entry | 17 of 21 mechanisable (one has two, for its two paths) |
| checks hand-verified instead | 3 (harness, `check-links`, `verify-vectors`) |
| checks covered elsewhere | 1 (`semver-checks`, V1 Finding 6 / X3) |

The survivor rate is much lower than 1.6.0's hollow-check rate, and that is worth
stating plainly rather than treated as a triumph: **the checks that exist mostly
work.** What 1.6.0 found was that several of them were not connected to anything,
which is a different failure and was much more common.

## Wired into CI

X1 deliberately did not wire it: the corpus exits 6 while a survivor stands, and a
known-red job is how a red build becomes background noise. With the survivor
fixed, X2 added it to the `checks` job, after the release binary is built — the
worked-examples entry needs one, and without it that mutation reports `DIRTY`
rather than a verdict.
