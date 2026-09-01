# S1 — what `ucal` could be for astrophysics, across several releases

**Status: a plan, with one dependency assessed and rejected, and one boundary
drawn that the rest of the plan depends on.**

---

## The premise, stated before anything is proposed

Astronomy's time bookkeeping is dominated by two error classes, and **neither of
them is physics**:

1. **Scale confusion.** UTC, TAI, TT, TDB, TCB, UT1, and the barycentric forms
   built on them. `TT − TAI` is 32.184 s exactly; `TT − UTC` is 69.184 s today
   and changes without warning; `TDB − TT` is periodic and under 1.7 ms. Papers
   report `BJD` without a scale, `HJD` where `BJD` was meant, and `JD` where the
   reader must guess. A 69-second error is invisible in a light curve and fatal
   in a pulsar residual.

2. **Accumulated arithmetic.** A linear ephemeris `T(E) = T₀ + E·P` propagated
   ten thousand cycles in `f64` from a value near 2 460 000 has ~2×10⁻¹¹ d of
   representation error per operation, which is small — and the *uncertainty*
   `E·σ_P` is not, and is routinely dropped. Observers then book a 30-minute
   window for an event whose 1σ prediction is ±40 minutes.

**This project is unusually well placed for both**, and badly placed for a third
thing it must not attempt. Its time is an exact unsigned integer, so `E·P` has no
representation error at any epoch and no accumulation at all. Its answers are
intervals by construction — Rule U says *the window is the value* — so a
prediction that has become uncertain **says so in its type** rather than in a
sentence somebody wrote in a docstring. And Rule C already refuses a parameter
used outside its stated validity window, which is exactly the failure mode of a
2015 ephemeris applied in 2026.

What it is **not** placed to do is evaluate ephemerides. See *The boundary*.

---

## The dependency question: `siderust`

