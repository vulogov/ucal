# S5 — more for physicists, with the surface checked first

**Status: a brainstorm. Every item below was confirmed absent from the command
surface and the library before it was proposed, which is a discipline this series
acquired the hard way — [`S2`](S2-deep-time.md) proposed extending the event
catalogue past the present when it already reached cosmological decade 100, and
the correction is recorded there.**

Fifth of the series after [`S1`](S1-astrophysics-roadmap.md) (ephemerides and the
boundary), [`S2`](S2-deep-time.md) (deep time), [`S3`](S3-exact-bounded-fitted.md)
(exact, bounded, fitted) and [`S4`](S4-outside-astronomy.md) (outside astronomy).

---

## What is already there, so that nothing here repeats it

`now datum from-jd to-jd from-civil to-civil explain between ladder cal show
ephem events timeline add ruler dilate lighttime cosmo verify tour seq wallclock
completions man doctor`.

`cosmo` has **`age`**, **`z`** and **`model`** and nothing else. `dilate` has the
static and circular-orbit cases. There is no epoch notation, no distance, no
kinematic dilation, and no general precision tool.

---

## The strongest four

### V1 — comoving and luminosity distance, by the quadrature that is already there

`ucal-cosmo` computes `t(z)` by **certified integer interval quadrature**, and
comoving distance is *the same integral with a different integrand*:

```text
t(z) = (1/H₀) ∫ dz' / [(1+z') E(z')]        already built
D_C(z) = (c/H₀) ∫ dz' / E(z')               the same machinery, one factor out
```

