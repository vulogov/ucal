# GE-U4 — a tier-scale navigator

**Status: proposed, not built. Kill criterion written first.**

A gated experiment in the shape the RFC uses for GE-1…GE-6: a question, a
measurement that answers it, and a condition under which the answer is *no* and
the work is deleted rather than defended.

---

## The question

Can a terminal interface make the domain's scale *felt*, when no static output
has?

This is narrower than "should there be a TUI", and deliberately. The project
states its scale constantly and nobody has ever experienced it:

| quantity | value |
|---|---|
| domain | 2.29 × 10¹⁰³ years |
| present epoch, as a fraction of it | 6 × 10⁻⁹⁴ |
| base-5 groups at the present epoch | 18 significant, 27 leading zeros |
| one tier step | × 3125 |
| tiers | T-12 … T32, 45 of them |

The 0.3.0 work already put the last two rows on screen. `ucal ladder` renders
the ladder as a table, and U3's dimming shows the leading-zero run as a region
rather than a number. Both help. Neither makes 10⁻⁹⁴ mean anything, because a
static rendering can only show you a ratio — it cannot make you *travel* one.

## What is proposed

One screen. A position on the ladder, and two keys that move it up and down a
tier. Holding a key walks the ladder, and the display re-renders the same
instant at the new scale, so the reader watches the present epoch collapse to
nothing over 45 steps and then walks back.

Nothing else. Not a calendar view — that is `ucal cal show` with borders drawn
round it, and it would be permanent surface bought for nothing. Not an event
browser; `ucal timeline` is a table and tables are what U2 was for.

## Kill criterion

**A reader who has used it for five minutes can state the ratio between two
named tiers within an order of magnitude, and a reader given `ucal ladder`
instead cannot.**

Both halves matter. The first alone would be satisfied by a reader who already
knew; the second is what distinguishes the navigator from the table it would
sit beside.

If it fails, it was an ornamented `ladder`, the crate is deleted, and the
failure is recorded the way GE-1, GE-2, GE-5 and GE-6 were — four of six gated
experiments in this project have fired their kill criteria, and the record of
them is more useful than the four features would have been.

## Protocol

Like GE-A4's two-reader test, this cannot be run by the author, and for the same
reason: the author already knows the answer to the question being asked. It
needs two people who have not read this repository.

1. Each is given the same three questions of the form *"how many `spark`s in a
   `drift`?"*, answers accepted within an order of magnitude.
2. One is given `ucal ladder` and five minutes. The other is given the navigator
   and five minutes.
3. Both are asked the questions again afterwards, with the tool taken away.
4. The criterion is met only if the navigator reader improves and the table
   reader does not.

Two readers is not a study and will not be reported as one. It is enough to
distinguish "this obviously works" from "this obviously does not", which is the
only question a kill criterion has to settle.

## Structure, if it is built

**A separate `ucal-tui` crate, not a feature of `ucal`.** `ratatui` and
`crossterm` are a large tree, and `cargo install ucal` should stay lean —
today the binary's direct dependency list is `ucal-core`, the four optional
sibling crates, `clap`, `anstyle` and `terminal_size`. That is worth keeping.

**Out of the publish set until it earns inclusion.** `xtask -- publish` derives
its order from the dependency graph and honours `publish = false`, so a crate
that is not ready is excluded by its own manifest rather than by anyone
remembering.

**It renders `Doc`s, not its own data.** Every command is already a pure
function from arguments to a `Doc`, and `Render` already carries a width. A
navigator that computed its own tier values would be a second source of truth
for §13.5 to worry about.

**The strip invariant does not apply to it**, and that is worth saying out
loud rather than discovering later. `strip_ansi(to_ansi(style)) == to_text()`
is a claim about a *document*; a full-screen interface has no plain rendering
to be equal to. If the navigator is built, the claim it needs is a different
one — that every quantity it displays is also obtainable from a command — and
that claim needs its own test rather than an appeal to this one.

## What would make this not worth building

Recorded now, while it is cheap to say:

- **If `ucal ladder` after U2 already conveys it.** The table is new. Nobody has
  used it yet, and the honest order is to find that out first — which is part of
  why this is a proposal and not a crate.
- **If the answer is a diagram rather than a program.** The book already carries
  one static plate on exactly this subject. A second, better one would cost an
  afternoon rather than a dependency tree, and would reach every reader of the
  PDF instead of the subset with a terminal.
- **If it needs to be interactive to work at all.** That is a hypothesis, not a
  finding. An animation — the same walk, recorded — would test the same idea at
  a fraction of the cost, and if the walk works recorded, the interactivity was
  never the load-bearing part.

The third is the one I would test first if this were resumed.

## Prior art in this repository

Not consulted for design, listed so a future reader knows what was already
tried: `ucal ladder` (the whole grid, one table), `ucal timeline` (the
catalogue against the grid), `ucal ruler` (evenly spaced marks between two
instants), and the book's scale plate. Each shows the ratio. None moves through
it.