[`siderust`](https://lib.rs/crates/siderust) 0.11.0 — VSOP87 and ELP2000-82B
analytical ephemerides, SGP4/TLE, frame transforms with compile-time frame
safety, POD with LSQ and EKF, Gaia DR3 catalogues. It is a serious and
well-shaped library, and its typed-frames design is the thing this project would
most like to have thought of first.

**It cannot be a dependency of any shipped crate here, for four independent
reasons, and any one would be enough.**

| | `siderust` | this workspace |
|---|---|---|
| licence | **AGPL-3.0-only** | MPL-2.0 |
| arithmetic | `f64` throughout | **Rule E**: no float in a shipped crate |
| environment | `std` only (via `chrono`) | `no_std` + wasm is a build we keep green |
| size | ~351 kSLoC, 15–30 MB, ~30 deps | six crates, two integer backends, `clap` |

The licence alone settles it: AGPL-3.0-only is not compatible with distributing
this workspace under MPL-2.0, and the author's dual-licence offer is a
commercial arrangement rather than a technical fix. `f64` settles it a second
time — Rule E is not a style preference, it is the reason the tier ladder's
bottom nine rungs mean anything.

### Where it *is* useful: as an oracle

This workspace has the pattern already, twice over. `ucal-civil` takes
`hifitime` and uses **only its integer-exact surface**. `ucal-cosmo` keeps a
float oracle in `#[cfg(test)]`, contained by a module and an explicit lint
marker, because a certified integer quadrature that agrees with nothing is a
number nobody can check.

`siderust` would be a **third oracle**, and a good one: an independent
implementation of VSOP87 and of the scale conversions, by a different author,
to cross-check any tick-native result against. Its `f64` answers become the
reference that this project's intervals must *contain* — which is the correct
relationship, since an interval that fails to contain a good float estimate has
found a defect in itself.

**Stop if** AGPL in `dev-dependencies` turns out to reach the distributed crate.
The conservative reading is that test-only code is not conveyed and the
obligation does not attach; the safe engineering answer is to keep the
comparison in a **separate unpublished crate** in this workspace — `xtask` is
already `publish = false` for a related reason — and to say in
`spec/CONFORMANCE.md` that it is there. A licence question answered by a
shrug is the kind of thing this project writes down.

---

## The boundary, drawn first

**`ucal` must not become an ephemeris library.** VSOP87 is ~2 500 published
periodic terms fitted in floating point; DE440 is a Chebyshev interpolation of a
numerical integration. Re-expressing either in exact rationals would be a large
project that **buys no accuracy**, because the coefficients themselves carry the
fit's error and the underlying integration is float from end to end. An exact
evaluation of an approximate series is a precise wrong answer, which is the one
thing this project's whole design is against.

So: **positions come from elsewhere.** What this project supplies is the *time*
that everything is indexed by, and the *arithmetic on published timing models*
that the field currently does in `f64` without carrying the uncertainty.

That boundary is what makes the rest of this plan small enough to finish.

---

## Phase A — time scales, exactly (one cycle)

The foundation. Nothing later is trustworthy without it, and it is the piece
with the best ratio of usefulness to work.

**A1 — Julian Date in and out, with the scale required and no default.**
Every cited parameter in `ucal-body` is indexed by JD in its source, and J2000 —
the epoch the entire body layer hangs on — *is* JD 2451545.0 TT. Today you
cannot convert a JD to a tick, so checking a shipped parameter against the paper
it came from is done by hand.

`ucal from-jd 2451545.0 --scale tt` and `ucal to-jd <T> --scale tdb`, with
`--scale` **mandatory**. A converter that defaults is a converter that is wrong
69 seconds of the time, silently, and the whole point is to make the scale a
thing the user typed. MJD and the `2400000.5` offset are exact and come free.

**A2 — `TDB` as a bounded interval rather than a value.** `TDB − TT` is a
periodic series whose amplitude is 1.6568 ms; the standard evaluation is a
float polynomial. Rather than either ignoring it or importing a float series,
report the conversion as a `Window` of half-width 1.7 ms and say why. **That is
the honest answer and it is also useful**: a great many analyses need only to
know the bound, and the ones that do not are the ones that should be using a
real ephemeris anyway.

**Stop if** the scale offsets cannot be sourced to a citation each under Rule C.
`TT − TAI = 32.184 s` is a defined constant; leap seconds are already in
`ucal-civil`; `TDB` is IAU 2006 Resolution B3. If one of them cannot be quoted
verbatim, that one does not ship.

---

## Phase B — the flagship: cited ephemerides that carry their own uncertainty

**This is the feature the field would actually use, and no library does it.**

A linear ephemeris is how nearly every repeating astronomical event is
published:

```
T_c(E) = T_0 + E · P          transiting planets, eclipsing binaries
T(E)   = T_0 + E · P + ½ E² P·Ṗ   pulsars, decaying orbits
```

`T_0` and `P` come with published uncertainties, and the useful quantity is not
`T(E)` — it is the **window** `T(E) ± sqrt(σ_T₀² + (E σ_P)²)`, which is what
decides whether an observation is worth scheduling. Most tooling computes the
centre and drops the width.

### Why it belongs here specifically

Everything needed is already built and, in two cases, built and unused:

- `Window` and Rule U — the answer is an interval and always was.
- `Anchor` with `uncertainty()` and a `revision` — an epoch with a width and a
  provenance, which is exactly `T_0`.
- `RatedParam` with a **secular rate** — `with_rate_per_julian_century` exists,
  has three call sites and all three are its own tests. `Ṗ` is the astronomical
  name for that field. [`H1`](H1-when-does-a-calendar-hold.md) Finding 2 records
  that this project models drift and no shipped parameter uses it. **A pulsar
  would.**
- Rule C and `UCAL-W0003` — an ephemeris propagated outside the epoch range it
  was fitted over must warn, and this is the one place where that rule maps onto
  a mistake people actually make in print.

### The shape

```rust
pub struct Ephemeris {
    epoch: Anchor,          // T_0, with its window and its citation
    period: RatedParam,     // P, with sigma, Pdot, and a validity window
    citation: Citation,
}

impl Ephemeris {
    fn time_of(&self, cycle: i64) -> Result<(Window<UC1>, Option<Warning>)>;
    fn cycle_at(&self, t: &Instant<UC1>) -> Result<(i64, Ratio, Option<Warning>)>;
    fn next_after(&self, t: &Instant<UC1>) -> Result<(i64, Window<UC1>)>;
}
```

`ucal ephem next hd189733b --after now`, answering *when is the next transit and
how wide is the window*, with the citation and the warning attached.

### The one thing that must be added, and why it is not a fabrication

`ucal-body`'s parameters carry a validity window and a rate and **no uncertainty
magnitude on the value**, and `calendar.rs` says why: the planetary sources do
not uniformly publish one, and *"adding a fabricated one would be worse than
omitting it"*.

**For ephemerides the opposite holds.** `σ_P` is published — it is the headline
number of every ephemeris refinement paper — so carrying it is Rule C working
rather than Rule C bent. That distinction is the spec delta this phase needs,
and it should be written in those terms: *an uncertainty is carried when it is
cited, and never when it is inferred.*

**Stop if** the propagated window turns out to be dominated by `σ_T₀` for every
real target, making the growth term decorative. Then the finding is that
published ephemerides are epoch-limited rather than period-limited, which is
worth knowing, and the feature is still correct — but the pitch changes.

---

## Phase C — certified two-body geometry, where certification earns its cost

`ucal-cosmo` already does flat ΛCDM **by certified integer interval
quadrature**: an enclosure that is proved to contain the answer, not an
iterate that stopped moving. The same technique applies to Kepler's equation
`M = E − e sin E`, by interval bisection with a rational sine bound.

The honest scope: this is **expensive and rarely necessary**. Nobody needs a
certified true anomaly for a finder chart. It earns its cost in exactly two
places — long-baseline propagation where the float error is not obviously
bounded, and a paper that wants to state a **proved** bound rather than a
converged one.

So this phase is worth doing **only if** Phase B finds a caller for it. Recorded
here so that it is a decision rather than an oversight, and left with its cost
attached.

**Stop if** the interval for a moderately eccentric orbit does not converge to
better than the observational uncertainty within a sane number of bisections.
Then certification costs more than the measurement is worth, and the answer is
to say so and stop.

---

## Phase D — the interchange surface

The point at which the field can use any of this without writing Rust.

- **`ucal-json/1` output already exists** and is version-promised, which is more
  than most astronomy tooling offers. A thin Python reader over it is a
  weekend's work *for somebody else*, and is the right shape: this project
  supplies exact time, the pipeline supplies the science.
- **Ephemerides as cited §15.1-style files**, so a third party can declare one
  and get windows out, exactly as `cal derive` does for calendars today. The
  loader, the validator, the precision probe and the round-trip export all
  already exist for body files; an ephemeris is a smaller version of the same
  thing.
- **An oracle suite against `siderust` and `astropy`** for the scale
  conversions, in an unpublished crate. Agreement with two independent
  implementations is the strongest claim available here, and it is the claim
  `ucal verify` currently has to say it cannot make.

---

## What this adds up to

Not an astronomy library. **A time substrate for one**, with three properties
the field does not otherwise get together in one place:

1. **Exact.** Integer ticks, no accumulation, no epoch-dependent precision loss.
2. **Interval-valued.** Uncertainty is in the type and grows correctly under
   propagation, so a prediction that has decayed cannot be read as sharp.
3. **Cited.** Every constant carries its source and its validity window, and
   using one outside that window warns rather than extrapolating.

The realistic sequence is **A, then B, then D**, with **C** only if B produces a
caller for it. A is one cycle. B is the flagship and is probably two, because
the uncertainty delta touches `ucal-body`'s parameter model. D is open-ended and
partly somebody else's.

**And the first thing to build is A1**, because it is small, it is immediately
useful to this project's own maintenance — the shipped parameters could then be
checked against their sources mechanically — and it is the piece everything
after it stands on.