Everything needed exists: the substitution, the interval arithmetic, the depth
and scale controls, the audit trail, and — since 1.12.0 — an exact `c` in
[`lighttime`](../CLI.md#ucal-lighttime). Luminosity distance is `(1+z)·D_C` for a
flat model, which is exact once `D_C` is an interval.

**This is the single most-used number in observational cosmology**, and every
online calculator produces it in floating point with no error bound at all. A
*certified* comoving distance — proved to contain the answer — is something no
other tool offers, and it costs one integrand.

**Stop if** the enclosure at usable depth is wider than the uncertainty on `H₀`
itself. Then the quadrature is not the limiting factor, the parameters are, and
the finding is that certification buys nothing here — which is worth knowing and
is the same shape as `dilate`'s stop condition.

### V2 — epoch notation, including the Besselian trap

Astronomers write `J2000.0`, `B1950.0`, `J1991.25` (Hipparcos) and `2016.0`
(Gaia DR3) constantly, and **this program understands none of them**.

The trap is that `J` and `B` are different years. A Julian epoch counts Julian
years of exactly 365.25 days from J2000.0; a Besselian epoch counts *tropical*
years of 365.2421988 days from a different origin. `B1950.0` and `J1950.0` are
**1.84 hours apart** — computed, not remembered: `JD 2433282.42346` against
`JD 2433282.50000`. Catalogue positions are still published against B1950 in
older literature.

*(The first draft of this paragraph said eighteen hours. It is 1.84, and the
figure was checked before this file was committed rather than after — which is
the discipline the header describes, applied to the file that describes it.)*

```
ucal from-epoch J2000.0 --scale tt
ucal from-epoch B1950.0 --scale tt
ucal from-epoch 2016.0  --scale tt     # Gaia DR3
```

This is A1's discipline applied to the other notation the field uses: **the scale
stays mandatory**, and the `J`/`B` prefix is required rather than defaulted,
because guessing it is a near-two-hour error that looks like nothing.

**Stop if** the Besselian definition cannot be quoted verbatim under Rule Y.1.
It is a published constant, so this should not fire — and if it does, `J` ships
alone and the refusal for `B` says why, which is the shape `from-jd` already uses
for `UTC` and `UT1`.

### V3 — kinematic dilation, completing the trio

`dilate` covers the static observer and the circular orbit. The third case is the
one every physicist meets first:

```text
dτ/dt = √(1 − β²)        β = v/c
```

Same `isqrt` bracket, same outward rounding, one more argument. It completes a
set that is currently missing its most familiar member, and it covers the cases
`dilate` cannot reach today — a spacecraft, a beam, a cosmic-ray muon.

**And it is a weak-field cancellation case again.** At `β = 10⁻⁴` — a fast
spacecraft — `1 − √(1−β²)` is `5 × 10⁻⁹`, and computing it in `f64` by that
expression loses the same half of the mantissa `dilate`'s doc comment measured
for the Sun. The certified bracket does not.

**Stop if** it cannot be told apart from the gravitational case in the output. A
reader who takes one for the other has a wrong answer that looks right, which is
exactly why `--orbiting` names which formula it used.

### V4 — cosmological time stretch, which is exact

A distant event's observed duration is `(1 + z)` times its emitted duration.
Exactly — no integral, no model, no parameters. It is why supernova light curves
are stretched, and it is a **measured confirmation of expansion**, not a
derivation from one.

```
ucal cosmo stretch --z 1.5 --emitted <DELTA>
```

If `z` is rational the answer is exact, and `Delta` arithmetic is already
integer. This is perhaps five lines, it needs no cosmology at all, and it is the
one quantity in this whole area that a reader can check by hand.

**Stop if** it turns out `between` already composes to give it. It does not
today — nothing multiplies a `Delta` by a rational at the surface — but that
would be the better fix if it did.

---

## Two that are smaller, and one that is bigger

### V5 — a general last-digit consistency check

`ephem validate` asks whether a period's quoted precision and its quoted σ agree.
`cal validate` asks the same of a body's parameters. **Neither is available for a
number a physicist has in their hand.**

```
ucal figure 3.52 --sigma 0.00000038
  the last digit of the value is 10^-2
  the stated uncertainty is 3.8 x 10^-7
  INCONSISTENT: sigma is 26000 times finer than the value's last place
```

[`S4`](S4-outside-astronomy.md) found metrology the strongest non-astronomy fit
for this project, and this is the smallest useful thing on that path. It needs no
time at all, which is either an argument that it belongs here — the discipline is
the product — or an argument that it does not.

### V6 — sidereal and solar day, side by side

`ucal-body` holds a rotation period and a solar day, and the relation between
them is already implemented as the `synodic` derivation. What `cal show` does not
do is put both on screen with the statement that **one is the body's spin and the
other is its day**, which is the distinction every planetary-science
undergraduate gets wrong once. Reporting, not arithmetic.

### V7 — the one that is bigger: `cosmo` for a non-flat model

Everything in `ucal-cosmo` assumes flat ΛCDM. Curvature changes the distance
integral's *form* — sinh, sin or identity depending on the sign of `Ω_k` — and
`sinh` is not something integer interval arithmetic reaches cheaply.

Recorded so it is a decision rather than an omission. **The honest position is
probably that flatness stays an assumption and the model says so**, which it
already does — `cosmo model` names the parameter set. If V1 is built, the flat
assumption becomes load-bearing for a *distance* rather than only for an age, and
that is worth restating at the point where it starts to matter more.

---

## What stays refused, unchanged

Ephemeris evaluation (`S1`), fitting anything (`S3`), `UT1` and Earth orientation
(Rule C, no offline value to cite), and a frame for `UC-1` (`S2`, and it is 2.0).

---

## Suggested order

1. **V4** — exact, tiny, and checkable by hand.
2. **V3** — completes `dilate`, and reuses its bracket exactly.
3. **V2** — the Besselian trap is a real 1.84-hour error, and A1's pattern
   applies unchanged.
4. **V1** — the largest, the most used, and the one with a genuine stop
   condition to measure first.
5. **V5** and **V6** alongside; **V7** recorded and not built.

Every one of these is arithmetic on constants that are already exact or already
cited. **None needs a new measurement** — which is the same property that
distinguished `S3`'s list, and is why the list is short.
