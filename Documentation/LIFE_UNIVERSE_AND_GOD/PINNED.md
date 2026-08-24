# Pinned state

UCAL-A1 **Rule S**: the source tree at this commit is the sole authority for
every claim the book makes about the software. RFC UCAL-1 describes intentions,
some of which the code did not keep; where they disagree, the code is right and
the RFC is evidence about the past.

| | |
|---|---|
| commit | `f9e5b811baeae03e3b0c0249cbec5931bd900c48` |
| short | `f9e5b81` |
| branch | `1.8.0` |
| date | 2026-08-24 |
| released | 1.7.0 on crates.io (six crates); this tree is 1.8.0, in development |

## Toolchain

| | |
|---|---|
| rustc | rustc 1.94.1 (e408947bf 2026-03-25) |
| cargo | cargo 1.94.1 (29ea6fb6a 2026-03-24) |
| typst | typst 0.14.2 (unknown hash) |

## Test state at this commit

| | |
|---|---|
| tests passing | 609 on the default backend; 221 in `ucal` built with `--features full` |
| suites | 47 |
| backends | `u512` (bnum) and `bigint` (num-bigint), both green on the same Rule W digest |
| constants harness | `cargo run -p xtask` — 96/96, two independent routes, and it now refuses to meet its exit criterion below 60 checks |
| lints | clean across 77 files; every exemption declared and reported by the tool |
| defect corpus | `cargo run -p xtask -- corpus` — 22 recorded mutations, one per check, no survivors |
| citations | every `§`, `Rule` and `D-A` resolves against `spec/` (136 distinct, and since 1.7.0 the documentation is scanned too) |
| spec deltas | 25 recorded, 24 standing and applied to the normative text |
| MSRV | 1.88, verified on the workspace and on `--features full` |
| CLI | no invocation panics; every rejection is a §19.5 exit code and a sentence |

## Typst packages

Pinned at the import site, so the build fails rather than drifting.

| package | version | used for |
|---|---|---|
| `cetz` | 0.4.2 | figures |

## Reproducing

```
git checkout 96d673c
cargo test --workspace --release
cargo run -p xtask
typst compile Documentation/LIFE_UNIVERSE_AND_GOD/BOOK.typ
```
