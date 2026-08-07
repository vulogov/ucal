#import "../design.typ": *

#appendix(letter: "C", title: "Diagnostic codes")

Generated from `ucal-core`'s error module by
`samples/gen-diagnostics.py`. The table is derived rather than
transcribed, for the same reason §13.5 makes the tier table generated: a
hand-copied list drifts silently, and a reference that disagrees with the
software is worse than none.

#section("What the codes are for")

Chapter 16 counted four epistemic limits and found the same response in all
four — the system errors or warns and never defaults. This appendix is that
policy enumerated. Every entry below is a place where the artifact declines
to produce a plausible number.

#section("Exit codes")

The command line maps each family to a process exit status, so a failure is
distinguishable by class without parsing the message.

#block(width: 100%)[
  #set text(size: 9.5pt)
  #table(
    columns: (auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 4.5pt),
    [*band*], [*subject*], [*exit*],
    [`E0001–E0007`], [notation and parsing], [2],
    [`E0010–E0014`], [profile, provenance, and names], [6],
    [`E0015`], [a build that does not reproduce its constants], [9],
    [`E0020–E0025`], [domain, ordering, and the claim], [3, 9],
    [`E0030–E0032`], [identifiers and encoding], [2, 3],
    [`E0040–E0043`], [the SI bridge and civil time], [2, 4],
    [`E0050–E0065`], [bodies, anchors, and calendars], [5, 7],
    [`E0070–E0080`], [numerics and cosmology], [3, 8],
  )
]

Two statuses fall outside that mapping and mean different things. Exit `1` is a
usage error, raised before any code is reached — an unknown flag, a missing
argument. Exit `70` is `EX_SOFTWARE`: a panic that reached the top of the
program. It is deliberately outside the `0–9` range the table uses, so that a
defect in `ucal` cannot be mistaken for a diagnosed failure of the input. When
it appears, the message says so, and gives the issue tracker rather than a
stack trace.

#section("Errors")

#block(width: 100%)[
  #set text(size: 9pt)
  #table(
    columns: (auto, 1fr, auto, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 4pt),
    [*code*], [*meaning*], [*exit*], [*ch.*],
    [`UCAL-E0001`], [malformed timestamp], [2], [—],
    [`UCAL-E0002`], [unknown profile tag], [2], [—],
    [`UCAL-E0003`], [mixed text forms in one string], [2], [—],
    [`UCAL-E0004`], [group value out of range (> 3124)], [2], [—],
    [`UCAL-E0005`], [invalid base-5 digit], [2], [—],
    [`UCAL-E0006`], [non-contiguous tier sequence], [2], [—],
    [`UCAL-E0007`], [calendar rendering without a kind/id qualifier], [2], [—],
    [`UCAL-E0010`], [locale table load failure], [6], [12],
    [`UCAL-E0011`], [duplicate name in the active locale table], [6], [12],
    [`UCAL-E0012`], [unknown key in HJSON data file], [6], [12],
    [`UCAL-E0013`], [profile lacks a datum_provenance record], [6], [12],
    [`UCAL-E0014`], [name not found in the active locale table], [6], [12],
    [`UCAL-E0015`], [this build does not reproduce the declared constants], [9], [12],
    [`UCAL-E0020`], [result precedes the datum], [3], [3],
    [`UCAL-E0021`], [result exceeds DOMAIN], [3], [5],
    [`UCAL-E0022`], [window inversion, lo > hi], [3], [12],
    [`UCAL-E0023`], [comparison indeterminate at stated precision], [4], [5],
    [`UCAL-E0024`], [lossy rendering requested without a rounding mode], [4], [5],
    [`UCAL-E0025`], [BIG_BANG_CLAIM used as a computational operand], [9], [12],
    [`UCAL-E0030`], [binary form is not 64 bytes], [3], [6],
    [`UCAL-E0031`], [instant outside UCID range], [3], [6],
    [`UCAL-E0032`], [invalid Crockford base-32], [2], [6],
    [`UCAL-E0040`], [civil date outside renderable range], [2], [7],
    [`UCAL-E0041`], [invalid civil date for the stated calendar], [2], [7],
    [`UCAL-E0042`], [second = 60 outside a leap-second instant], [2], [7],
    [`UCAL-E0043`], [foreign-unit input finer than the bridge constant permits], [4], [7],
    [`UCAL-E0050`], [profile mismatch], [5], [5],
    [`UCAL-E0060`], [body parameter missing required provenance or as-measured value], [7], [5],
    [`UCAL-E0061`], [leap-rule derivation cannot meet the requested drift bound], [7], [5],
    [`UCAL-E0062`], [calendar has no anchor; local fields cannot be produced], [7], [8],
    [`UCAL-E0063`], [anchor phase definition not evaluable for this body], [7], [8],
    [`UCAL-E0064`], [grouping cycle requested but none derivable from any satellite], [7], [8],
    [`UCAL-E0065`], [legacy calendar supplied where a derived calendar is required], [7], [8],
    [`UCAL-E0070`], [division by zero or by an interval containing zero], [8], [5],
    [`UCAL-E0071`], [requested enclosure width unreachable at the permitted depth], [8], [5],
    [`UCAL-E0080`], [tier index outside the profile grid], [3], [5],
  )
]

#section("Warnings")

A warning is returned alongside a value. It never replaces one, and it is
never suppressed by default — chapter 8's `UCAL-W0003` and chapter 5's
`UCAL-W0004` are both cases where the answer is real and incomplete.

#block(width: 100%)[
  #set text(size: 9pt)
  #table(
    columns: (auto, 1fr, auto),
    stroke: (x, y) => if y == 0 { (bottom: 0.6pt) } else { (bottom: 0.2pt + luma(200)) },
    inset: (x: 5pt, y: 4pt),
    [*code*], [*meaning*], [*ch.*],
    [`UCAL-W0001`], [precision loss in the requested rendering], [5],
    [`UCAL-W0002`], [leap-second table may be stale; bounded error reported], [7],
    [`UCAL-W0003`], [body parameter evaluated outside its validity window], [8],
    [`UCAL-W0004`], [cosmology enclosure width exceeds one tick], [5],
    [`UCAL-W0005`], [value produced by a legacy (non-derived) calendar], [8],
    [`UCAL-W0006`], [quantity comparable to or smaller than BIG_BANG_CLAIM], [12],
  )
]

#callout(label: "The four this book turns on")[
  / `UCAL-E0020`: a result preceding the datum. Chapter 3 — a malformed
    question refused, not a value on the far side of an origin.
  / `UCAL-E0025`: `BIG_BANG_CLAIM` used as an operand. Chapter 12, and the
    only code in the set that no program can reach, because the type has no
    operators for it to reach through.
  / `UCAL-E0062`: a calendar with no anchor. Chapter 16 — an absence
    reported rather than defaulted.
  / `UCAL-W0003`: a parameter outside its validity window. Chapter 20 found
    this to be *ʿāda* compiled: reliable where observed, no claim beyond.
]

