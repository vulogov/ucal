# Pinned state

UCAL-A1 **Rule S**: the source tree at this commit is the sole authority for
every claim the book makes about the software. RFC UCAL-1 describes intentions,
some of which the code did not keep; where they disagree, the code is right and
the RFC is evidence about the past.

| | |
|---|---|
| commit | `96d673cee03246deb12b00bd52c0ca0317f8a8d2` |
| short | `96d673c` |
| branch | `1.2.0` |
| date | 2026-08-05 |
| released | 1.1.0 on crates.io (six crates); this tree is 1.2.0, in development |

## Toolchain

| | |
|---|---|
| rustc | rustc 1.94.1 (e408947bf 2026-03-25) |
| cargo | cargo 1.94.1 (29ea6fb6a 2026-03-24) |
| typst | typst 0.14.2 (unknown hash) |

## Test state at this commit

| | |
|---|---|
| tests passing | 518 |
| suites | 33 |
| backends | `u512` (bnum) and `bigint` (num-bigint), both green |
| constants harness | `cargo run -p xtask` — 96/96, two independent routes |
| lints | clean; every exemption declared and reported by the tool |
| citations | every `§`, `Rule` and `D-A` resolves against `spec/` (117 distinct) |
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
