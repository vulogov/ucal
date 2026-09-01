# S3 — more astrophysics, sorted by what kind of answer exists

**Status: a brainstorm, with every number checked. Third of the astrophysics
series after [`S1`](S1-astrophysics-roadmap.md) (ephemerides, and the boundary)
and [`S2`](S2-deep-time.md) (black holes, stellar lifetimes, and the domain).**

---

## The organising idea

The field's time quantities fall into **three classes**, and almost no tool
distinguishes them:

1. **Exact by definition.** An SI or IAU *defining* constant is not a
   measurement; it is a decision. Conversions built on one are exact rationals,
   and this project can be perfectly right about them. Nobody exploits this.
2. **Bounded, where the bound is free.** The value needs data this project must
   not carry, and the *envelope* needs nothing at all. A2 already did this for
   `TDB`; several more work the same way.
3. **A fit.** Least squares, model grids, anything with a covariance matrix.
   **Not this project's business**, and the line is worth drawing before the
   features are, because two of the items below sit right on it.

Sorting by that rather than by subject is what makes this list short.

---

## Class 1 — exact by definition, and unexploited

### E1 — `TCG` and `TCB`, which are exactly what `TDB` is not

**The strongest single item here.** A2 shipped `TDB` as a ±1.7 ms *bound*,
because `TDB − TT` is a periodic series whose evaluation is floating point. That
is the honest ceiling for `TDB`, and it does not apply to its two neighbours:

| | rate constant | status |
|---|---|---|
| `TCG − TT` | `L_G = 6.969290134 × 10⁻¹⁰` | **defining** (IAU 2000 Resolution B1.9) |
| `TCB − TDB` | `L_B = 1.550519768 × 10⁻⁸` | **defining** (IAU 2006 Resolution B3) |

A defining constant is exact. Both are terminating decimals, so both are exact
rationals, so **`TT ↔ TCG` and `TDB ↔ TCB` are exact linear conversions** — no
series, no float, no bound. This project would be *exactly right* about them
where every other tool is approximately right.

**And the difference is large.** Measured:

- `TCB` runs ahead of `TDB` by **0.489 s per Julian year**.
- `TCG` runs ahead of `TT` by **0.022 s per Julian year**.

Half a second a year, in a field where pulsar timing residuals are microseconds.
Confusing `TDB` with `TCB` is not a rounding — it is a linear drift that reaches
a minute inside two centuries, and papers do confuse them.

**Stop if** the rate constants cannot be quoted verbatim from the resolutions
under Rule Y.1. They are short and unambiguous, so this should not fire; if it
does, the finding is that this project cannot cite the IAU, which would be worth
knowing for a great deal more than this.

### E2 — light-travel time, and the three units that behave differently

`c = 299792458 m/s` is exact by definition. So is the astronomical unit, since
IAU 2012 Resolution B2: `1 au = 149597870700 m`, a decision rather than a
measurement. From that:

| unit | light-travel time | kind |
|---|---|---|
| **1 light-year** | **31 557 600 s exactly** | an integer |
| 1 au | `1024642950/2053373` s = 499.0047838… s | an exact rational |
| 1 parsec | `648000/π` au | **irrational** |

The light-year is the joke that turns out to be true: it is defined as a Julian
year times `c`, so **its light-travel time is a Julian year and the conversion is
the identity**. A light-year is a time unit wearing a distance's clothes, and
this project can say so with no arithmetic at all.

The parsec is the interesting one. `1 pc = 648000/π au` is an exact *definition*
of an irrational number, so it needs a **certified bracket** — the same outward
bracketing `dilate` does with `isqrt`, applied to `π` instead. An interval, and
correct, rather than a decimal that is neither.

**I have not seen a tool state that distinction**, and it is the sort of thing
this project exists to state: two of the three convert exactly and the third
cannot, for a reason that is about the definition rather than about the code.

### E3 — characteristic age, `τ = P/(2Ṗ)`

Both fields are **already on `Ephemeris`** from B. The division is an exact
rational, and `τ` is the standard age estimate for a pulsar.

What makes it worth shipping is the caveat, not the arithmetic. `τ` assumes a
braking index of exactly 3 and an initial period much shorter than the present
one, and it is **routinely wrong by a factor of a few** — the Crab's `τ` is about
1240 yr against a known age of 972. A number that carries its own assumption
beside it is the whole habit of this project, and this is a case where the field
itself knows the assumption is shaky and prints the number anyway.

