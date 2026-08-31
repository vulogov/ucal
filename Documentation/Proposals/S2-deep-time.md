# S2 — deep time: black holes, stellar lifetimes, and what this clock can index

**Status: research, measured. Two questions asked, one answered yes with a
feature attached, one answered *partly* with the other part deferred to 2.0 for
a reason that is not a scheduling excuse.**

Companion to [`S1`](S1-astrophysics-roadmap.md), which covers ephemerides and
draws the boundary at *this project must not become an ephemeris library*. This
one is about the other end of the ladder.

---

## First, the domain — because it decides both answers

`UC-1` holds `2^512 − 1` ticks. In units this question needs:

| | |
|---|---|
| domain ceiling | 1.3408 × 10¹⁵⁴ ticks |
| in Julian years | **2.2906 × 10¹⁰³** |
| as a cosmological decade | **η = 103.4** |

The cosmological decade `η = log₁₀(t/yr)` is Adams & Laughlin's scheme, and it
is how the far future is discussed in the literature.

**The Black Hole Era ends at about η = 100**, when the largest black holes
finish evaporating. TON 618, at 6.6 × 10¹⁰ M☉, evaporates at η ≈ 99.8 — which is
26% of this domain, measured in ticks.

So: **`UC-1`'s domain covers the entire history of the universe from the datum
to the end of the Black Hole Era, and stops just inside the Dark Era.**

That is a coincidence and not a design. 512 bits was chosen to put the Planck
tick at the bottom and the observable age comfortably inside; that it lands one
decade past the last black hole is arithmetic nobody arranged. It is worth
stating because it answers the question *what can this clock index* exactly:
**everything with a time in it, up to η ≈ 103, and nothing beyond.**

### And what the catalogue actually covers

`ucal timeline` calls itself *the whole of time, in one document*. Its nineteen
events run from inflation to the bridge epoch:

| | |
|---|---|
| catalogue | η ≤ 10.1 |
| domain | η ≤ 103.4 |
| fraction of the domain, linearly | **6 × 10⁻⁹⁴** |

**Ten decades of a hundred and three, and every event is in the past.** The
catalogue has no entry after the present. That is the finding both of these
questions run into before they run into anything else, and it is the cheapest
thing on this page to fix.

---

## Black holes: three questions, three different answers

### 1. Dynamical and observational timescales — supported today, needing a catalogue

The light-crossing time of a horizon, `r_s/c = 2GM/c³ = 9.85 μs × (M/M☉)`:

| | | |
|---|---|---|
| 10 M☉ | 98.5 μs | 1.83 × 10³⁹ ticks |
| Sgr A* (4.3 × 10⁶ M☉) | 42.4 s | 7.86 × 10⁴⁴ ticks |
| M87* (6.5 × 10⁹ M☉) | 17.8 h | 1.19 × 10⁴⁸ ticks |

Nine orders of magnitude, every one an exact integer, all sitting on the tier
ladder where they can be compared without an Earth unit in sight. Ringdown
damping times, ISCO orbital periods and EHT observing cadences are all in the
same band.

**This needs no new arithmetic.** `between`, `add`, `ruler` and the ladder do it
now. What is missing is *data*: a black hole is not a `Body` — it has no
rotation period and no solar day, so `ucal-body` cannot hold one — and there is
nowhere to put a cited mass.

**The feature is a catalogue**, in the shape `ucal-events` already has: cited,
interval-valued entries, each carrying `M`, and deriving `r_s/c` and the ISCO
period from it exactly. Masses come with real uncertainties (Sgr A* is
4.297 ± 0.013 × 10⁶ M☉ from GRAVITY) and the derived times inherit them as
windows, which is Rule U doing the obvious thing.

### 2. Evaporation and the far future — a catalogue entry with a caveat that matters

`t_evap ≈ 2.1 × 10⁶⁷ yr × (M/M☉)³`, and every value is inside the domain:

