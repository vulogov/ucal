# Life, the Universe, and God

*A Software Engineer's Instrument for the Immeasurable*

A book about `ucal`, written to RFC UCAL-A1. Typst source; the compiled artifact
is `BOOK.pdf`.

## Build

```
typst compile Documentation/LIFE_UNIVERSE_AND_GOOD/BOOK.typ
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
every marked block must leave every technical claim standing. That is a
scheduled production step, not a hope.

## State

Front matter and Part I are drafted. The remaining chapters are carried as
stubs that state their specification, so the book's shape is visible in the
contents and its length is not quietly understated — the same discipline as
`UCAL-E0062` in the software it describes.

| part | chapters | state |
|---|---|---|
| — | preface | drafted |
| I — Foundations | 1–4 | drafted |
| II — What was built | 5–8 | specified |
| III — What implementation refused | 9–10 | specified |
| IV — The datum | 11–13 | specified |
| V — Any celestial body | 14–17 | specified |
| VI — Nine readings | 18–26 | specified |
| VII — The instrument as research tool | 27–28 | specified |
| VIII — The claim | 29–32 | specified |
| — | about the author | drafted |

## Note on the assets

The SVGs in `assets/logo/` are the identity files with their CSS `var()`
references resolved to the fallbacks declared alongside them. Typst's SVG
renderer has no CSS variable support, so an unresolved file renders with pieces
missing. The values are unchanged; `Documentation/logo/` remains the source.