---

## Class 2 — bounded, where the bound is free

### B1 — the barycentric correction, as an envelope

`S1` put barycentric correction out of scope because `BJD` needs the Earth's
position relative to the solar-system barycentre, which needs an ephemeris, which
is the boundary. That remains true of the **value**. It is not true of the
**bound**:

```
|BJD − JD| ≤ 1 au / c = 499.004783836… s
```

For any target, any date, no ephemeris. The same move A2 made for `TDB`, and it
is more useful than it sounds: **it answers whether the question is sensitive to
the correction at all.** A transit timed to a minute is not; a pulsar residual is,
by six orders of magnitude. Most people asking do not need the correction and
cannot easily tell.

Supply your own light-travel time — from your own ephemeris, with its own
citation — and the answer becomes exact. Supply nothing and you get ±499 s, which
is an answer rather than a refusal.

### B2 — *does this correction matter here?*

The generalisation, and cheap once E1 and B1 exist: given a claimed precision,
say which of the corrections are larger than it. Leap seconds (~70 s), the
barycentric term (≤499 s), `TDB−TT` (≤1.7 ms), `TCB−TDB` (0.489 s/yr and
growing). Four numbers and a comparison.

It is a teaching tool as much as a calculator, and the thing it teaches is the
first of the two error classes `S1` opened with.

---

## Class 3 — the sequel to B, and where it stops

### O1 — `O−C` residuals

**The standard tool of variable-star and pulsar work**, and the thing people
actually do with an ephemeris: observed minus calculated, per cycle. Given B's
`Ephemeris` and a list of observed instants, `O−C` is exact integer subtraction,
and each residual is placed against the window B already computes.

No new data, no new physics, and it turns B from a predictor into an instrument.

**Where it stops, and this matters:** reporting `O−C` is exact; **fitting a new
ephemeris to it is not this project's job.** A quadratic trend in `O−C` *is* `Ṗ`,
and extracting it is least squares — floating point, or interval least squares,
which is a research project rather than a feature. Report, do not fit. A tool
that reports residuals honestly and refuses to fit them is more useful than one
that does both adequately, because the fit is where a reader most needs to know
which code produced the number.

### O2 — phase folding

Many instants to phases against one ephemeris, for a light curve. `cycle_at`
already does one; this is the loop and a stable output shape.

---

## Class 4 — extending `dilate`

### D1 — the circular orbit

A *static* observer at `r` runs at `√(1 − r_s/r)`. An observer in a **circular
orbit** at `r` runs at `√(1 − 3r_s/(2r))` — gravitational and kinematic dilation
combined, and the factor of 3/2 is exactly the kind of thing that is easy to get
wrong and easy to test.

Same arithmetic, same `isqrt` bracket, one more case — and it is the case that
covers GPS clocks, the S2 star around Sgr A*, and any pulsar in a binary. It
roughly triples what `dilate` is good for, for a few lines.

**Stop if** the two cases cannot be told apart in the output. A user who reads a
static factor as an orbital one has a wrong answer that looks right, so the
command must name which it computed rather than emitting a bare number.

---

## What stays refused

- **Ephemeris evaluation.** `S1`'s boundary, unchanged: an exact evaluation of an
  approximate series is a precise wrong answer.
- **Fitting anything.** See O1.
- **`UT1` and Earth orientation.** ΔUT1 is observed and published weekly, and
  Rule C will not let this project invent one offline.
- **A frame for `UC-1`.** `S2` records why that is 2.0: one unsigned integer per
  instant asserts there is one time.

---

## Suggested order

1. **E1** — `TCG`/`TCB`. Largest gap between what is possible and what exists,
   and it turns A2's one honest limitation into three scales where two are exact.
2. **O1** — `O−C`. Turns B into an instrument, and needs no new data at all.
3. **E2** — light-travel time. Small, and the light-year identity is a good
   demonstration of why exact definitions are worth respecting.
4. **D1** and **E3** — a few lines each, alongside whatever else is happening.
5. **B1/B2** — after E1, because the sensitivity question wants all the
   corrections in one place to be worth asking.

Every one is arithmetic on constants that are already exact or already cited.
**None of them needs a new measurement**, which is what distinguishes this list
from `S2`'s — where the stellar timeline was blocked on model provenance and the
black hole catalogue on published masses.