| | η | ticks |
|---|---|---|
| 10 M☉ | 70.3 | 1.23 × 10¹²¹ |
| Sgr A* | 87.2 | 9.78 × 10¹³⁷ |
| M87* | 96.8 | 3.38 × 10¹⁴⁷ |
| TON 618 | 99.8 | 3.54 × 10¹⁵⁰ |

**The caveat is the interesting part, and it is a Rule C caveat.** That formula
is the evaporation time of a hole radiating into empty space. A black hole
cannot evaporate while it is colder than its surroundings, and a stellar-mass
hole is far colder than the CMB — around 6 × 10⁻⁹ K against 2.7 K. Evaporation
does not *begin* until the CMB has cooled below the hole's temperature, which
for a stellar-mass hole is somewhere around η ≈ 19–20 and depends on the
expansion history.

So `t_evap` is **not** the time until the hole is gone; it is the duration of a
process that has not started. A catalogue entry that reports it as a date would
be making a claim its source does not. The honest entry is an interval with the
onset condition stated — which is exactly the shape `ucal-events` already uses
for `inflation` (*10⁻³⁶ to 10⁻³² s*, as published) and exactly the discipline
`UCAL-W0006` already enforces elsewhere.

**Recommended, and small.** Adams & Laughlin (1997), *A dying universe*,
Rev. Mod. Phys. **69**, 337 is the citable source for the whole far-future
sequence and for the decade scheme itself. Roughly eight entries — degenerate
era, proton decay if it happens, black hole era, the last evaporation — take
`timeline` from covering 10 decades to covering 100, and it is the one change
that would make *the whole of time* a description rather than a title.

### 3. Proper time against coordinate time — the real question, and a 2.0 one

**This is where the honest answer is a limitation.**

Tick 0 is stipulated as the FLRW `t → 0` limit. That makes absolute time here a
*cosmological coordinate*, and `ucal` has therefore always been a coordinate
clock in one frame without ever having had to say so — because until now nothing
it described had a competing clock.

Black holes have nothing else. `dτ/dt = √(1 − r_s/r)`, which goes to zero at the
horizon: an infalling observer reaches the singularity in **finite proper time**
while coordinate time diverges. Two worldlines from the same event to the same
event disagree about how much time passed, and neither is wrong.

**`ucal` cannot express that and must not pretend to.** A single unsigned
integer per instant is a statement that there is one time, and that is the thing
general relativity denies. Papering over it would be worse than the gap.

The right shape is a **second profile**. The mechanism exists — `--profile`, and
§19.4 already distinguishes kinds — and `UC-1` would gain a stated frame rather
than an implicit one. **That is a 2.0 question**, because giving `UC-1` a frame
is a change to what a tick *means*, and belongs in
[`ROAD-TO-2.0.md`](ROAD-TO-2.0.md) rather than in a minor release.

### What *is* buildable in 1.x, and is genuinely unusual

**The dilation factor as a certified interval.** `ucal-core` already has
`isqrt_floor` and `isqrt_ceil`, which is precisely the primitive for bracketing
`√(1 − r_s/r)` in exact rationals with no float anywhere:

```
ucal dilate --rs-over-r 1/2
  factor   [0.7071067811865475244…, 0.7071067811865475245…]   certified
  one second of proper time there is
  [1.41421356…, 1.41421357…] s of coordinate time here
```

An enclosure that is *proved* to contain the answer rather than an iterate that
stopped moving — the same standard `ucal-cosmo` holds its ΛCDM quadrature to,
and something no floating-point tool offers at all. It states the ratio between
two clocks without claiming that either is *the* clock, which keeps it inside
what `UC-1` can honestly say.

**Stop if** the interval cannot be narrowed below the uncertainty on `M` for any
real object. Then certification is decorative — the mass dominates — and the
finding is that the answer was never precision-limited, which is worth writing
down and is a reason not to build it.

---

## Stars: formation to death

### Everything is representable, comfortably

