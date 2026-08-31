# S4 — is any of this useful outside astronomy?

**Status: research, answered. The short version is that the *library* is narrow,
the *design rules* are broad, and there are four domains where the fit is real
rather than rhetorical. Measured before answered.**

---

## What it costs, because that decides most of it

Measured on this machine, `u512` backend, release build, `black_box` on both
sides so nothing is optimised away:

| operation | cost | against `i64` |
|---|---|---|
| `i64` nanosecond add | 0.7 ns | — |
| **`Ticks` add (512-bit)** | **2.6 ns** | 3.7× |
| `Ticks` multiply | 12.8 ns | 18× |
| **`Ratio` multiply (reduces)** | **1290 ns** | ~1800× |
| render to 6 decimals | 327 ns | |
| parse a 61-digit tick count | 141 ns | |
| `size_of::<Instant<UC1>>()` | **64 bytes** | 8× |

**Three conclusions, and they are not the ones I expected before measuring.**

1. **Integer tick arithmetic is cheap.** 512-bit addition is within four times a
   native `i64` — it is eight adds with carry, and the branch predictor does the
   rest. Anything that would have used `i64` nanoseconds can use ticks without
   thinking about it.
2. **Exact rational arithmetic is not.** 1.3 µs per multiply, because it reduces
   by `gcd` every time. That is fine at design time and fine per query; it is not
   fine per event in a hot loop. **The rational layer is an analysis tool, not a
   per-event one**, and any engineering use should treat it that way.
3. **Memory is the real constraint.** 64 bytes a timestamp against 8 rules out
   bulk storage: a billion events is 64 GB rather than 8. For anything that
   *stores* time at volume, this is the wrong representation and no amount of
   correctness makes up for it.

---

## Where the fit is real

### 1. Metrology and calibration chains — the strongest

**Rule C is a calibration certificate.** A value, the source it came from, the
window it is valid over, and a warning (`UCAL-W0003`) when it is used outside
that window. That is exactly the structure of an instrument's calibration, and
in most engineering codebases it is a comment beside a constant.

The mechanism matters more than the idea. Everyone agrees constants should carry
provenance; this project *fails the build* when one does not, and `W0003` is
produced by the arithmetic rather than by somebody remembering. 1.10.0's whole
theme was that the machinery existed and reached nobody, which is the ordinary
state of this idea elsewhere.

There is a second, sharper thing metrology already knows and most code does not:
**the difference between a defining constant and a measured one.** E2 makes it
concrete — `c` and the astronomical unit are exact *by decision*, the parsec is
irrational *by definition*, and a measured quantity is neither. Code that mixes
the three has lost the ability to say which of its digits mean anything.

### 2. Safety-critical timing

The properties safety standards ask for are the ones this project chose for
unrelated reasons: **no floating point** in shipped crates, **no wrap and no
saturate** (Rule O), refusal in preference to approximation, and answers that are
**proved enclosures** rather than converged iterates.

That last one is the argument. In a safety case, *"this interval is proved to
contain the answer"* is a claim you can carry; *"this iteration stopped moving"*
is one you have to defend. `dilate` and `ucal-cosmo` both work that way, and the
technique — bracket with `isqrt_floor`/`isqrt_ceil`, round outward, never
narrow — transfers to any bounded quantity, not just time.

**Not a claim that this crate is certifiable.** It is not certified, has not been
through any process, and MPL-2.0 is not the licence such work usually starts
from. The *techniques* transfer; the artefact does not.

### 3. Long-horizon stewardship

Nuclear waste repositories, geological disposal, deep archival: **10⁴ to 10⁶
years**, with cited parameters and stated validity windows, and a requirement
that the record survive the institutions that made it.

This is the one domain where the domain *size* is a feature rather than a
curiosity, and where the stipulated datum earns its keep — a time coordinate that
does not depend on a calendar, an epoch, or a civilisation continuing to keep
one. `S2` measured the ceiling at cosmological decade 103.4; a repository needs
decade 6, which is comfortable in a way `time_t` is not.

The honest limit: this project supplies the *coordinate*, and the hard part of
long-term stewardship is the marker, the medium and the institution. A good time
representation is a small share of that problem, and pretending otherwise would
be the sort of overclaim `S1` refused about ephemerides.

### 4. Regulated and forensic timestamping

The failure mode in practice is not precision. It is **a timestamp that does not
say which clock it came from** — and regimes that care about trade timing or
evidentiary records ask for traceability and documented divergence, which is the
same requirement in different words.

A1's `--scale` is mandatory and will stay mandatory for exactly this reason: a
converter that defaults is silently wrong by 69 seconds whenever it guesses. That
one design decision transfers to any system where a timestamp crosses an
organisational boundary, and it costs nothing to adopt.

---

## Where it is a bad fit, said plainly

- **Distributed systems.** The field exists *because* there is no universal
  clock; ordering there is causal, not absolute. `ucal` is explicitly a
  coordinate clock in one frame (`S2`), and using it as if it settled ordering
  between machines would be actively misleading. This is the clearest *anti*-fit
  on the page.
- **Bulk event storage.** 64 bytes a timestamp. See the table.
- **Hot paths using rationals.** 1.3 µs a multiply.
- **Tight embedded memory.** `ucal-core` is `no_std` and builds for wasm, which
  is real — but a 64-byte instant and a 512-bit ALU in software is a lot to ask
  of a small MCU.
- **Anything wanting wall-clock convenience.** This project makes you say which
  scale, refuses `JD(UTC)`, and answers with intervals. That is the point, and it
  is the wrong trade for a log line.

---

## The part that transfers best is not the code

Five practices, none of which needs this crate, all of which are cheap:

1. **Every check is verified strict by injecting the defect it exists to
   catch.** The defect corpus is 24 mutations and none survives. Most projects
   have checks nobody has ever shown will reject anything — and this project
   found its own examples: a suite that passed on the *wrong* error, an oracle
   that compared one code path with itself, a lint whose scope silently excluded
   a whole tree.
2. **A generated artefact must regenerate, or it is not generated.** P2 found a
   generator that had drifted twenty lines from its own output — one whose header
   said it existed because hand-copied lists drift.
3. **A version marked released must exist where a user can get it.** R1 exists
   because 1.10.0 was marked released and existed nowhere: no tag, no release, no
   package. Every check in the tree read files that were in the tree.
4. **An uncertainty is carried when it is cited, and never when it is
   inferred.** B's rule, and it resolves the argument most codebases have by
   picking a side and applying it consistently.
5. **A claim can have a shelf life, and the claim should say so.** R2's finding:
   *`cargo package` is byte-reproducible* was measured correctly, was true when
   measured, and expired the moment a later compatible version was published.
   Nobody stated the expiry. That generalises further than anything else here.

---

## The bottom line

**As a library: narrow, and honestly so.** Most engineering time problems are
well served by `i64` nanoseconds, and should be. The cases that are not are the
ones needing more than about twenty orders of magnitude of range, exact rational
time arithmetic, enforced provenance, or proved enclosures — and few problems
need more than one of those at once.

**As a worked example of how to be exact about a physical quantity: broad.** The
rules are mechanical, the checks are verified strict, and the reasoning is
written down where the code is rather than in a design document nobody opens.

**And the four domains above are real**, in descending order of confidence:
metrology, safety-critical timing, long-horizon stewardship, regulated
timestamping. None of them is astronomy, and none of them was designed for.
