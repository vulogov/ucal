# Pinned state

UCAL-A1 **Rule S**: the source tree at this commit is the sole authority for
every claim the book makes about the software. RFC UCAL-1 describes intentions,
some of which the code did not keep; where they disagree, the code is right and
the RFC is evidence about the past.

| | |
|---|---|
| commit | `425bb3aa5111bf70e5c0044ad11bcd57ead31f38` |
| short | `425bb3a` |
| branch | `main` — the merge commit of the 1.12.0 cycle |
| date | 2026-08-31 |
| released | the tree declares 1.12.0 and its release note is dated 2026-08-31; the local clone carries no `v1.12.0` tag, so the release state is taken from the note and was not verified from here |
| previously | `f9e5b81` (1.8.0, 2026-08-24), re-pinned 2026-11-01 |

## Toolchain

| | |
|---|---|
| rustc | rustc 1.95.0 (59807616e 2026-04-14) |
| cargo | cargo 1.95.0 (f2d3ce0bd 2026-03-21) |
| typst | typst 0.15.1 (unknown hash) |

## Test state at this commit

Measured on 2026-11-01 in a detached worktree at the commit, except where a row
says otherwise.

| | |
|---|---|
| tests passing | 765 on the default backend; 702 on `bigint`; 340 in `ucal` built with `--features full` |
| suites | 52 test binaries report on the default backend |
| backends | `u512` (bnum) and `bigint` (num-bigint), both green |
| constants harness | `cargo run -p xtask` — 96/96, two independent routes, exit criterion met |
| lints | 0 violations across 90 files; every exemption declared and reported by the tool |
| defect corpus | 24 recorded mutations, none surviving — as the 1.12.0 release note records; not re-run for this pin |
| citations | every `§`, `Rule` and `D-A` resolves against `spec/` (137 distinct, documentation included) — `check-docs`, run on a tree whose `spec/` and `xtask/` are byte-identical to the commit |
| spec deltas | 25 recorded, 24 standing and applied to the normative text |
| MSRV | 1.88, as declared; verified by CI per the release note, not re-run here |
| CLI | no invocation panics; every rejection is a §19.5 exit code and a sentence (the `hostile_input` suite, inside the count above) |

## Typst packages

Pinned at the import site, so the build fails rather than drifting.

| package | version | used for |
|---|---|---|
| `cetz` | 0.4.2 | figures |

## Reproducing

```
git checkout 425bb3a
cargo test --workspace --release
cargo run -p xtask
typst compile Documentation/LIFE_UNIVERSE_AND_GOD/BOOK.typ
```