| | η | ticks |
|---|---|---|
| 40 M☉ O star, main sequence | 6 | 5.85 × 10⁵⁶ |
| the Sun, main sequence | 10 | 5.85 × 10⁶⁰ |
| 0.1 M☉ red dwarf | 13 | 5.85 × 10⁶³ |
| last star formation | 14 | 5.85 × 10⁶⁴ |
| white dwarfs cool to black dwarfs | 15 | 5.85 × 10⁶⁵ |

Thirteen decades of stellar life, from a massive star's million years to a red
dwarf's ten trillion, inside a domain that reaches η = 103. **No representation
problem exists here at all.** The problem is entirely one of provenance.

### The shape is a timeline, not a calendar

A star has a rotation period, and no second period to divide it by, so Rule K
derives nothing: there is no stellar analogue of a solar day and a year, and
`ucal-body` correctly has nowhere to put one. Forcing a calendar onto a star
would be the mistake D-A5 refuses when it says *"month-like" is an Earth
predicate* — imposing a familiar structure because it is familiar.

What a star has is a **sequence of phases with durations**: pre-main-sequence,
main sequence, subgiant, red giant branch, helium flash, horizontal branch,
asymptotic giant branch, planetary nebula, white dwarf cooling. That is a
timeline, and `ucal-events` is already a cited interval-valued timeline. The
feature is that machinery pointed at one object rather than at the universe.

### The obstacle, stated before anyone starts

**Stellar phase durations are model outputs, not measurements.** They come from
evolution codes — MIST, PARSEC, Geneva — and they move with mass, metallicity,
rotation, convective overshoot and mass-loss prescription. Rule C wants a
citation *and a validity window*, and here the validity window is not an
interval of time: it is a region of the **model grid**.

That is a different shape from anything `RatedParam` currently carries, and it
is the interesting design problem rather than an obstacle to route around. The
precedent is D-A5: *the grouping satellite is the calendar's declaration*, not
the body's. Here, **the model is the timeline's declaration** and must be named
in the file, so that two timelines for the same star from two grids are two
declarations rather than a contradiction.

### The one number that is a measurement

**The Sun's age is dated, not modelled**: 4.567 Gyr from lead-isotope dating of
CAIs (Bouvier & Wadhwa 2010) — which this project already cites, for the
`solar-system` event. That makes it the right anchor for a demonstration, and
the one place where a stellar timeline can be pinned to something with an
observational uncertainty rather than a grid coordinate.

`ucal star sun --at now` — *4.567 Gyr old, 46% through the main sequence, next
boundary at …, on the MIST grid at Z = 0.0142* — with the model cited and
`UCAL-W0003` when asked outside its range. The warning is not decoration: asking
a solar-metallicity grid about a metal-poor star is exactly the case it exists
for.

**Stop if** the phase boundaries cannot be quoted verbatim from a published
grid. Rule Y.1 wants the figure as published, and an interpolated boundary read
off somebody's plot is not one. Then the honest outcome is that a stellar
timeline needs the grid file itself as an input — which is a §15.1-shaped
problem this project already knows how to solve — and the feature waits for
somebody to supply one.

---

## What this adds up to, and in what order

1. **Extend the event catalogue past the present** — eight or so cited entries,
   Adams & Laughlin for the far future, and `timeline` covers 100 decades
   instead of 10. Cheapest thing here and the one that fixes an existing title
   that overclaims.
2. **A black hole catalogue** — cited masses, derived `r_s/c` and ISCO periods
   as windows. Small, uses machinery that exists, and demonstrates the ladder
   across nine orders of magnitude in one screen.
3. **`ucal dilate`** — certified `√(1 − r_s/r)` by `isqrt` bracketing. Unusual,
   float-free, and honest about being a ratio rather than a clock.
4. **Stellar phase timelines** — the largest, and blocked on the model-grid
   provenance question rather than on any arithmetic.
5. **A frame for `UC-1`** — 2.0, and the reason is that it changes what a tick
   means.

**None of this requires the domain to grow**, which is the answer to the
question behind both: the clock is already long enough. What it lacks is
entries, and one honest sentence about which frame it is keeping time in.
