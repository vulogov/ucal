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
| [0.2.0](0.2.0.md) | — | **unreleased** |
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
   what shipped.
2. Add the row to the table above.
3. Bump the workspace version and the internal dependency requirements
   together — they move in lockstep, so a `0.2.0` facade never resolves against
   a `0.1.x` core.
4. `cargo test --workspace --release`, both backends, plus
   `cargo run -p xtask -- lint` and `check-docs`.
5. `cargo publish --workspace --dry-run`, then for real.
6. Tag `vX.Y.Z`, annotated, and push the tag.
7. Open the next file.
