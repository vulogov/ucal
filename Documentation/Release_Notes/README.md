# Release notes

One file per version, named for the version: `0.1.0.md`, `0.1.1.md`, `0.2.0.md`.

The file for the version under development is created the moment the cycle opens
and is marked **UNRELEASED** at the top. It accumulates entries as work lands,
rather than being reconstructed from the log on release day — a note written
alongside the change knows why the change was made, and one written a month later
knows only what it did.

## Contents

| version | date | state |
|---|---|---|
| [1.11.0](1.11.0.md) | — | **unreleased** — the standard that stops at the edge of one build |
| [1.10.0](1.10.0.md) | 2026-08-28 | released — a requirement with no wire |
| [1.9.0](1.9.0.md) | 2026-08-27 | released — the artefact, not the repository |
| [1.8.0](1.8.0.md) | 2026-08-24 | released — the frozen four, measured |
| [1.7.0](1.7.0.md) | 2026-08-22 | released — does each check catch what it exists for |
| [1.6.0](1.6.0.md) | 2026-08-20 | released — the mechanism without a wire |
| [1.5.0](1.5.0.md) | 2026-08-11 | released — spend what the research found, and build a clock |
| [1.4.0](1.4.0.md) | 2026-08-10 | released — decide 2.0 before it is forced |
| [1.3.0](1.3.0.md) | 2026-08-09 | released — find the defects nobody is going to report |
| [1.2.0](1.2.0.md) | 2026-08-08 | released — the questions a stranger asks first |
| [1.1.0](1.1.0.md) | 2026-08-07 | released — make it cheap to try |
| [1.0.0](1.0.0.md) | 2026-08-06 | released — **the promise, made**, with the contact gate open |
| [0.8.0](0.8.0.md) | 2026-08-05 | released — the last breaking window, spent on additions |
| [0.7.0](0.7.0.md) | 2026-08-05 | released — the asks made specific, then the waiting |
| [0.6.0](0.6.0.md) | 2026-08-05 | released — know exactly what 1.0 would freeze |
| [0.5.0](0.5.0.md) | 2026-08-05 | released — no rule enforced by convention alone |
| [0.4.0](0.4.0.md) | 2026-08-04 | released — every number says what it is |
| [0.3.0](0.3.0.md) | 2026-08-03 | released — legibility |
| [0.2.0](0.2.0.md) | 2026-08-01 | released — supersede RFC UCAL-1 |
| [0.1.1](0.1.1.md) | 2026-07-31 | released |
| [0.1.0](0.1.0.md) | 2026-07-31 | released |

## What a release note is for

The audience is somebody deciding whether to upgrade, and what will break if
they do. So each file leads with what changed for *them*, not with what was
done to the repository.

Every entry says three things:

1. **What changed** — in the caller's terms, not the implementation's.
2. **Whether it breaks anything** — explicitly, including "no".
3. **Why**, when the why is not obvious. A version bump needs no reason; a
   changed default does.

Sections, in this order, omitting any that are empty:

```
Breaking            what will stop compiling or stop behaving as it did
Added               new capability
Changed             different behaviour that is not breaking
Fixed               a defect, with what it did wrong
Verification        what was run before this shipped
Internal            visible in the repository, invisible to a caller
```

**Breaking goes first and is never softened.** While the crates are `0.x`, a
minor bump is permitted to break the API, and a reader has no way to tell a
safe upgrade from an unsafe one except by being told.

## Two rules that follow from the project's own

**Do not claim what was not measured.** If a note says something is faster, a
number and the conditions belong with it. Rule X's habit of reporting an
enclosure rather than a point estimate applies to prose about performance too.

**Record what was refused.** When an experiment's kill criterion fires, or a
proposed change is dropped after being tried, that belongs in the notes. The
reasoning is the expensive part, and it is lost if only the outcome is written
down. `spec/SPEC-DELTAS.md` is the long-form record; a release note carries the
one-line version and links to it.

## Closing a cycle

1. Replace **UNRELEASED** with the date, and check every entry still describes
   what shipped — including **Breaking**. 0.3.0 opened with that section reading
   "none yet" and closed with three entries, none of which had been noticed
   until it was checked rather than assumed.
2. Add the row to the table above.
3. Bump the workspace version. Since 0.3.0 this is one edit to one file: the
   five internal requirements live in `[workspace.dependencies]` and the members
   inherit them, and `xtask -- lint`'s `version-lockstep` fails if they drift
   apart.
4. `cargo test --workspace --release`, both backends, plus
   `RUSTFLAGS="-D warnings" cargo build --workspace --all-targets --release`
   and `cargo run -p xtask -- lint`, `check-docs`, `verify-vectors`.

   **`--all-targets` matters.** Without it `cargo build` compiles the libraries
   and binaries and never touches a test file, so an unused import in a test
   passes locally and fails in CI — which is exactly what happened on the first
   CI run, because the workflow sets `RUSTFLAGS` globally and `cargo test`
   inherits it.
5. `cargo +nightly fuzz run parse_instant fuzz/corpus/parse_instant -- -max_total_time=600`
   and the other two targets, if anything touched a parser.

   Not in CI, for the reason `fuzz/README.md` gives: a fuzz job either has a
   time budget and proves less as the code grows, or it has none and is not a
   CI job. Anything it finds goes into the committed corpus, and what has been
   run is recorded in that README rather than summarised as "fuzzed".

