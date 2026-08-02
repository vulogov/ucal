# Pinned state

UCAL-A1 **Rule S**: the source tree at this commit is the sole authority for
every claim the book makes about the software. RFC UCAL-1 describes intentions,
some of which the code did not keep; where they disagree, the code is right and
the RFC is evidence about the past.

| | |
|---|---|
| commit | `f84f157344226f317381b7b6051ed7e950fc6543` |
| short | `f84f157` |
| branch | `0.2.0` |
| date | 2026-07-31 |
| released | 0.1.1 on crates.io (six crates); this tree is 0.2.0, in development |

## Toolchain

| | |
|---|---|
| rustc | rustc 1.94.1 (e408947bf 2026-03-25) |
| cargo | cargo 1.94.1 (29ea6fb6a 2026-03-24) |
| typst | typst 0.14.2 (unknown hash) |

## Test state at this commit

| | |
|---|---|
| tests passing | 381 |
| suites | 22 |
| backends | `u512` (bnum) and `bigint` (num-bigint), both green |
| constants harness | `cargo run -p xtask` — 96/96, two independent routes |
| lints | clean; two declared exemptions, both reported by the tool |
| citations | every `§`, `Rule` and `D-A` resolves against `spec/` |

## Typst packages

Pinned at the import site, so the build fails rather than drifting.

| package | version | used for |
|---|---|---|
| `cetz` | 0.4.2 | figures |

## Reproducing

```
git checkout f84f157
cargo test --workspace --release
cargo run -p xtask
typst compile Documentation/LIFE_UNIVERSE_AND_GOD/BOOK.typ
```
