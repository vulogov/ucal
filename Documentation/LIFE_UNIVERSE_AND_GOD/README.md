# Life, the Universe, and God

*A Software Engineer's Instrument for the Immeasurable*

A book about `ucal`, written to RFC UCAL-A1. Typst source; the compiled artifact
is `BOOK.pdf`.

## Build

```
typst compile Documentation/LIFE_UNIVERSE_AND_GOD/BOOK.typ
```

Requires Typst 0.14 or newer and network access on first build, to fetch
`@preview/cetz:0.4.2` from Typst Universe. The build is warning-free.

## Layout

| path | contents |
|---|---|
| `BOOK.typ` | master file; the eight parts and their chapters, in order |
| `design.typ` | design tokens and page chrome — SLOT-VOICE, realised |
| `slots.typ` | Rule L: author-supplied material, and the assertions that fail the build without it |
| `chapters/` | one file per chapter |
| `assets/logo/` | identity assets, per `Documentation/logo/README.md` |
| `PINNED.md` | Rule S: the commit every claim is checkable against |

## Two rules you can see on the page

**Rule S** — the source tree at the commit in `PINNED.md` is the sole authority
for every claim about the software. RFC UCAL-1 describes intentions, some of
which the code did not keep; where they disagree, the code is right and the RFC
is evidence about the past.

**Rule M** — every interpretive claim is marked typographically. `#claim("code")`
and `#claim("history")` pass through unmarked because they are checkable;
`interpretation` and `resonance` are ruled off in their own blocks. Deleting
every marked block must leave every technical claim standing:

```
python3 Documentation/LIFE_UNIVERSE_AND_GOD/deletion-test.py
```

That is A-P8, and it is a script rather than a promise. It strips every marked
block, compiles what remains, and fails if any surviving prose refers into a
deleted one. It checks structural dependence; it cannot check whether an
argument became less persuasive.

## State

**Complete draft.** All eight parts, 32 chapters, 217 pages.

RFC UCAL-A1 set a 200-page hard ceiling; it was raised on the author's
instruction and the book runs through it. Part VI at the book band — nine full
chapters rather than one with nine sections — is where the pages went.

| part | chapters | state |
|---|---|---|
| — | preface | drafted |
| I — Foundations | 1–4 | drafted |
| II — What was built | 5–8 | drafted |
| III — What implementation refused | 9–10 | drafted |
| IV — The datum | 11–13 | drafted |
| V — Any celestial body | 14–17 | drafted |
| VI — Nine readings | 18–26 | drafted |
| VII — The instrument as research tool | 27–28 | drafted |
| VIII — The claim | 29–32 | drafted |
| — | about the author | drafted |

## The samples

`samples/run-samples.py` regenerates every artifact under `assets/output/`.
Chapter 27 quotes those files and nothing else; chapter 28 reports what they
failed to establish.

```
python3 Documentation/LIFE_UNIVERSE_AND_GOD/samples/run-samples.py
```

S4 and S5 read a pinned instant rather than the clock, because a sample whose
output changes between runs is a demonstration and not evidence.

## Checks

| what | how |
|---|---|
| Rule M — no technical claim rests on an interpretive one | `python3 deletion-test.py` |
| Rule B — no uncited bibliography entry | `python3 samples/check-refs.py` |
| the six samples regenerate | `python3 samples/run-samples.py` |
| the diagnostic appendix matches the source | `python3 samples/gen-diagnostics.py` |

`GE-A4-reader-test.md` is the two-reader protocol. **It has not been run** — it
needs two people who are not the author, and it is the one experiment he cannot
run. Chapter 28 records it as not run, and the preface's dual-audience claim is
untested until it is.

## Note on the assets

The SVGs in `assets/logo/` are the identity files with their CSS `var()`
references resolved to the fallbacks declared alongside them. Typst's SVG
renderer has no CSS variable support, so an unresolved file renders with pieces
missing. The values are unchanged; `Documentation/logo/` remains the source.