6. `cargo run -p xtask -- check-links`, which asks the network whether the
   cited URLs still reach the cited documents.

   **Deliberately not in the verification block above**, and not on `push`.
   Every other check here is offline and deterministic; this one depends on
   third-party servers, and a check that turns the tree red because somebody
   else is having a bad morning trains its reader to ignore it. "CI green on
   every push, with no known-failing job" is a 1.0 exit criterion, and the way
   the last false criterion survived a whole release was nobody reading a red
   job.

   **Since 1.9.0 it also runs on a schedule** —
   [`.github/workflows/links.yml`](../../.github/workflows/links.yml), 07:00 UTC
   every Monday, plus `workflow_dispatch`. A scheduled job has neither problem:
   it cannot block a push and it fails on its own account. A failure **opens an
   issue**, because a job that fails into the Actions tab is a mechanism with
   nothing attached to it. Running it here at release time is still worth doing
   — a release should not go out on a week-old answer — but the release is no
   longer the only time anybody asks.

   A `MOVED` result matters as much as a `FAIL`. The citation that prompted
   this check answered **`200 OK`** — `nssdc.gsfc.nasa.gov/planetary/factsheet/`
   redirects to a general NASA page, which serves a perfectly good document that
   is not the one being cited. A status check alone would have passed it.

   It cannot tell you the page still *says* what was cited. Nothing mechanical
   can.

7. `cargo semver-checks check-release --default-features`, against the last
   published version.

   `--default-features` is not a preference: without it the tool enables every
   feature at once, trips the `u512`/`bigint` guard, and cannot build the crate.

   **From 1.1.0 it is load-bearing.** Under `0.x` a minor bump already permitted
   breaking changes, so the lints were skipped and the run passed on any diff;
   with 1.0.0 as a baseline it runs 196 checks per crate and a failure means the
   release is not a minor one. It is the mechanism for the semver floor, which
   was the one promise in `STABILITY.md` with none.

8. `cargo run -p xtask -- publish` for the dry run, then
   `cargo run -p xtask -- publish --execute` for real.

   It derives the order from the dependency graph rather than repeating a list,
   packages every crate before uploading any, refuses on a dirty working tree,
   and then publishes one crate at a time.

   The two cargo invocations are not symmetric, and the asymmetry is the whole
   procedure. Packaging **must** use `--workspace`: in workspace mode cargo
   knows all six versions are going out together and resolves the internal
   requirements against the local tree, where per-crate it resolves
   `ucal-core = "^0.3.0"` against the registry and fails, because that version
   is exactly what is about to be uploaded. Publishing **must not** use
   `--workspace`, for the reason below.

   Not `cargo publish --workspace`. It fails on this workspace with a cargo
   internal error — `no hash listed for ucal-core` — because it tries to verify
   a dependent against a dependency that is not on the index yet. Publishing
   sequentially is not a workaround that skips verification: each crate is
   verified normally, against the real index, once the one below it is live.

   Dev-dependencies constrain the order too. `ucal-cosmo` needs `ucal-events`
   only for its float oracle, and cargo still resolves it when verifying the
   package — so the edge is real even though nothing in the shipped code uses
   it.
9. Tag `vX.Y.Z`, annotated and signed, and push the tag.

   Pushing the tag triggers `.github/workflows/release.yml`, which builds `ucal`
   for five targets, runs `ucal verify` on each artefact before packaging it,
   and attaches them to the release with a `SHA256SUMS.txt`.

   If a runner outage eats the run, re-dispatch it with the tag rather than
   moving the tag.

10. `cargo run -p xtask -- sign-release X.Y.Z`, once that workflow has finished.

    **This step needs you, and cannot be given to CI.** The minisign secret key
    is on one laptop with an offline backup and must never enter this repository
    or a runner; a signature CI could produce would attest to a GitHub secret,
    which is a different and weaker claim wearing the same shape.

    The command downloads the checksum file the workflow attached, signs it with
    a trusted comment naming the tag, **verifies the signature against
    `fixtures/ucal.pub` before uploading anything**, and attaches it. It refuses
    a checksum file naming no files, and refuses to re-sign one that already has
    a signature.

    Through 1.8.0 this step did not exist, and nine release notes carried the
    same sentence about it. C1 closed the half of that sentence which was about
    this repository; the half about the world — *the key has no authority behind
    it that is not the author* — is not closable from here.

11. `cargo run -p xtask -- verify-release X.Y.Z`.

    Three comparisons, none of which existed before 1.9.0: the published
    `.crate` files against a checkout of the tag, the release binaries against
    `SHA256SUMS.txt`, and that file against its signature.

    It checks the tag out into a temporary worktree rather than trusting the
    working tree — the first run of it was made from a 1.9.0 tree against
    published 1.8.0 crates and reported five failures, every one of them cargo
    correctly refusing to resolve `^1.9.0` against an index that does not have
    it yet.

    A comparison that could not be performed is reported as `--` and exits 5,
    not 0. A check that could not run has not passed.

12. Open the next file.
