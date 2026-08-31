# Consuming `ucal` from another language

**D** of [`S1`](Proposals/S1-astrophysics-roadmap.md)'s plan. This project supplies
exact time; a pipeline supplies the science. That division is deliberate — `S1`
draws the boundary at *this project must not become an ephemeris library* — and
it only works if the seam between them is documented rather than inferred.

Everything here is a contract, not a convenience. [`STABILITY.md`](STABILITY.md)
states the promise; this page is how to use it.

---

## The shape

Every command that emits a document supports `--json`, and every document opens
with the same field:

```json
{"format": "ucal-json/1", "ticks": "8070205...", ...}
```

**Check `format` before anything else.** It is the version of the surface, not of
the program: a `1.x` release and a `1.y` release emit `ucal-json/1`, and a reader
that pins to the program's version has pinned to the wrong thing.

### Every number is a string

```json
{"ticks": "8070205189123984864657505252035637180530466139316558837890625"}
```

A tick count is a 61-digit integer today and can be 155 digits at the domain
ceiling. **JSON's number type is a double in most parsers**, which would silently
round it to 17 significant figures — and this whole project exists because that
rounding is not acceptable. So integers, rationals and rendered decimals are all
strings, and a consumer parses them with a big-integer or decimal type.

In Python:

```python
import json, subprocess
from decimal import Decimal

doc = json.loads(subprocess.check_output(["ucal", "--json", "now"]))
ticks = int(doc["ticks"])          # exact, arbitrary precision
```

`int()` is exact for any size. `float()` is not, and is the mistake this format
is shaped to prevent.

### Streaming is JSONL

A command taking one instant accepts `-` and reads one per line, emitting **one
document per line**:

```
ucal seq <A> <B> --step T1 | ucal --json explain -
```

One JSON object per output line, not an array. An array would have to be complete
before it was valid, which is not a filter — and a stream that cannot be consumed
incrementally is a stream in name only.

A command taking *two* instants streams when exactly one side is `-`; both sides
is refused rather than guessed at, because a stream of pairs and the same line
used twice are different things and neither is obviously meant.

### Windows, and why so many answers are one

Rule U: **the window is the value.** Where a result is interval-valued it says so
in its shape rather than in a note:

```json
{"window": {"lo": "…", "hi": "…", "width_ticks": "…"}}
```

A consumer that reads only `lo` has read a number that is true and incomplete.
`cal from`, `ephem at`, `from-jd --scale tdb` and every event in the catalogue
answer this way, and the width is usually the interesting half — an ephemeris
prediction's window is what decides whether an observation is worth scheduling.

### Warnings are data

```json
{"warning": "UCAL-W0003: …"}
```

**A warning is not a failure and does not change the exit code.** `UCAL-W0003`
means a parameter was used outside the range its source states — Rule C requires
saying so and forbids extrapolating silently, not extrapolating. A pipeline that
drops the field has thrown away the part that says the number is being asked to
do something its source does not support.

### Exit codes

| | |
|---|---|
| 0 | success |
| 1 | usage error |
| 2 | parse error |
| 3 | domain error |
| 4 | precision error |
| 5 | profile mismatch |
| 6 | data or config error, including missing provenance |
| 7 | calendar derivation or anchor error |
| 8 | cosmology model or enclosure error |
| 9 | internal invariant violation |

**Diagnostics go to stderr and stdout stays clean**, so a non-zero exit never
leaves half a document on the pipe. A consumer can parse stdout unconditionally
and check the code afterwards.

---

## What the promise is

From [`STABILITY.md`](STABILITY.md), and enforced by
`crates/ucal/tests/json_surface.rs` against a committed baseline:

- a path that **disappears** is a breaking change;
- a path whose **type changes** is a breaking change;
- a path that **appears** is not, and needs no discussion.

That asymmetry is the contract. A consumer must ignore fields it does not know,
and may rely on the ones it does.

**What it cannot promise is meaning.** A field that keeps its name and its type
and starts reporting something else passes every check here and is exactly the
breakage the promise is about. Nothing mechanical reaches that, and it is said
here rather than left for a green build to imply otherwise.

The machine-readable form is
[`fixtures/ucal-json-1.schema.json`](../fixtures/ucal-json-1.schema.json),
generated from the same baseline and checked against it on every push.

## Commands that emit no document

`--json` does not apply to `completions` or `man`, which emit a shell script and
roff; to `seq`, which emits one decimal tick count per line so that it composes
with the shell; or to `cal export`, which emits an HJSON body file because a
generator's output is an input to something else.

**These are enumerated rather than discovered.** Since 1.12.0 a `check-docs`
check requires every command either to contribute to the surface baseline or to
be listed as emitting no document with a reason — because until then a legitimate
absence and an oversight looked identical to every check in the tree, and one of
them was an oversight.

---

## A worked consumer

```python
import json, subprocess
from decimal import Decimal

def ucal(*args):
    """One ucal document. Raises with the diagnostic on a non-zero exit."""
    p = subprocess.run(["ucal", "--json", *args], capture_output=True, text=True)
    if p.returncode != 0:
        raise RuntimeError(f"ucal exited {p.returncode}: {p.stderr.strip()}")
    return json.loads(p.stdout)

def stream(args, lines):
    """Many documents, one per input line."""
    p = subprocess.run(["ucal", "--json", *args], input="\n".join(lines),
                       capture_output=True, text=True, check=True)
    return [json.loads(l) for l in p.stdout.splitlines() if l.strip()]

# An epoch from a paper, converted with the scale named — there is no default,
# because a converter that defaults is silently wrong by 69 seconds when it
# guesses.
d = ucal("from-jd", "2451545.0", "--scale", "tt")
t0 = int(d["ticks"])

# An ephemeris prediction. The window is the answer; the centre alone is not.
p = ucal("ephem", "at", "ephemeris.hjson", "--cycle", "5000", "--sigmas", "3")
pred = p["prediction"]
half = int(pred["half_width_ticks"])
if p.get("warning"):
    print("outside the fitted range:", p["warning"])
print("3σ window is", 2 * half, "ticks wide")
```

**Three habits worth copying**: parse integers with `int`, read `warning`
whenever it is present, and treat a window's width as part of the answer rather
than as diagnostics. Every one of them is a thing this format is shaped to make
easy and a `float()` is enough to undo.

---

## Cross-checking

`ucal verify` re-derives the declared constants and says plainly that a build
agreeing with itself is not verification. The stronger claim available is
agreement with an implementation written by somebody else, and since 1.12.0 the
Julian Date conversions are checked against
[`hifitime`](https://crates.io/crates/hifitime) — a different author, working
from the standards rather than from this code.

That check is **one-directional and says so**: `hifitime` returns `f64` days, and
a Julian Date near 2 460 000 has about two microseconds of representation spacing
in a double, so the oracle cannot check this crate past that. It catches a wrong
epoch, a wrong day length, a scale applied backwards and an off-by-one in the MJD
offset — every error anybody actually makes — and it cannot catch a
sub-microsecond one. Nothing available could.

[`S1`](Proposals/S1-astrophysics-roadmap.md) assessed
[`siderust`](https://lib.rs/crates/siderust) for the same job and rejected it:
AGPL-3.0-only against this workspace's MPL-2.0 settles it before `f64`,
`std`-only and 351 kSLoC each settle it again.
