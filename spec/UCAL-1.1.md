<!--
  NORMATIVE. This is the corrected specification.

  It is RFC UCAL-1 with the seventeen standing deltas applied in place. The
  original is kept verbatim at RFC-UCAL-1.md; the reasoning behind each change
  is in SPEC-DELTAS.md. Section and rule numbering is unchanged from the
  original and MUST NOT be renumbered: the implementation cites it 494 times.
-->

> ## UCAL-1.1 — the normative specification
>
> RFC UCAL-1 with the **seventeen standing deltas applied in place**. Every
> amended passage is marked inline with the delta that changed it and its class:
>
> | class | meaning | count |
> |---|---|---|
> | **CORRECTION** | the original is wrong | 6 |
> | **AMENDMENT** | a normative change adopted by decision | 6 |
> | **EDITORIAL** | no behavioural effect | 5 |
>
> The original text is preserved at [`RFC-UCAL-1.md`](RFC-UCAL-1.md) — a
> correction is only meaningful against the text it corrects. The reasoning for
> each change, at length, is in [`SPEC-DELTAS.md`](SPEC-DELTAS.md). The rule
> vocabulary is in [`RULES.md`](RULES.md).
>
> One delta is **withdrawn** and deliberately still visible: D-A1 claimed the
> 44 BC fixture was a day late, and the fault turned out to be in the checking
> oracle. A retracted claim is part of the record.
>
> **Numbering is frozen.** Amendments change the text in place; they never
> renumber it.

---

# RFC UCAL-1 — Universe Calendar

**FINAL DRAFT — implementation-ready**

| Field | Value |
|---|---|
| RFC | UCAL-1 |
| Title | Universe Calendar: absolute time in Planck ticks with a base-5 tier calendar |
| Author | Vladimir Ulogov |
| Status | Final draft — implementation-ready |
| Date | 2026-07-29 |
| Crate | `ucal` |
| Supersedes | UCAL-1 rev 1, rev 2, rev 3, rev 4; `UNIVERSE_CALENDAR_PROPOSAL.md`; `UNIVERSE_CALENDAR_PROPOSAL_v2.md` |
| License | MIT OR Apache-2.0 |

This document is authoritative. All prior revisions and proposals are void. Every open question carried by rev 4 is closed here as a decision (§23); the remaining uncertainty is confined to six gated experiments (§24) that affect implementation choices, not the specification.

The key words MUST, MUST NOT, REQUIRED, SHALL, SHOULD, SHOULD NOT, MAY, and OPTIONAL are to be interpreted as described in RFC 2119.

---

## Section map

| Part | Sections | Content |
|---|---|---|
| **0** | 0.1–0.5 | Problem, goals, non-goals, failure modes and their metrics, terminology |
| **A** | 1–11 | Normative specification: 24 named rules |
| **B** | 12–18 | Library implementation: crates, algorithms, public API contract |
| **C** | 19 | CLI |
| **D** | 20–24 | Delivery: phases, tests, conformance, decisions, gated experiments |
| **Appendices** | A–I | Constants, tiers, fixtures, locales, diagnostics, codec, schemas, numerics, derived calendars |

**Rule quick index:** Q (datum) · A (atomicity) · Y (metrology) · Z (zero) · M (order) · F (frame) · P (profile) · W (width) · O (overflow) · E (integrality) · R (rendering) · G (grid) · N (names) · T (truncation) · U (uncertainty) · D (text forms) · S (sort) · B (binary) · I (UCID) · L (leap seconds) · K (derivation) · J (anchor) · C (body parameters) · X (enclosures)

---

# Part 0 — Rationale and scope

## 0.1 Problem

Time for events outside the Earth timeline is expressed in units defined by the rotation and orbit of one planet, anchored to epochs of local cultural significance, and mediated by a civil calendar with irregular months, leap years, leap seconds, and a discontinuity at 1582. Every cosmological statement is therefore a silent conversion: "380,000 years after the Big Bang" means 380,000 × 365.25 Earth days, a quantity with no physical relationship to the event described.

This RFC specifies a time system with no Earth content in its arithmetic — an unsigned integer count of Planck-time units since a stipulated datum near the origin of the universe, and a positional calendar over that integer in base 5 — together with **one** mechanism for deriving a local calendar for any rotating, orbiting body, of which Earth is an instance rather than the template.

## 0.2 Goals

- **G1** — A single unsigned integer, exact and reproducible, identifies any instant from the datum to 2.29×10¹⁰³ years after it.
- **G2** — The finest addressable interval is one tick. Any event describable at that resolution can be pinpointed exactly.
- **G3** — The calendar is a positional numeral system, not a conversion table: truncation is rounding, prefix comparison is chronological comparison, concatenation is refinement.
- **G4** — The tick is primitive in definition and in computation. Foreign units are derived, optional, and confined to a declared bridge.
- **G5** — One calendar-derivation mechanism serves every body. Earth is an instance of it, not an exception to it.
- **G6** — All arithmetic is integer arithmetic. No floating-point type appears anywhere in the workspace.
- **G7** — Uncertainty is first-class: cosmological results, calendar anchors, and the datum's own physical identification are certified intervals, and the type system keeps them out of the arithmetic.

## 0.3 Non-goals

| # | Non-goal |
|---|---|
| N1 | Planck time is **not** asserted to be a quantum of time. It is a natural unit chosen as the resolution limit of this system. |
| N2 | No relativistic model. Absolute time is not observer-invariant; velocity and gravitational dilation are out of scope. |
| N3 | No claim that the datum is a physically meaningful instant. Classical time is not defined below ~1 tick. |
| N4 | Not a replacement for `chrono`, `time`, or `hifitime` for civil timekeeping. |
| N5 | No timezone database, no locale-aware civil formatting, no business-day arithmetic. |
| N6 | No ephemerides. Body parameters are static, optionally with a first-order rate; orbits are not integrated. |
| N7 | UCID is not a UUID substitute and carries no randomness. |
| N8 | No cosmological model fitting. `ucal-cosmo` evaluates a declared model; it does not infer parameters. |
| N9 | Profile `UC-1` does not track future revisions of the measured age of the universe. Revisions create new profiles. |
| N10 | No sub-tick representation. Intervals shorter than one tick are not representable and MUST NOT be approximated. |
| N11 | Not a monotonic clock source or a scheduling primitive. |
| N12 | No time before the datum. The value domain is unsigned. |
| N13 | No transcendental function evaluation anywhere. Quantities requiring one are computed as certified enclosures by quadrature. |
| N14 | No claim that the tick is independent of Earth *metrology*. Its ratio to the SI second is a declared convention (Rule Y). |
| N15 | **Phase is not derived.** A calendar's anchor is an empirical constant per body with an uncertainty window (Rule J). |
| N16 | **The civil Gregorian calendar is not derivable and is not derived.** It is declared legacy interop data (§8.6), outside Rule K. |
| N17 | **The datum is not measured and is not the Big Bang.** Tick 0 is a stipulation (Rule Q); its relationship to the FLRW t→0 limit is a separately declared claim that never enters a computation. |

## 0.4 Failure modes, mechanisms, and metrics

Each failure mode maps to a mechanism that prevents it and to the artifact that proves the mechanism works. If a row has no green metric, the mechanism is unproven.

| # | Failure | Mechanism | Metric |
|---|---|---|---|
| F1 | Timestamps shift when the age constant is revised | Rule P: profiles named, versioned, type-bound, tagged in every serialized form | Compile-fail test: cross-profile arithmetic rejected |
| F2 | Precision invented by zero-filling a truncated timestamp | Rule T: truncation yields `Window` | No parse path returns tick-precision from truncated input (exhaustive parse test) |
| F3 | Foreign units inject rounding into the core | Rules A, E, Y: tick-primitive constants, exact integer bridge | 10⁶ random civil instants convert with zero rounding |
| F4 | Leap seconds leak into absolute-time arithmetic | Rule L: TT-only pivot | Differential vs `hifitime` at every leap-second boundary |
| F5 | Backend change silently changes the wire format | Rule B: fixed 64-byte canonical binary | Both backends emit byte-identical encodings for every fixture |
| F6 | A derived calendar drifts because parameters were treated as constants | Rule C: epoch, rate, validity window, as-measured value | `UCAL-W0003` fires on out-of-window evaluation |
| F7 | Overflow wraps instead of erroring | Rules O, W: closed checked domain on both backends | Fuzz at `DOMAIN_MAX ± 1`; no wrapping op exposed |
| F8 | Float error and parameter uncertainty conflated into one tolerance | Rules E, X: interval arithmetic, widths reported separately | `CosmoResult` carries both widths; float lint green |
| F9 | Earth becomes the template rather than an instance | Rule K: one mechanism; legacy quarantined | `earth-d` and `mars-d` built by the identical generic path (test constructs both from data alone) |
| F10 | A body calendar with no anchor silently phases off Earth | Rule J: anchors required, structured, body-defined | `UCAL-E0062`; no constructor accepts a calendar without an anchor |
| F11 | Exactness of the arithmetic mistaken for accuracy about the origin; the ±0.020 Gyr uncertainty leaks into timestamps | Rule Q: datum stipulated; claim declared separately and non-consumable | Compile-fail test: `SignedWindow` cannot reach `Instant`/`Delta` arithmetic |
| F12 | The datum's Earth-flavoured provenance hidden in prose, unauditable | Rule Q.4: `datum_provenance` is machine-readable data | Provenance chain re-executes to the declared `ORIGIN_OFFSET` and residual |
| F13 | Prose drift reintroduces overclaims the spec removed | Rule Q.1 + documentation lint | Lint fails on "creation of the universe" / "age of the universe is" as descriptions of tick 0 |

## 0.5 Terminology

- **tick** — the primitive unit; one Planck time under the active profile.
- **absolute time** — an unsigned integer count of ticks since the datum.
- **datum / absolute zero** — tick 0. A **stipulated** reference point, conventionally identified with the FLRW t→0 limit. Not a measurement, not an observed event (Rule Q, N17).
- **`BIG_BANG_CLAIM`** — the declared signed tick window within which a profile asserts the FLRW t→0 limit lies relative to its datum. Metadata; never an operand.
- **tier** — a power of 5 on the grid 5^(5k); the calendar's digit-group scale.
- **beat** — the base tier, 5⁶⁰ ticks; the "universe second".
- **bridge constant** — a profile constant whose sole purpose is conversion to a foreign unit system (`SECOND`).
- **instant / delta / window** — a point / an unsigned magnitude / a closed interval in absolute time.
- **anchor** — the tick value plus phase definition fixing where a calendar's counting begins (Rule J).
- **cycle** — an optional grouping period within a calendar year, derived from a satellite's synodic period.
- **derived calendar** — a calendar produced by Rule K. **legacy calendar** — declared tables preserved for interop (§8.6).
- **profile** — the versioned constant set fixing the datum and the tick.
- **UCID** — the fixed-width sortable text identifier of an instant.

---

# Part A — Normative specification

## 1. Model, frame, primacy, datum

**1.1 Frame.** Absolute time is proper time along a comoving worldline in an FLRW frame — cosmological time in the CMB rest frame. This is the frame in which "the universe is 13.787 Gyr old" is a meaningful statement.

**Rule F (Frame).** Every profile MUST declare its frame. Implementations MUST NOT convert between frames and MUST NOT claim observer-independence.

**1.2 Domain.** Absolute time is unsigned and increases without bound within the profile domain.

**Rule Z (Zero).** Tick 0 is the datum. No value precedes it. Any operation whose result would be negative MUST fail with `UCAL-E0020`, never wrap and never saturate.

**Rule M (Monotone total order).** For instants a, b in one profile, exactly one of a < b, a = b, a > b holds, and it is the chronological order.

**1.3 The datum.**

**Rule Q (Datum).**

1. **Tick 0 is a stipulated datum.** Exact by declaration, unrevisable within a profile. Implementations and documentation MUST NOT describe it as measured, derived, observed, or as "the creation of the universe". Permitted phrasing: *the datum, conventionally identified with the FLRW t→0 limit*.
2. The stipulation is a necessity, not a shortcut, for three independent reasons, each sufficient:
   - **Exactness cannot come from measurement.** The published age carries ±0.020 Gyr — 1.170698×10⁵⁸ ticks, 0.145% of the span. Measurement never yields a single integer, and a datum inheriting the error bar would make every timestamp uncertain relative to zero, destroying G1.
   - **The t→0 limit is not an observable event.** It is where the FLRW extrapolation's coordinates degenerate, and by N3 classical time is undefined below roughly one tick.
   - **The extrapolation is model-dependent.** Under inflation the FLRW t→0 limit is not a physical event at all.
   This places the datum in ordinary company: TAI's 1958-01-01, the Julian Day epoch at −4712, the Unix epoch. The parallel to the SI second is exact — 9 192 631 770 caesium cycles was *chosen* to match the ephemeris second, and the definition does not inherit that provenance.
3. **A profile MUST declare `BIG_BANG_CLAIM`**: a signed tick window, relative to its own datum, within which it asserts the FLRW t→0 limit lies, with citation. The window is signed because the limit may lie before the datum, which is not representable as a tick (N12). **No arithmetic operation may consume it.** It is reportable metadata only — `ucal datum`, `ucal doctor`, `ucal explain --claim` — surfaced so a user learns that the physical *interpretation* is uncertain while the *arithmetic* is exact. Use as an operand is `UCAL-E0025`, and §21.3 requires a compile-fail test proving the type cannot reach arithmetic.
4. **A profile MUST carry a machine-readable `datum_provenance` record**: the empirical input with unit and citation, the exact conversion chain, and the rounding applied. Provenance is data, not prose — auditable, re-executable, and replaceable without editing specification text. Absence is `UCAL-E0013`.
5. Changing the datum, `BIG_BANG_CLAIM`, or `datum_provenance` produces a **new profile**; Rule P then keeps values from the two from mixing.

**1.4 Primacy.**

**Rule A (Atomicity).** The tick is the only primitive unit of this specification.

1. Every profile constant MUST be declared as an exact non-negative integer count of ticks. No profile constant may be expressed in seconds, days, years, or any other foreign unit.
2. `ucal-core` MUST NOT reference, name, or define any Earth-derived quantity. The Julian year, the day, the hour, and every civil calendar live outside it.
3. Conversion to a foreign unit system is permitted only through a constant explicitly declared a **bridge constant**. Profile `UC-1` declares exactly one: `SECOND`.
4. A bridge constant MUST be an exact integer number of ticks, so conversion *into* absolute time is multiplication and never requires rounding.
5. Quantities derived from a bridge constant (the tick's length in seconds, the implied age in Gyr) are **informative**: documented as consequences, never inputs to a computation.

**1.5 Metrology.**

**Rule Y (Metrology).** Empirical inputs may arrive in foreign units, because that is how measurement works. This is permitted at exactly three points and nowhere else — datum provenance (Rule Q.4), body parameters (Rule C), cosmological model parameters (§10.2) — and at each of them:

1. the foreign-unit value MUST be recorded verbatim with its unit and citation;
2. conversion into ticks MUST be exact through a declared bridge constant, or rejected (`UCAL-E0043`);
3. the **declared constant** — the value the specification and the code use — is the tick value;
4. no computation may consume a foreign-unit quantity, and no result may be expressed in one except by rendering (Rule R).

Rule Y is the boundary that makes Rules A, Q, C, and J satisfiable in practice. It concedes metrology (N14) and nothing else: the direction of definition and the location of rounding remain as Rule A specifies. The same definition/determination split governs Rule J — an anchor's phase is *defined* by a physical event of its body, while its *determination* rests on observation timestamped in whatever scale the observers used (§9.4).

## 2. Profile UC-1

**2.1** A profile is an immutable, named constant set. `UC-1` is normative; Appendix A is the complete listing.

```
Profile UC-1
  frame            FLRW comoving (cosmological time)
  datum:
    BEAT           5^60 ticks                                       (primitive)
    ORIGIN_OFFSET  9 304 311 741 502 590 385 × BEAT ticks           (primitive)
    DOMAIN         [0, 2^512)                                       (primitive)
    BIG_BANG_CLAIM datum ± 631 152 × 18 548 584 399 861 × 10^39 ticks
                   = ± 1.170 698×10^58 ticks  (± 0.020 Gyr; Planck 2018)
  bridge:
    SECOND         18 548 584 399 861 × 10^30 ticks                 (bridge constant, exact)
    SI_EPOCH       0000-01-01T00:00:00.000 TT, proleptic Gregorian,
                   astronomical year numbering  ≡ tick ORIGIN_OFFSET
  datum_provenance:  §2.2
  informative (derived, never an input):
    tick length    1 / SECOND = 5.391 247 000 000 139 6 × 10^-44 s
    implied age    ORIGIN_OFFSET / SECOND = 13.787 000 000 000 Gyr of Julian years

> **[D-A10 · EDITORIAL]** The implied age quoted here is the **unrounded input**, not the quotient. `ORIGIN_OFFSET` was rounded half-even to a whole beat, so `ORIGIN_OFFSET / SECOND` differs from the input by the recorded residual of −0.017190364 s. The distinction matters because Rule Q.4 makes the provenance chain auditable, and an audit that cannot see the rounding cannot check it.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).


```

`SI_EPOCH` anchors the **bridge**, not the datum and not any calendar: the profile asserts "the Earth date year-0 falls at tick `ORIGIN_OFFSET`" — a statement about Earth. Calendar anchors are separate (Rule J).

**2.2 `datum_provenance` (normative data).**

```hjson
datum_provenance: {
  input:     { value: "13.787 Gyr", quantity: age_of_universe, uncertainty: "0.020 Gyr",
               source: "Planck 2018 results VI, A&A 641 A6" }
  unit_defs: { Gyr: "10^9 × 31 557 600 s", note: "Julian years, exact by definition" }
  chain: [
    "AGE_s        = 13 787 000 000 × 31 557 600 = 435 084 631 200 000 000 s   (exact)"
    "AGE_ticks    = AGE_s × SECOND
                  = 8070204002895596516263200000000000000000000000000000000000000  (exact)"
    "beats        = round(AGE_ticks / BEAT) = 9 304 311 741 502 590 385"
    "ORIGIN_OFFSET = beats × BEAT
                  = 8070204002895596515944343085635637180530466139316558837890625"
  ]
  rounding:  { to: BEAT, mode: half_even,
               residual: "-318856914364362819469533860683441162109375 ticks = -0.017 190 364 s",
               rationale: "whole-beat datum ⇒ all sub-beat digits of SI_EPOCH are zero (§2.4)" }
  earth_dependency: "The input arrives in Julian years and the bridge anchor is an Earth
                     calendar date. Both are metrology (Rule Y). Neither appears in any
                     computation: ORIGIN_OFFSET is a declared integer of ticks."
  alternative_routes: [
    "A future profile MAY anchor provenance on an observable — e.g. CMB last scattering at
     z = 1089.9 ± 0.4 — and derive the offset to the datum through ucal-cosmo in ticks,
     removing the Julian year and the Earth date from the chain. This improves auditability,
     not exactness: measurement yields a window and a datum is a point, so any route
     terminates in a stipulation (Rule Q.2).  See GE-6."
  ]
}
```

**2.3** `ORIGIN_OFFSET`, `SECOND`, `DOMAIN`, and `BIG_BANG_CLAIM` are not revisable within `UC-1`. Implementations MUST NOT recompute them from published measurements. Revised measurements produce a new profile (Rule Q.5).

**2.4 Alignment invariants (normative consequences).** Because `SECOND` is divisible by 5³⁰ and `ORIGIN_OFFSET` by 5⁶⁰:

- an instant at a whole SI second has zero in tiers T−12 … T−7 (its low 30 base-5 digits);
- an instant at a whole nanosecond has zero in its low 21 base-5 digits;
- `SI_EPOCH` has zero in all tiers below T0.

These are invariant tests (§21.3), not observations.

**2.5 Year numbering (bridge only).** Astronomical numbering: year `0000` **is** 1 BC; `-0001` is 2 BC. Proleptic Gregorian year 0 differs from proleptic Julian year 0 by two days; `SI_EPOCH` is defined on proleptic Gregorian. TT did not exist in year 0; `SI_EPOCH` is a proleptic label, not a clock reading.

**Rule P (Profile binding).** Instants are parameterized by profile at the type level. Cross-profile arithmetic and comparison MUST NOT compile. Every serialized form MUST carry the profile tag. Conversion between profiles is available only through an explicit `rebase` reporting the constant tick shift.

**2.6** A profile MAY declare `DOMAIN: unbounded`, which REQUIRES the `bigint` backend. `UC-1` does not.

## 3. Representation, backends, arithmetic

**Rule W (Width).** The value domain of `UC-1` is `[0, 2^512)` on **every** backend. The `bigint` backend MUST enforce the same ceiling. The backends are therefore behaviorally identical, which makes differential testing between them a conformance test rather than an approximation.

**3.1 Backends.**

| feature | type | properties |
|---|---|---|
| default | `bnum` 512-bit unsigned (`U512`) | stack, `Copy`, `const`-constructible, `no_std`, zero deps |
| `bigint` | `num_bigint::BigUint` | heap, not `Copy`, requires `alloc`, ceiling per Rule W |

```rust
#[cfg(not(feature = "bigint"))] pub type Ticks = bnum::types::U512;
#[cfg(feature = "bigint")]      pub type Ticks = num_bigint::BigUint;
```

512 bits covers 2.290567×10¹⁰³ years — past proton decay (10³⁴–10⁴⁰ yr), stellar black hole evaporation (~10⁶⁷ yr), and heat death (~10¹⁰⁰ yr). The present epoch uses 7×10⁻¹⁷ of the range.

**3.2** Public value types derive `Copy` only on the default backend:

```rust
#[cfg_attr(not(feature = "bigint"), derive(Copy))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Instant<P: Profile> { ticks: Ticks, _p: PhantomData<P> }
```

Enabling `bigint` removes `Copy` from `Instant`, `Delta`, and `Window`. This is a documented API difference; all other semantics are identical.

**3.3** Profile constants MUST be `const` on the default backend:

```rust
pub const BEAT: U512 = U512::parse_str_radix("867361737988403547205962240695953369140625", 10);
pub const ORIGIN_OFFSET: U512 = U512::parse_str_radix(
    "8070204002895596515944343085635637180530466139316558837890625", 10);
pub const SECOND: U512 = U512::parse_str_radix(
    "18548584399861000000000000000000000000000000", 10);
const BIG_BANG_CLAIM_HALFWIDTH: U512 = U512::parse_str_radix(
    "11706976141141069872000000000000000000000000000000000000000", 10);
pub const TIER: [U512; 45] = /* 5^0, 5^5, …, 5^220 — generated, never transcribed */;
```

`BIG_BANG_CLAIM_HALFWIDTH` is private and reachable only through `Profile::big_bang_claim() -> SignedWindow` (Rule Q.3). On the `bigint` backend these become `LazyLock`. The tier table MUST be generated from the exponent list by `build.rs` or a const routine.

**Rule E (Integrality).** No floating-point type — `f32`, `f64`, or any wrapper over one — may appear in any signature, field, constant, or intermediate anywhere in the workspace, `ucal-cosmo` included. Permitted numeric machinery:

1. exact integers (`Ticks`, widened `U1024` intermediates for multiply-then-divide);
2. exact rationals over integers;
3. scaled fixed-point integers with a declared, recorded scale;
4. integer square root with directed rounding (Appendix H);
5. interval pairs of any of the above.

A CI lint MUST fail the build on any float token in any shipped crate. A float reference implementation is permitted in `dev-dependencies` **as a test oracle only** and MUST be marked as such.

**Rule O (Overflow).** Every operation that can leave the domain MUST have a checked form returning `Result`. Wrapping and saturating arithmetic MUST NOT be exposed on time types.

**Rule R (Rendering).** Rounding exists only when rendering an exact internal value into a coarser foreign representation (ticks → a decimal count of nanoseconds, a window → a midpoint, an exact rational → a decimal string). Every such API MUST take an explicit `Rounding` (`Trunc`, `Ceil`, `HalfEven` default, `HalfUp`) and MUST report lossy renderings (`UCAL-W0001`). No API that *constructs* absolute time may round: construction from a foreign unit is exact or it is an error (`UCAL-E0043`).

## 4. Tier grid

**Rule G (Grid).** Tiers are the powers `5^(5k)`. Tier index is relative to the beat: `T[k] = 5^(60 + 5k)`. Each tier is exactly 5 base-5 digits, i.e. 3125 units of the tier below. The consequences are the point of the design: a timestamp is the tick count written in base 5 and grouped in fives, truncation *is* rounding, prefix comparison *is* chronological comparison, and writing all digits pinpoints one tick.

**4.1** Named tiers:

| tier | exponent | ≈ duration | name |
|---|---|---|---|
| T5 | 5⁸⁵ | 441.607 Myr | deep |
| T4 | 5⁸⁰ | 141.314 kyr | drift |
| T3 | 5⁷⁵ | 45.221 yr | span |
| T2 | 5⁷⁰ | 5.285 d | sweep |
| T1 | 5⁶⁵ | 146.130 s | arc |
| **T0** | **5⁶⁰** | **46.762 ms** | **beat** |
| T−1 | 5⁵⁵ | 14.964 µs | flicker |
| T−2 | 5⁵⁰ | 4.788 ns | glint |
| T−3 | 5⁴⁵ | 1.532 ps | spark |
| T−4 … T−12 | 5⁴⁰ … 5⁰ | 4.90×10⁻¹⁶ … 5.39×10⁻⁴⁴ s | unnamed |
| T6 … T32 | 5⁹⁰ … 5²²⁰ | 1.38×10¹² … 2.29×10¹⁰³ yr | unnamed |

The present epoch is 31.22 deeps. For scale, `BIG_BANG_CLAIM`'s half-width is 141.53 drifts: the datum's physical identification is uncertain by ~141 units of a tier the calendar renders exactly.

**4.2** The tier grid is the **universal** ladder: body-independent, and the canonical way to state any duration. Calendar units (§9) are a local overlay, never a replacement.

**Rule N (Names are display-only).** The canonical identity of a tier is its exponent. Names come from a locale table (Appendix D). Implementations MUST accept `T[k]` and `5^e` notation wherever a name is accepted. A name collision within an active table is `UCAL-E0011`; a name **not found** in it is `UCAL-E0014`.

> **[D-A18 · AMENDMENT]** **A build that does not reproduce its own constants needs a code.** §3.3 requires the declared constants to be reproducible and Appendix E named no code for the answer being *no*. `UCAL-E0015`, exit 9. Raised by `ucal verify` and by nothing else.
>
> **[D-A17 · AMENDMENT]** **A lookup miss needs its own code.** The original defined `UCAL-E0011` for a *collision* and named nothing for a *miss*, which is the far commoner event — a person types a tier name that does not exist. An implementation had to invent a code or misuse one, and misusing one is worse: a misused code looks correct in every place a reader might check it.

**4.3** Bridge table (informative, for `ucal explain`): 1 ms = 66.83 flicker · 1 s = 21.385 beat · 1 min = 1283.1 beat · 1 h = 24.64 arc · 1 d = 591.25 arc · 1 week = 1.324 sweep · 1 yr (Julian) = 69.11 sweep · 1 kyr = 22.11 span · 1 Myr = 7.076 drift · 1 Gyr = 2.264 deep.

Nothing on the ladder is near a second or an hour. That is the accepted cost of leaving the Earth paradigm (D-2 rationale); `ucal explain` prints the SI equivalent **on request**.

> **[D-A16 · AMENDMENT]** **The SI equivalent is printed on request, not always.** The original said `ucal explain` "always prints the SI equivalent alongside". An SI second is an Earth unit, and printing one beside every instant is the substitution Rule A.5 exists to refuse — the conversion is available under `--bridge` and is not performed unasked. The two Earth-calendar commands (`to-civil`, `from-civil`) and `datum`'s provenance chain keep theirs unconditionally, the first because a civil label *is* an Earth label and the second because §19.2 requires the audit trail.

## 5. Value types

```rust
pub struct Instant<P: Profile>;                       // a point: one tick
pub struct Delta;                                     // unsigned magnitude in ticks
pub struct Signed { sign: Sign, mag: Delta }
pub struct SignedWindow { lo: Signed, hi: Signed }    // metadata only (Rule Q.3)
pub struct Window<P: Profile> { lo: Instant<P>, hi: Instant<P> }   // closed, lo <= hi
pub enum   Precision { Tick, Tier(i8) }
pub struct Tier(i8);
pub enum   Rounding { Trunc, Ceil, HalfEven, HalfUp }
```

`SignedWindow` has no arithmetic operators, no `From<SignedWindow> for Delta`, and no path to `Window`: it cannot be added to an `Instant` and cannot be widened into one.

**5.1** `Sub` is NOT implemented on `Instant`:

```rust
fn since(&self, earlier: &Self) -> Result<Delta, TimeError>;   // E0020 if earlier > self
fn between(&self, other: &Self) -> Signed;                     // always succeeds
fn checked_add(&self, d: &Delta) -> Result<Self, TimeError>;    // E0021 on domain exit
fn checked_sub(&self, d: &Delta) -> Result<Self, TimeError>;    // E0020 on underflow
```

**Rule T (Truncation is uncertainty).** A value stated to tier precision denotes the closed interval `[v, v + 5^e − 1]`. Parsing a truncated notation MUST yield a `Window`, or an `Instant` tagged with `Precision`; it MUST NOT yield a bare tick-precision `Instant`. Comparison across unequal precision MUST use interval semantics and MUST be able to return indeterminate (`UCAL-E0023`).

**Rule U (Uncertainty propagation).** Window arithmetic is interval arithmetic with outward rounding: `lo` combines with `lo`, `hi` with `hi`. `lo > hi` is `UCAL-E0022`. Windows MUST NOT be silently collapsed; `Window::midpoint(Rounding)` must be called explicitly.

**5.2** `Delta` supports `add`, checked `sub`, `mul_u64`, `div_u64`, `divmod`, `tier_of()` (largest tier ≤ self), and rendering in any tier.

## 6. Text notation


> **[D-A8 · AMENDMENT]** **Each form declares its own anchor, and a printed form denotes an interval.** The human form anchors at **T0** (the group before the `:`); the digit form anchors at **T32**. A form printed to a coarser tier states `Precision::Tier(e)` and denotes the closed interval `[v, v + 5^e − 1]` (Rule T) — it is not a point that happens to have trailing zeros. A form's precision is the tier of its last group, so the human form cannot express a precision coarser than T0.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

**6.1 Human form** — decimal group values, most significant tier first:

```
UC1 0031·0687·2481·2999·3108·2437
```

Groups are decimal integers in `[0, 3124]`, zero-padded to 4 digits, tiers contiguous and descending; the last group's tier is the stated precision. Groups above the highest non-zero tier are omitted unless `--pad`.

**6.2 Digit form** — base-5 digits, five per group; canonical for parsing and sorting:

```
UC1/5 00000.00000.00000.00000.00111.10222.34411.43444. … .00000
```

**Rule D (Two forms, one value).** Both MUST round-trip. The tag distinguishes them: `UC1` = decimal groups (human), `UC1/5` = base-5 digit groups (canonical). Either is accepted on input; they MUST NOT be mixed within one string (`UCAL-E0003`).

**6.3** Separator default `·` (U+00B7), configurable at the library level; `.` MUST be accepted on input for shell-hostile contexts. The separator MUST NOT be a decimal or base-5 digit.

**6.4** A sub-beat part is introduced by `:`; groups after it are tiers T−1 downward:

```
UC1 0031·0687·2481·2999·3108·2437:1104·2790·0251
```

**6.5 Named form** — parseable, not canonical: `31 deep, 687 drift, 2481 span, 2999 sweep, 3108 arc, 2437 beat`.

**6.6 Calendar-qualified form.** A rendering in a local calendar MUST carry the calendar id and its kind:

> **[D-A9 · AMENDMENT]** **A calendar id needs a grammar.** Splitting at the first `:` is ambiguous: in `earth-civil: 2026-07-29T00:00:00Z` the body itself contains colons, so a first-colon split yields the "calendar id" `2026-07-29T00` and **silently succeeds**. A calendar id is therefore `[a-z][a-z0-9]*(-[a-z0-9]+)*` optionally followed by `/` and a decimal revision, and the separator is the first `:` **after a well-formed id**. Anything else is `UCAL-E0001` rather than a plausible wrong answer.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).



```
earth-d/1:   2026-208.4137          # derived Earth calendar, anchor revision 1
earth-civil: 2026-07-29T00:00:00Z   # legacy Gregorian (non-derived)
mars-d/1:    0212-334.0918
```

`-d` marks a Rule K derivation; any id declared `legacy` marks §8.6 data. Emitting a local calendar rendering without this qualifier is `UCAL-E0007`.

**Rule S (Sort order).** Lexicographic order equals chronological order for the binary form (§7.1) and UCID (§7.2). It does NOT hold for text forms unless zero-padded to a fixed tier width. Text and calendar-qualified forms MUST NOT be documented as sortable.

## 7. Binary encoding and identifiers

**7.1 Canonical binary** — exactly **64 bytes, big-endian, zero-padded**, on every backend, for every profile with `DOMAIN = [0, 2^512)`.

**Rule B (Binary form).** Fixed-width and backend-independent. Length-prefixed, minimal, or varint encodings MUST NOT be used canonically. Byte order is chronological order, so the encoding is directly usable as a database key or sort key. Profiles with `DOMAIN: unbounded` MUST define their own encoding and MUST NOT claim `UC-1` compatibility.

**7.2 UCID** — fixed **52-character Crockford base-32** of the 256-bit big-endian value, uppercase, no checksum, no separators.

**Rule I (UCID).** Defined only for instants `< 2^256` (≈1.978×10²⁶ years, past the end of the stelliferous era); outside that range `UCAL-E0031`. UCID sorts lexicographically in chronological order. It contains no randomness — by §2.4 an instant from a nanosecond clock has ≥21 low base-5 zeros — so UCID MUST NOT be documented or used as a unique identifier for concurrent events.

**7.3** `serde` is OPTIONAL and feature-gated: canonical binary for binary formats, §6.1 human form for human-readable formats, always with the profile tag. Calendar-qualified forms MUST NOT be used as a serialization format.

## 8. SI bridge and legacy civil interop

**8.1** The bridge lives only in `ucal-civil` and uses `hifitime` for civil time:

```
Instant ⟷ ticks ⟷ exact rational TT seconds since SI_EPOCH ⟷ hifitime::Epoch ⟷ UTC / TAI / Gregorian
```

`hifitime` is chosen for integer-exact durations, IERS leap seconds, multiple time scales, model-checked correctness, pure Rust, and `no_std` capability.

**8.2 Exactness.** Foreign → absolute is `ORIGIN_OFFSET + s × SECOND` with `s` an exact rational number of TT seconds. For any `s` whose denominator divides 10³⁰ the product is an exact integer and no rounding occurs. Input finer than 10⁻³⁰ s MUST be rejected with `UCAL-E0043`, never rounded. Absolute → foreign yields an exact rational; rendering follows Rule R.

**Rule L (Leap seconds).** TT is the only pivot. Leap seconds exist solely at the UTC parse/format boundary and MUST NOT appear in absolute-time arithmetic. Implementations MUST accept the UTC label `23:59:60` on leap-second instants and MUST document that UTC labels are non-unique across a leap second while absolute time is not.

**8.3 SI duration units**, defined in ticks, in `ucal-civil::si`:

```
NANOSECOND = SECOND / 10^9      MINUTE = 60 × SECOND      HOUR = 3600 × SECOND
DAY_SI     = 86400 × SECOND     WEEK_SI = 604800 × SECOND
YEAR_JULIAN = 31 557 600 × SECOND    YEAR_GREGORIAN_MEAN = 31 556 952 × SECOND
```

These are **SI conveniences, not calendar units**. `DAY_SI` is 86400 SI seconds, not one rotation of Earth; the latter is a `Body` parameter (§9) and is not exactly 86400 s. Conflating them is the error this split exists to prevent. Calendar-dependent quantities (`YEAR_TROPICAL`, `MONTH_SYNODIC_MEAN`) are not unit-like and belong to `ucal-body`; no API named `to_years()` or `to_months()` may exist without an explicit definition parameter.

**8.4** `ucal now` reads the system clock as UTC and converts through the bundled leap-second table. The table version MUST be reported by `ucal doctor`; a stale table MUST warn (`UCAL-W0002`) with the bounded error, never convert silently. Offline operation is REQUIRED; no runtime network access.

**8.5 Input calendars for the bridge:** proleptic Gregorian (default) and Julian, both **legacy** per §8.6. Year numbering per §2.5; `BC`/`AD` and `BCE`/`CE` accepted and normalized.

**8.6 Legacy calendars.**

```rust
pub trait LegacyCalendar {
    fn id(&self) -> &str;                       // "earth-civil"
    fn kind(&self) -> Kind;                     // Kind::Legacy, always
    fn tables(&self) -> &DeclaredTables;        // month lengths, week length, leap rule
    fn citation(&self) -> &Citation;
    fn fields(&self, t: &Instant<UC1>) -> Result<LegacyFields, CivilError>;
    fn instant(&self, f: &LegacyFields) -> Result<Instant<UC1>, CivilError>;
}
```

A legacy calendar is a **declared table**, not a derivation. `LegacyCalendar` and `BodyCalendar` are distinct traits with no blanket conversion; a function requiring a derived calendar MUST NOT accept a legacy one (`UCAL-E0065`). Every legacy rendering carries the qualifier (§6.6) and, on request, `UCAL-W0005`.

Shipped legacy calendars: proleptic Gregorian and Julian only (D-20). Each MUST declare its arbitrary content explicitly: irregular month lengths, the 7-day week (no astronomical period), the 97/400 leap rule (not a convergent — Appendix I.1), and the 1582 discontinuity where applicable.

## 9. Calendars

**Rule K (Single derivation mechanism).** Every calendar in this specification is an instance of

```
Calendar = (Body, Anchor, Cycles, LeapRule)
```

1. **Units** derive from the body's `rotation_period`, `solar_day`, and `orbital_period`, each an exact rational of ticks (Rule C). No calendar may declare a unit length.
2. **`LeapRule`** is derived by continued-fraction expansion of `orbital_period / solar_day` (§9.5). No calendar may declare an intercalation rule.
3. **`Cycles`** are optional grouping periods derived from a named satellite's synodic period (§9.6). No calendar may declare a grouping table.
4. **`Anchor`** is supplied per Rule J and is the only empirical, non-derived component of a calendar.
5. Earth is an ordinary instance. There is no privileged body, no body-specific code path, and no crate named after a body.
6. Declared tables are permitted **only** through `LegacyCalendar` (§8.6), which is outside this rule and marked as such in every output.

**Rule J (Anchor).** An anchor MUST be a structure, not a bare number:

```rust
pub struct Anchor {
    pub tick: Instant<UC1>,        // where local counting begins
    pub phase: PhaseDefinition,    // a physical event OF THIS BODY
    pub method: Determination,      // how `tick` was established
    pub window: Window<UC1>,        // uncertainty; MUST contain `tick`
    pub citation: Citation,
    pub revision: u32,
}
pub enum PhaseDefinition {
    MeanSolarMidnight { meridian: Meridian }, NorthwardEquinox,
    SouthwardEquinox, Perihelion, Custom { description: String, citation: Citation },
}
```

1. `phase` MUST name a physical event of the body itself. An anchor MUST NOT be **defined** by reference to another body's calendar, clock, or epoch; its *determination* may cite an observation timestamped in any scale (Rule Y).
2. `window` is REQUIRED and MUST contain `tick`. Anchor uncertainty propagates: every `fields()` result is interval-valued (Rule U) and MUST report the anchor revision that produced it.
3. A calendar without an anchor MUST NOT produce local fields — `UCAL-E0062`, not a guess and not a fallback to another body.
4. An anchor whose `phase` cannot be evaluated for the body's declared parameters is `UCAL-E0063`.
5. Anchors are versioned data, not code. Re-determination bumps `revision`; renderings carry it (§6.6) so values from different revisions are never silently compared.

**9.1 What this buys and what it costs.** Rule K makes the unit ladder and the intercalation of any calendar a pure consequence of the tick, the datum, and the body's periods. Rule J is the honest remainder: phase is empirical (N15). The datum and a tick give elapsed intervals; they cannot say where a planet was pointing — that would require ephemerides (N6). The cost is exactly **one cited, interval-valued constant per body**, with the same status the datum's own physical identification has under Rule Q.3.

**9.2 Body.**

```rust
pub trait Body {
    fn id(&self) -> &str;
    fn rotation_period(&self) -> RatedParam;   // sidereal, ticks
    fn solar_day(&self)      -> RatedParam;    // synodic, ticks
    fn orbital_period(&self) -> RatedParam;    // tropical, ticks
    fn formation(&self)      -> Option<Window<UC1>>;
    fn obliquity(&self)      -> Option<Measured>;   // not a RatedParam

> **[D-A11 · CORRECTION]** Obliquity cannot be a `RatedParam`. A `RatedParam` is a quantity with a linear rate in ticks per tick; obliquity is an angle, and its secular change is not expressible in that form without inventing a unit the type does not carry.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

    fn satellites(&self)     -> &[Satellite];
}
pub struct RatedParam {
    value: Ratio<Ticks>,          // ticks, exact rational — never seconds, never float
    rate:  Option<Ratio<Ticks>>,  // first derivative per tick
    epoch: Instant<UC1>,
    valid: Window<UC1>,
    source: Citation,
    as_measured: MeasuredValue,   // verbatim foreign-unit input + unit (Rule Y.1)
}
pub struct Satellite { id: String, orbital_period: RatedParam, retrograde: bool }
```

**Rule C (Body parameters).** Every parameter MUST carry an epoch, a validity window, a citation, and its verbatim as-measured value, and MUST be stored for computation as an exact rational of ticks. Evaluation outside the validity window MUST warn (`UCAL-W0003`) and MUST NOT silently extrapolate. Earth's rotation lengthens ~1.8 ms/century and its tropical year shortens ~0.53 s/century; both are `rate` terms, not constants.

**9.3 BodyCalendar.**

```rust
pub trait BodyCalendar {
    fn id(&self) -> &str;                       // "earth-d", "mars-d"
    fn kind(&self) -> Kind;                     // Kind::Derived, always
    fn body(&self) -> &dyn Body;
    fn anchor(&self) -> &Anchor;
    fn cycles(&self) -> &[Cycle];
    fn leap_rule(&self) -> &LeapRule;
    fn fields(&self, t: &Instant<UC1>) -> Result<DerivedFields, BodyError>;
    fn instant(&self, f: &DerivedFields) -> Result<Window<UC1>, BodyError>;
}
pub struct DerivedFields {
    pub year: i64, pub day: u32, pub day_fraction: Ratio<Ticks>,
    pub cycle: Option<CyclePosition>,
    pub window: Window<UC1>,          // from anchor and parameter uncertainty
    pub anchor_revision: u32,
}
```

`instant()` returns a `Window`, never an `Instant`: local fields cannot resolve to a single tick while the anchor has width.

**9.4 Determination of an anchor (informative).** Observational, not a computation this specification performs (N6, N15). Workflow: state the phase definition; obtain the instant of that event from an ephemeris or observation with its uncertainty; convert exactly to ticks through the bridge; record `tick`, `window`, `method`, `citation`. The bridge appears in the *determination*, never in the *definition* — which is what keeps Rule J.1 satisfiable.

**9.5 Derived intercalation.**

```rust
fn derive_leap_rule(solar_day: &Ratio<Ticks>, orbital_period: &Ratio<Ticks>,
                    max_drift: &DriftBound) -> Result<LeapRule, BodyError>;

> **[D-A13 · CORRECTION]** A `Delta` is an unsigned count of ticks — a duration. A drift bound is a **rate**: D-12 states the default as "1 day / 10 000 yr". It is therefore `DriftBound { days, per_years }`, expressed in the body's **own** local days and years, so that the same bound means the same thing on Mars without meaning the same duration.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

```

Algorithm: form `r = orbital_period / solar_day` exactly; split into whole day count and fraction; expand the fraction as a continued fraction (Appendix H.5); walk the convergents in order, computing each one's exact worst-case drift; return the first convergent meeting `max_drift`, together with the full sequence walked and the guaranteed bound. If no depth meets `max_drift`, `UCAL-E0061`.

**Correction to earlier revisions (normative).** Revisions 1 and 2 of this RFC claimed the machinery reproduces "the Julian and Gregorian rules as convergents." Only the first is true. For Earth the convergents are 1/4, 7/29, 8/33, 31/128, 752/3105, …: the Julian rule 1/4 is convergent 1, while **97/400 is not a convergent at any depth** — 8/33 is more accurate with a denominator twelve times smaller, and 31/128 is 124× more accurate (Appendix I.1). Implementations MUST NOT special-case 97/400 into the sequence; §21.3 requires a test asserting its absence.

**9.6 Derived grouping cycles.**

```rust
fn derive_cycles(body: &dyn Body, orbital_period: &Ratio<Ticks>,
                 bounds: CycleBounds) -> Vec<Cycle>;
```

Algorithm: for each satellite, compute the synodic period relative to the body's **orbital period** exactly as `1 / |1/P_orb − 1/P_year|`

> **[D-A12 · CORRECTION]** The formula as issued measures against the primary's **solar day**, but Appendix I.2 divides the *year* by the synodic month to reach 12.368266761. For Earth the two differ by a factor of 28: 1.038 d (the interval between successive moonrises) against 29.530589 d (the synodic month). Only the year-relative form reproduces I.2.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).
; admit the satellite only if that period lies within `bounds` (default 5–100 solar days, D-11)

> **[D-A5 · AMENDMENT]** **Grouping cycles are declared per body, not admitted by a global bound.** The 5–100 bracket is calibrated on Earth's Moon, which makes it an Earth-derived constant sitting inside the one mechanism Rule K exists to keep Earth-free (failure mode F9), and whether a satellite is "month-like" is not derivable because *month-like* is an Earth predicate.
>
> A calendar MAY declare at most one `grouping_satellite`, which MUST cite the ground for the choice. The *structure* of the cycle remains derived by continued-fraction expansion and MUST NOT be declared. A calendar declaring none has `cycles() == []` — not a fallback, and not an error at construction. This applies Rule J's existing pattern for phase to grouping admission.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

; for each admitted satellite, expand `orbital_period / synodic_period` as a continued fraction and return the convergents as candidate cycle-to-year commensurability rules.

Applied to Earth this derives the 19-year Metonic cycle (235 synodic months / 19 tropical years) with no special-casing. Applied to Mars it derives **nothing**: Phobos's synodic period is 0.4500 sol and Deimos's 5.3629 sols, so neither qualifies for a 668-sol year. A body with no qualifying satellite has `cycles() == []` and a calendar of years and days only; implementations MUST NOT synthesize one (`UCAL-E0064`). The absence is the correct output — a mechanism that invented a month would be Earth structure leaking.

**9.7** Built-in bodies (HJSON, user-extensible): Sun, Mercury, Venus, Earth, Moon, Mars, Jupiter, Saturn, Titan, Europa, Uranus, Neptune, Pluto. Schema in Appendix G.

**9.8** The derived Earth calendar (`earth-d`) will **not** reproduce the civil Gregorian calendar and MUST NOT be presented as doing so. §21.3 requires a test that quantifies the divergence rather than asserting agreement.

## 10. Cosmology mapping

**10.1** `ucal-cosmo` maps absolute time to scale factor and redshift under a declared flat ΛCDM model, and back, with integer arithmetic only (Rule E).

**10.2 Representation.** Parameters are exact rational **intervals**, each carrying its as-measured value and citation (Rule Y.1):

```rust
pub struct RatInterval { lo: Ratio<Ticks>, hi: Ratio<Ticks> }
pub struct LambdaCdm { h0: RatInterval, omega_m: RatInterval, omega_l: RatInterval,
                       omega_r: RatInterval, as_measured: Vec<MeasuredValue>,
                       citation: Citation }
```

Redshift inputs parse as exact decimals into `Ratio<Ticks>`; `1100`, `1089.80`, and `0.5` are all exact. There is no float path in or out.

**10.3 Method.** `t(z) = ∫ dz / ((1+z) H₀ E(z))`

> **[D-A14 · CORRECTION]** **The integral as written is improper and cannot be quadratured.** Its upper limit is infinite, and no subdivision of `[z, ∞)` into finitely many panels bounds it; truncating at some large `z_max` replaces a proof with a guess about the tail. Substitute `u = 1/(1+z)`:
>
> ```text
> t(z) = (1/H0) ∫_0^{u₀} u du / √(Ω_r + Ω_m u + Ω_Λ u⁴),   u₀ = 1/(1+z)
> ```
>
> `z → ∞` becomes `u → 0`, the range is compact, and `Ω_r > 0` keeps the denominator away from zero, so the endpoint needs no limiting argument. A specification that demands exactness and then writes an improper integral has not finished writing the integral.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

 with `E(z) = √(Ω_r(1+z)⁴ + Ω_m(1+z)³ + Ω_Λ)` is evaluated as a **certified enclosure**: subdivide the range into `2^depth` panels; on each panel bound the integrand above and below using monotonicity plus directed integer square roots; sum the directed Riemann sums separately with outward rounding. No transcendental function is evaluated (N13).

**Rule X (Cosmology outputs are enclosures).** `z → t` and `t → z` MUST return a `Window` (respectively `RatInterval`) provably containing the true value under the declared model, reporting **separately** (a) the arithmetic enclosure width from quadrature and subdivision depth and (b) the width contributed by parameter uncertainty. Merging them into one opaque tolerance is prohibited (F8). Model, parameter set, and citation MUST accompany every result.

**10.4** Inversion (`t → z`) is monotone bisection on exact rationals to a declared width in ticks; the width MUST be ≥ 1 tick and MUST be recorded. Refinement is by subdivision depth, not a floating tolerance.

**10.5** Default parameter set: Planck 2018, cited, with published uncertainties as interval bounds. Alternatives declarable in HJSON.

**10.6 Relationship to the datum.** A cosmological result is a time *relative to the datum*, not relative to the FLRW t→0 limit. For any statement about the first `BIG_BANG_CLAIM` half-width of absolute time — the first ~141 drifts — the implementation MUST surface `UCAL-W0006`, because there the datum's own physical identification is comparable to or larger than the quantity being discussed. `BIG_BANG_CLAIM` remains a non-operand (Rule Q.3): the warning is emitted, the arithmetic is untouched.

## 11. Rule index

| Rule | Subject | § |
|---|---|---|
| Q | Datum stipulated; `BIG_BANG_CLAIM` declared separately and non-consumable; provenance is data | 1.3 |
| A | Atomicity — tick primitive; bridge constants exact integers | 1.4 |
| Y | Metrology — foreign units only at three declared points; declared constants are ticks | 1.5 |
| Z | Zero and unsigned domain | 1.2 |
| M | Monotone total order | 1.2 |
| F | Frame declaration | 1.1 |
| P | Profile binding and tagging | 2.5 |
| W | Value domain identical across backends | 3 |
| O | Overflow is a typed error | 3 |
| E | Integrality — no float anywhere in the workspace | 3 |
| R | Rounding only on rendering, never on construction | 3 |
| G | Tier grid 5^(5k), universal ladder | 4 |
| N | Names are display-only | 4 |
| T | Truncation is uncertainty | 5 |
| U | Interval arithmetic for windows | 5 |
| D | Two text forms, one value | 6 |
| S | Sort order on binary and UCID only | 6 |
| B | Fixed 64-byte canonical binary | 7 |
| I | UCID range and non-uniqueness | 7 |
| L | Leap seconds at the boundary only | 8 |
| K | Single calendar-derivation mechanism; Earth is an instance | 9 |
| J | Anchor: structured, cited, interval-valued, body-defined | 9 |
| C | Body parameter provenance, in ticks | 9 |
| X | Cosmology results are certified enclosures | 10 |

---

# Part B — Library implementation

## 12. Workspace

```
ucal-core     no_std (no-alloc when radix formatting is disabled). Ticks backend,
              Instant/Delta/Signed/SignedWindow/Window/Precision, tier grid, base-5 and
              decimal-group codecs, locale table, profiles (datum + provenance + claim),
              exact rational and interval arithmetic, integer sqrt, continued fractions,
              diagnostics.  Knows no foreign unit except the declared bridge constant.
              deps: bnum | num-bigint (+ num-rational on bigint)

ucal-civil    ::si      SI duration constants in ticks, hifitime bridge, TT pivot,
                        leap seconds, exact civil parsing
              ::legacy  LegacyCalendar impls: proleptic Gregorian, Julian.
                        Declared tables only. No derivation. Quarantined.
              deps: ucal-core, hifitime

ucal-body     Body, Satellite, RatedParam, Anchor, BodyCalendar, derive_leap_rule,
              derive_cycles, body + anchor data. Earth is an entry like any other.
              deps: ucal-core, deser-hjson

ucal-cosmo    flat ΛCDM by certified interval quadrature; integer-only.
              deps: ucal-core

ucal-events   Event catalog: cited, Window-valued cosmological and geological milestones.
              Versioned independently by citation set (D-7).
              deps: ucal-core, deser-hjson

ucal          lib facade + single bin. clap CLI, HJSON config, rulers, cross-body views,
              datum reporting.
              deps: all of the above
```

No crate is named after a celestial body (Rule K.5). All dependencies are pure Rust.

| feature | default | effect |
|---|---|---|
| `u512` | yes | `bnum` 512-bit backend; `Instant` is `Copy` |
| `bigint` | no | `num-bigint` backend; `Instant` is not `Copy`; enables `num-rational` path |
| `civil` | yes | `ucal-civil` (SI bridge + legacy calendars) |
| `body` | yes | `ucal-body` (derived calendars) |
| `events` | yes | `ucal-events` |
| `cosmo` | no | `ucal-cosmo` |
| `serde` | no | serialization per §7.3 |
| `std` | yes | `ucal-core` is `no_std` without it |

Dependency direction is enforced: `ucal-body` MUST NOT depend on `ucal-civil`, and `ucal-civil::legacy` MUST NOT depend on `ucal-body`. The graph itself prevents the derived path from reaching civil tables.

## 13. `ucal-core` — public API contract

```rust
// ---- profile ----
pub trait Profile: 'static {
    const BEAT: Ticks;
    const ORIGIN_OFFSET: Ticks;
    const DOMAIN_MAX: Ticks;
    const FRAME: Frame;
    const TAG: &'static str;                        // "UC1"
    fn bridge() -> &'static Bridge;                 // the only door to foreign units
    fn tiers()  -> &'static TierTable;
    fn big_bang_claim() -> SignedWindow;            // metadata only (Rule Q.3)
    fn datum_provenance() -> &'static Provenance;   // Rule Q.4; absence is E0013
}
pub struct UC1;

pub struct Bridge { pub name: &'static str, pub ticks: Ticks, pub divisibility: u32 }
pub struct MeasuredValue { pub verbatim: &'static str, pub unit: &'static str,
                           pub citation: Citation }
pub struct Provenance { pub input: MeasuredValue,
                        pub unit_defs: &'static [(&'static str, &'static str)],
                        pub chain: &'static [&'static str],
                        pub rounding: RoundingRecord,
                        pub notes: &'static [&'static str] }

// ---- values ----
impl<P: Profile> Instant<P> {
    pub const ZERO: Self;
    pub fn from_ticks(t: Ticks) -> Result<Self, TimeError>;
    pub fn ticks(&self) -> &Ticks;
    pub fn tier_value(&self, t: Tier) -> u16;                       // 0..=3124
    pub fn groups(&self, from: Tier, to: Tier) -> Vec<u16>;
    pub fn floor_to(&self, t: Tier) -> Self;
    pub fn ceil_to(&self, t: Tier) -> Result<Self, TimeError>;
    pub fn round_to(&self, t: Tier, mode: Rounding) -> Result<Self, TimeError>;
    pub fn window_at(&self, p: Precision) -> Window<P>;             // Rule T materialized
    pub fn since(&self, earlier: &Self) -> Result<Delta, TimeError>;
    pub fn between(&self, other: &Self) -> Signed;
    pub fn checked_add(&self, d: &Delta) -> Result<Self, TimeError>;
    pub fn checked_sub(&self, d: &Delta) -> Result<Self, TimeError>;
    pub fn to_ucid(&self) -> Result<Ucid, TimeError>;
    pub fn to_bytes(&self) -> [u8; 64];
    pub fn from_bytes(b: &[u8; 64]) -> Result<Self, TimeError>;
    pub fn rebase<Q: Profile>(&self) -> Result<(Instant<Q>, Signed), TimeError>;
}

// ---- formatting ----
pub struct Fmt { pub form: Form, pub sep: char, pub sub_sep: char,
                 pub precision: Precision, pub pad: bool, pub locale: LocaleId }
pub enum Form { HumanGroups, Digit5, Named }
pub fn parse<P: Profile>(s: &str, ctx: &Fmt) -> Result<(Instant<P>, Precision), ParseError>;
pub fn render<P: Profile>(v: &Instant<P>, f: &Fmt) -> String;       // requires alloc

// ---- numerics (ucal-core::num) ----
pub fn mul_div(a: &Ticks, n: &Ticks, d: &Ticks) -> Result<(Ticks, Ticks), NumError>;
pub fn isqrt_floor(x: &Ticks) -> Ticks;
pub fn isqrt_ceil(x: &Ticks) -> Ticks;
pub fn cf_expand(r: &Ratio<Ticks>, max_depth: u32) -> Vec<u64>;
pub fn convergents(cf: &[u64]) -> Vec<Ratio<Ticks>>;
pub enum Kind { Derived, Legacy }
```

**13.1** Base-5 codec per Appendix F: repeated `divmod` by `5^5 = 3125`, one group per step — 44 steps at full width, not 221. Digit-by-digit division by 5 MUST NOT be used.

**13.2** `Bridge` is the only type exposing a foreign unit. `ucal-core` MUST NOT contain the identifiers `second`, `day`, or `year` outside that declaration and outside `MeasuredValue`/`Provenance` string data; a CI lint enforces this alongside the float lint (Rules A.2, Y).

**13.3** `SignedWindow` has no arithmetic impls and no conversion to `Delta`, `Instant`, or `Window`. §21.3 requires a compile-fail test proving that lifting this restriction breaks the build (Rule Q.3).

**13.4** `Kind` lives in core so every rendering path can be forced to state derived-vs-legacy (§6.6).

**13.5** The tier table, the locale table, and the docs table in §4.1 MUST be generated from one source of truth so they cannot drift.

## 14. `ucal-civil`

```rust
// ::si
pub enum Scale { Tt, Tai, Utc }
pub fn from_si_seconds(s: &Ratio<Ticks>) -> Result<Instant<UC1>, CivilError>;   // exact
pub fn to_si_seconds(t: &Instant<UC1>) -> Ratio<Ticks>;                          // exact
pub fn from_epoch(e: hifitime::Epoch) -> Result<Instant<UC1>, CivilError>;
pub fn to_epoch(t: &Instant<UC1>, r: Rounding) -> Result<hifitime::Epoch, CivilError>;
pub fn from_civil(y: i64, m: u8, d: u8, h: u8, min: u8, s: u8, sub: SubSecond,
                  scale: Scale, cal: CivilCalendar) -> Result<Instant<UC1>, CivilError>;
pub fn to_civil(t: &Instant<UC1>, scale: Scale, digits: u8, r: Rounding)
                  -> Result<CivilFields, CivilError>;
pub fn leap_table_version() -> &'static str;

// ::legacy
pub struct Gregorian;  pub struct Julian;      // impl LegacyCalendar, kind() == Legacy
pub fn parse_date(s: &str) -> Result<(Instant<UC1>, Precision), CivilError>;   // "44 BC-03-15"
```

**14.1** `SubSecond` is an exact decimal fraction of at most 30 digits (`UCAL-E0043` beyond). Construction is exact; rendering rounds only at the requested digit count and reports loss.
**14.2** `to_civil` on a UTC leap-second instant MUST return `sec = 60` rather than normalizing.
**14.3** The renderable civil range is a public constant; exceeding it is `UCAL-E0040`, never a panic.

## 15. `ucal-body`

**15.1** Loader `deser-hjson`, strict (unknown keys → `UCAL-E0012`). Body files and anchor files are separate and version independently: parameters change with better measurement, anchors with re-determination.
**15.2** `derive_leap_rule` and `derive_cycles` MUST be deterministic and MUST return the full convergent sequences they walked, so any derived calendar is auditable end to end.
**15.3** `DerivedFields` MUST NOT contain a month or weekday unless a cycle was derived (§9.6). No fallback structure is permitted.
**15.4** Earth's entry has no special code path, no extra fields, and no compile-time distinction from Mars's.
**15.5** `fields()` algorithm: `elapsed = t.since(anchor.tick)?`; divide by `solar_day` (with `rate` applied over the interval) to get whole local days and a fraction; apply `leap_rule` to convert whole days to (year, day-of-year); apply `cycles` if present; propagate `anchor.window` and parameter uncertainty into `DerivedFields::window` by interval arithmetic (Rule U).

## 16. `ucal-cosmo`

```rust
pub fn t_of_z(&self, z: &Ratio<Ticks>, depth: u32)
    -> Result<CosmoResult<Window<UC1>>, CosmoError>;
pub fn z_of_t(&self, t: &Window<UC1>, width: &Delta)
    -> Result<CosmoResult<RatInterval>, CosmoError>;
pub fn a_of_t(&self, t: &Instant<UC1>, depth: u32)
    -> Result<CosmoResult<RatInterval>, CosmoError>;

pub struct CosmoResult<T> {
    pub value: T,
    pub arithmetic_width: Delta,     // from quadrature and subdivision
    pub parameter_width: Delta,      // from model uncertainty
    pub depth: u32, pub scale: u32,  // fixed-point scale used (D-6)
    pub model: ModelId, pub citation: Citation,
}
```

Per Rule X the two widths are reported separately and never merged. Per §10.6, results inside the claim half-width carry `UCAL-W0006`.

## 17. `ucal-events`

Cited, `Window`-valued milestones — inflation, recombination, reionization, first stars, galaxy formation, Solar System formation, LUCA, Cambrian, K-Pg, hominin divergence, present. Each entry: id, label (localized), `Window<UC1>`, source citation, and a note where the window falls inside `BIG_BANG_CLAIM` (§10.6). Versioned independently of `ucal` so citation revisions do not force a library release (D-7).

## 18. Configuration

Layered HJSON: built-in defaults → `$XDG_CONFIG_HOME/ucal/config.hjson` → environment → flags. No secrets, no network.

```hjson
{
  profile: UC-1
  format:  { form: human, sep: "·", sub_sep: ":", locale: en, pad: false }
  rounding: half-even                 // rendering only (Rule R)
  claim_warnings: true                // UCAL-W0006 per §10.6
  calendars: { default: null, cycle_bounds: [5, 100], max_drift: "1 day / 10000 yr" }
  bodies_dir: null
  anchors_dir: null
  cosmology: { model: planck2018, depth: 24 }
}
```

---

# Part C — Command line

## 19. CLI

```
ucal now [--precision beat|arc|…] [--form human|digit5|named]
ucal datum                                   # datum statement, BIG_BANG_CLAIM, provenance
ucal from-si <SECONDS>
ucal from-civil <DATE> [--scale tt|tai|utc] [--calendar gregorian|julian]
ucal to-civil <T> [--scale …] [--digits N] [--round half-even|trunc|ceil|half-up]
ucal convert <T|DELTA> --to <tier|ns|s|min|h|d|week|year-julian> [--round …]
ucal diff <A> <B> [--in <tier>]
ucal explain <T> [--claim]
ucal id <T>  |  ucal parse <UCID|T>
ucal ladder [--locale ru]
ucal timeline [--tier drift] [--from <T>] [--to <T>]
ucal ruler --from <T> --to <T> --step <tier>
ucal cal list | cal show <id> <T> | cal derive <body> [--max-drift D] [--cycle-bounds LO,HI]
             | cal anchor <body>
ucal show <T> --calendars earth-d,mars-d,titan-d,earth-civil
ucal body list | body show <id>
ucal events list | events show <id>
ucal z <REDSHIFT> [--depth N]                # feature cosmo
ucal doctor
```

**19.1** Every command supports `--profile`, `--sep`, `--locale`, `--json`. `--json` output is stable and versioned.
**19.2** `ucal datum` MUST print, in this order: the datum statement ("tick 0 is a stipulated reference point, conventionally identified with the FLRW t→0 limit"), `BIG_BANG_CLAIM` with citation, the full provenance chain from §2.2, and the rounding residual. It MUST NOT present the implied age as a measurement of the universe.
**19.3** `ucal doctor` reports profile, backend and domain ceiling, leap-second table version, feature set, lint status, and the presence of a `datum_provenance` record.
**19.4** `ucal cal list` MUST display `kind` for every entry. `ucal show --calendars` is the primary demonstration of Rules K and J: one absolute instant, several local renderings, each with its anchor revision and uncertainty window, with legacy Gregorian shown alongside and labelled.

**19.5 Exit codes.**

| exit | meaning |
|---|---|
| 0 | success |
| 1 | usage error |
| 2 | parse error |
| 3 | domain error (Rules Z, O, W) |
| 4 | precision error (Rules T, R) |
| 5 | profile mismatch (Rule P) |
| 6 | data/config error, including missing provenance (Rule Q.4) |
| 7 | calendar derivation or anchor error (Rules K, J, C) |
| 8 | cosmology model or enclosure error (Rule X) |
| 9 | internal invariant violation, including metadata used as an operand (Rule Q.3) |

---

# Part D — Delivery

## 20. Phases

| phase | content | exit criterion |
|---|---|---|
| UC-P0 | Constants harness: two independent exact-integer derivations of both primitives, the claim half-width, and every fixture; `datum_provenance` record; signed vector file | Appendices A, C, I reproduced by both routines; §2.4 invariants hold; provenance chain re-executes to the declared `ORIGIN_OFFSET` and residual |
| UC-P1 | `ucal-core` types, backends, domain checks, tier grid; float lint and foreign-identifier lint live from day one | Property tests identical on both backends; lints fail a deliberate violation |
| UC-P2 | `Profile` metadata surface: `big_bang_claim`, `datum_provenance`, `SignedWindow` isolation | Compile-fail test proves `SignedWindow` cannot reach arithmetic; `UCAL-E0013` on a profile without provenance |
| UC-P3 | `ucal-core::num`: `mul_div`, directed `isqrt`, exact rationals, intervals, continued fractions | Directed-rounding post-conditions verified; `cf_expand`/`convergents` reproduce Appendix I |
| UC-P4 | Base-5 and decimal-group codecs, both text forms | Round-trip property tests over the full domain; 44-step encode verified |
| UC-P5 | Canonical binary, UCID, sort-order proofs | Bytewise order == numeric order, fuzzed; `UCAL-E0031` above 2²⁵⁶ |
| UC-P6 | Precision, `Window`, interval comparison | Rules T and U enforced; no path yields tick precision from truncated input |
| UC-P7 | `ucal-civil::si`: TT pivot, exact conversion, leap seconds, `SubSecond` | 10⁶ random civil instants convert with zero rounding; `UCAL-E0043` on finer input; differential vs `hifitime` |
| UC-P8 | `ucal-civil::legacy`; `Kind` plumbed through every rendering path | No rendering path omits the qualifier; `UCAL-E0065` fires correctly |
| UC-P9 | CLI core: `now`, `datum`, `from-civil`, `to-civil`, `explain`, `doctor`, `--json` | Golden-output tests; `ucal datum` matches §19.2 ordering and makes no measurement claim |
| UC-P10 | Locale tables, `ru` locale, `ladder`; documentation lint | Docs and `--help` generated from the table; drift test; Rule Q.1 lint green |
| UC-P11 | `ucal-body`: `Body`, `Satellite`, `RatedParam` with `as_measured`, Rule C enforcement, body data | Parameters stored in ticks with verbatim inputs retained; `UCAL-W0003` fires; no dependency on `ucal-civil` |
| UC-P12 | `Anchor`, Rule J, anchor data files and versioning | `UCAL-E0062` without an anchor; `fields()` interval-valued; anchor revision in every rendering |
| UC-P13 | `derive_leap_rule` + `derive_cycles` | Reproduces Appendix I exactly: Earth 1/4 as convergent 1 and 97/400 absent at every depth; Metonic 235/19 derived; Mars yields no cycle |
| UC-P14 | Derived calendars for Earth, Mars, Titan end to end; `ucal cal *`, `ucal show --calendars` | `earth-d` built by the identical code path as `mars-d`; divergence from `earth-civil` quantified and recorded |
| UC-P15 | `ucal-events`, `timeline`, `ruler` | One-screen demo; every entry cited and `Window`-valued; `UCAL-W0006` where applicable |
| UC-P16 | `ucal-cosmo` (Rules X, §10.6); `no_std` + wasm build; release documentation | `z = 1100` enclosure lies inside the catalog's recombination window; enclosure narrows monotonically with depth; float oracle contained; `no_std` green |

## 21. Test plan

**21.1 Property tests** — round-trips through both text forms, canonical binary, and UCID; truncation monotonicity; `floor_to ≤ id ≤ ceil_to`; lexicographic == numeric for binary and UCID; `since`/`between` consistency; tier decomposition reassembles the original value; for every derived calendar, `instant(fields(t))` contains `t`.

**21.2 Differential tests** — default backend vs `bigint` backend byte-identical for every operation on every fixture and on random inputs (this is what Rule W buys); `ucal-civil::si` against `hifitime` directly; `ucal-cosmo` against a float ΛCDM oracle in `dev-dependencies`, asserting only that the certified enclosure contains the oracle's value.

**21.3 Invariant lints and required assertions.** CI MUST fail on: any float token in a shipped crate; the identifiers `second`, `day`, `year` in `ucal-core` outside `Bridge`/`MeasuredValue`/`Provenance` data; a `ucal-body` → `ucal-civil` dependency; `unwrap`/`panic!` reachable from public API; any hand-transcribed constant the P0 harness does not reproduce. Required assertions:

1. §2.4 alignment invariants — 30 trailing base-5 zeros for whole SI seconds, 21 for whole nanoseconds, `SI_EPOCH` zero below T0.
2. The `datum_provenance` chain re-executes to the declared `ORIGIN_OFFSET` with the stated residual.
3. A compile-fail test proving `SignedWindow` cannot be used as an operand (Rule Q.3).
4. No public API accepts `BIG_BANG_CLAIM` or its half-width as an argument.
5. Documentation lint: "creation of the universe", "age of the universe is", and equivalents do not appear as descriptions of tick 0 (Rule Q.1).
6. Earth's intercalation convergents equal Appendix I.1; **1/4 is convergent 1; 97/400 does not appear at any depth**.
7. Earth's grouping-cycle sequence contains 235/19; Mars yields no cycle under default bounds.
8. `earth-d` and `earth-civil` diverge, with the divergence measured and recorded — not asserted away.
9. No code path constructs a `BodyCalendar` without an `Anchor`.
10. `earth-d` and `mars-d` are produced by the identical generic code path, verified by a test that constructs both from data alone.
11. Cross-profile arithmetic fails to compile (Rule P).
12. No wrapping or saturating arithmetic is reachable on time types (Rule O).

**21.4 Fixtures** — a signed vector file containing at minimum: absolute zero, `SI_EPOCH`, `44 BC-03-15`, Apollo 11, Unix epoch, GPS epoch, J2000.0, every leap-second instant, `2026-07-29`, Earth formation, recombination, first stars, Cambrian, K-Pg. Each entry: civil date and scale, decimal ticks, both text forms, UCID, 64-byte hex. Appendices C and I seed it.

**21.5 Test target** ≈ 1,300 tests from a greenfield baseline of 0.

## 22. Conformance classes

| class | requires |
|---|---|
| **C-Core** | Rules Q, A, Y, Z, M, P, W, O, E, R, G, N, T, U, D, S, B, I; Appendix C reproduced exactly; §2.4 invariants; provenance present and re-executing; `SignedWindow` isolation |
| **C-Bridge** | C-Core + Rule L + §8.1–8.5, including bridge exactness and `UCAL-E0043` |
| **C-Calendar** | C-Core + Rules K, J, C + §9; Appendix I reproduced exactly; Earth built by the generic path; no anchor-less calendar constructible |
| **C-Legacy** | C-Core + §8.6; `Kind` present in every rendering; `UCAL-E0065` enforced |
| **C-Cosmo** | C-Core + Rule X + §10, with enclosure widths reported separately and `UCAL-W0006` per §10.6 |
| **C-Full** | all of the above + CLI §19 + `--json` stability |

An implementation MAY claim C-Calendar without C-Legacy. It MUST NOT claim C-Calendar if any calendar in it uses a declared table. It MUST NOT claim C-Core if it describes tick 0 as measured, or if any arithmetic path can consume `BIG_BANG_CLAIM`.

## 23. Decisions

Every open question from prior revisions is closed here. These are normative.

| # | Decision | Rationale |
|---|---|---|
| D-1 | `ORIGIN_OFFSET` keeps the value derived from 13.787 Gyr, not a rounder beat count | Auditable provenance beats memorability; a rounder constant would cost ~1 day of implied age for no functional gain |
| D-2 | Base tier is 5⁶⁰ (46.762 ms); the grid is uniform `5^(5k)` from 5⁰ | Uniform digit grouping with no ragged fields at either end; the loss of a second-scale and hour-scale unit is the accepted cost of leaving the Earth paradigm |
| D-3 | `SECOND` is the nearest multiple of **10³⁰** to the measured reciprocal Planck time | Makes every decimal SI subdivision to 10⁻³⁰ s exact; the 2.6×10⁻¹⁴ relative deviation is nine orders below the measurement's own uncertainty |
| D-4 | Backends: `U512` default, `bigint` feature; U256 dropped | 512 bits covers the whole standard far-future timeline, so backend width never needs widening — and width is a wire-format commitment (Rule B) |
| D-5 | No float anywhere, `ucal-cosmo` included | Certified enclosures are strictly better than float tolerances, and the integer path needs only `isqrt` (N13) |
| D-6 | Fixed-point scale for `ucal-cosmo` is per call, recorded in `CosmoResult` | Different queries need different precision; a global scale would over- or under-serve |
| D-7 | Event catalog is a separate crate `ucal-events`, versioned by citation set | Citations get revised more often than the library |
| D-8 | Canonical binary is fixed 64 bytes; UCID is 52 chars over 256 bits | Sortable without rules; a base-32 encoding of the full 512-bit range would be unusable as an identifier |
| D-9 | Canonical text form for parse and sort is the base-5 digit form (`UC1/5`); the decimal group form (`UC1`) is for humans | One value, two tagged forms (Rule D) |
| D-10 | Separator default `·`, `.` always accepted on input | Typography without shell hostility |
| D-11 | Default `cycle_bounds` 5–100 solar days | Brackets "month-like" without admitting Phobos-scale or multi-year satellites |
| D-12 | Default `max_drift` is 1 day / 10 000 yr, overridable per body | A defensible fixed default; per-body derivation from parameter uncertainty is GE-3 work |
| D-13 | `Precision` is a runtime field, not a type parameter | Type-level precision would infect every signature for little safety gain over Rule T |
| D-14 | `rebase` between profiles is permitted and returns the shift | Reporting-only would force users to reimplement it less safely |
| D-15 | Anchor `PhaseDefinition` is an open enum; `Custom` requires a citation | Bodies vary more than a closed set can anticipate |
| D-16 | `BIG_BANG_CLAIM` is a symmetric half-width; asymmetric windows are permitted by the type | Matches how the Planck 2018 uncertainty is published without foreclosing asymmetric ones |
| D-17 | Crate is renamed `ucal-earth` → `ucal-civil`, split `::si` / `::legacy` | No crate may be named after a body (Rule K.5); the split is what quarantines declared tables |
| D-18 | Shipped legacy calendars are proleptic Gregorian and Julian only | Anything more invites the legacy layer to grow into a calendar library (N5) |
| D-19 | Documentation is English with the `ru` locale table shipped in-tree | Locale data serves users without doubling the documentation burden |
| D-20 | Tiers above T5 and below T−3 remain unnamed, addressable by index | Naming them is a locale change, not a specification change (Rule N) |
| D-21 | No `UC-2` profile in 0.1; the CMB-anchored provenance route is documented future work | It improves auditability, not exactness — every route ends in a stipulation (Rule Q.2). See GE-6 |

## 24. Gated experiments

Remaining uncertainty is implementation-level. Each experiment has a kill criterion; failing it changes an implementation choice, not the specification.

| # | Experiment | Kill criterion |
|---|---|---|
| GE-1 | Certified quadrature performance: can `t_of_z` at depth 24 complete in interactive time with `U512`/`U1024` integer arithmetic? | If depth 24 exceeds ~2 s on commodity hardware, reduce the default depth and expose an explicit "high precision" mode rather than degrading the enclosure guarantee |
| GE-2 | Fixed-point scale selection: which scale yields ≤1-tick enclosure width for `z = 1100` at a practical depth? | If no scale below `U1024` intermediate width achieves it, publish the achievable width and set `UCAL-W0004` accordingly rather than promising tick-level cosmology |
| GE-3 | Anchor determination: can Earth, Mars, and Titan anchors be established to a window narrower than one local solar day from published ephemerides? | If not, `DerivedFields` windows exceed one day and the derived calendars are honest but coarse; document the width rather than narrowing it by assumption |
| GE-4 | Backend library choice: `bnum` (pre-1.0, recently restructured its type surface) vs `ruint` vs `crypto-bigint` on `const` construction, `divmod` throughput, and API stability | If `bnum`'s churn breaks the build more than once per minor release, switch to `ruint`; `Ticks` is a type alias behind `TickInt`, so the cost is bounded |
| GE-5 | `no_std` + no-alloc viability with radix formatting disabled, targeting wasm | If the core cannot build without `alloc`, drop the no-alloc claim and keep `no_std` + `alloc` |
| GE-6 | CMB-anchored provenance (`UC-2`): does deriving the datum offset from `z = 1089.9 ± 0.4` through `ucal-cosmo` produce a shorter, fully tick-native chain? | If the resulting enclosure is wider than the current published age uncertainty, the route adds auditability without adding rigour; leave D-21 standing |

---

# Appendix A — Constants (profile UC-1)

```
DATUM (declared, in ticks)
  BEAT              = 5^60 = 867361737988403547205962240695953369140625
  ORIGIN_OFFSET     = 9 304 311 741 502 590 385 × BEAT
                    = 8070204002895596515944343085635637180530466139316558837890625
                      (203 bits; 88 base-5 digits; 61 trailing base-5 zeros)

> **[D-A2 · EDITORIAL]** `ORIGIN_OFFSET` has **61** trailing base-5 zeros, not 62. Verified by exact integer computation along two independent routes.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

  DOMAIN            = [0, 2^512)
  DOMAIN_MAX        = 2^512 − 1 ≈ 1.3408×10^154 ticks

DATUM CLAIM (declared metadata; never an operand — Rule Q.3)
  BIG_BANG_CLAIM    = datum ± 631 152 × 18 548 584 399 861 × 10^39 ticks
                    = ± 11706976141141069872000000000000000000000000000000000000000
                    ≈ ± 1.170 698×10^58 ticks = ± 0.020 Gyr (Planck 2018)
                    = 0.1451 % of ORIGIN_OFFSET = ± 141.53 drift

BRIDGE (declared, exact integer of ticks)
  SECOND            = 18 548 584 399 861 × 10^30
                    = 18548584399861000000000000000000000000000000
  SI_EPOCH          ≡ tick ORIGIN_OFFSET
                    = 0000-01-01T00:00:00.000 TT, proleptic Gregorian,
                      astronomical year numbering (= 1 BC)

DATUM PROVENANCE (declared data — Rule Q.4; full record §2.2)
  input             13.787 ± 0.020 Gyr, age_of_universe, Planck 2018 VI (A&A 641 A6)
  unit_defs         Gyr = 10^9 × 31 557 600 s (Julian years, exact by definition)
  chain             AGE_s     = 435 084 631 200 000 000
                    AGE_ticks = 8070204002895596516263200000000000000000000000000000000000000
                    beats     = round(AGE_ticks / BEAT) = 9 304 311 741 502 590 385
  rounding          to BEAT, half_even,
                    residual = −318856914364362819469533860683441162109375 ticks
                             = −0.017 190 364 s

INFORMATIVE (derived; never an input)
  tick length       = 1 / SECOND = 5.391 247 000 000 139 6 × 10^-44 s
                      (deviates 2.583×10^-14 relative from the measured 5.391247×10^-44 s,
                       which itself carries ~10^-5 uncertainty)
  implied age       = ORIGIN_OFFSET / SECOND = 435 084 631 200 000 000.0 s
                    = 13.787 000 000 000 Gyr of Julian years
                      — a consequence of the declared datum, NOT a measurement (Rule Q.1)
  1 s               = 21.385 061 835 beats
  UCID ceiling      = 2^256 − 1 ≈ 1.978 172×10^26 yr
  domain ceiling    ≈ 2.290 567×10^103 yr
  present epoch     = 31.220 deeps
```

# Appendix B — Tier table

> **[D-A3 · EDITORIAL]** The published seconds column is imprecise in the fifth significant figure at several tiers, because it was chained from neighbouring rows rather than computed independently. Each value MUST be rendered from the exact rational `5^e / SECOND` in one step under half-even rounding. [`docs/TIERS.md`](../docs/TIERS.md) is generated that way and is authoritative.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).



| k | exponent | seconds (informative) | human | name |
|---|---|---|---|---|
| 5 | 85 | 1.3934×10¹⁶ | 441.607 Myr | deep |
| 4 | 80 | 4.4590×10¹² | 141.314 kyr | drift |
| 3 | 75 | 1.4269×10⁹ | 45.221 yr | span |
| 2 | 70 | 4.5660×10⁵ | 5.285 d | sweep |
| 1 | 65 | 1.4613×10² | 146.130 s | arc |
| 0 | 60 | 4.6762×10⁻² | 46.762 ms | beat |
| −1 | 55 | 1.4964×10⁻⁵ | 14.964 µs | flicker |
| −2 | 50 | 4.7884×10⁻⁹ | 4.788 ns | glint |
| −3 | 45 | 1.5323×10⁻¹² | 1.532 ps | spark |
| −4 | 40 | 4.9034×10⁻¹⁶ | — | — |
| −5 | 35 | 1.5691×10⁻¹⁹ | — | — |
| −6 | 30 | 5.0212×10⁻²³ | — | — |
| −7 | 25 | 1.6068×10⁻²⁶ | — | — |
| −8 | 20 | 5.1417×10⁻³⁰ | — | — |
| −9 | 15 | 1.6454×10⁻³³ | — | — |
| −10 | 10 | 5.2652×10⁻³⁷ | — | — |
| −11 | 5 | 1.6848×10⁻⁴⁰ | — | — |
| −12 | 0 | 5.3912×10⁻⁴⁴ | tick | tick |

Tiers 6…32 (5⁹⁰…5²²⁰) exist, are unnamed, and span 1.38×10¹² to 2.29×10¹⁰³ years.

# Appendix C — Fixtures

> **[D-A4 · CORRECTION]** The human forms printed here are **truncated at T−5**, not exact to the tick. Read as tick-precision values they disagree with the tick counts in the same table; read as Rule T statements at T−5 they are correct, and all eight tick fixtures reproduce bit-exactly.
>
> A claim that the 44 BC fixture was one day late was raised during verification and **withdrawn** (D-A1): the fault was in the checking oracle, not the RFC. Appendix C is 8/8 correct.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).



Computed with exact integer arithmetic under `UC-1`. **Civil times are TT labels** — the quoted civil time is taken as TT, which is the conversion an implementation MUST reproduce with `--scale tt`. UTC-scale fixtures require the leap-second table and are a UC-P7 deliverable.

```
absolute zero (the datum)
  ticks  0
  human  UC1 0000·0000·0000·0000·0000·0000
  ucid   0000000000000000000000000000000000000000000000000000

SI_EPOCH  0000-01-01T00:00:00 TT
  ticks  8070204002895596515944343085635637180530466139316558837890625
  human  UC1 0031·0687·2437·0454·2703·2885
  sub    all tiers below T0 are zero  (§2.4)
  ucid   0000000000050PM5TBHF4BFKRZC1KVN566SZGWG5DZ0SSBM29FJ1

-0043-03-15T00:00:00 TT   (44 BC, astronomical year -0043)
  ticks  8070203977843789392286957152835637180530466139316558837890625
  human  UC1 0031·0687·2436·0622·0843·1347:2726·0773·2384·0202·2453
  ucid   0000000000050PM5STSSZT2034C3TGX8CMS2Z79C0SGBSBM29FJ1

1969-07-20T20:17:40 TT   (Apollo 11 landing, TT label)
  ticks  8070205155746435292175415045495637180530466139316558837890625
  human  UC1 0031·0687·2480·2184·1466·1514:0493·1291·0839·2005·2854
  ucid   0000000000050PM6JDVVP2F5SVFPGQVWXDMG2X4TRBKE3ZM29FJ1

1970-01-01T00:00:00 TT   (Unix epoch label)
  ticks  8070205156009508751803579616835637180530466139316558837890625
  human  UC1 0031·0687·2480·2215·1648·1438:0170·1214·0735·1806·1815
  ucid   0000000000050PM6JE1FP4P4GK3BBVJDEP65A8Y6G77WSBM29FJ1

1980-01-06T00:00:00 TT   (GPS epoch label)
  ticks  8070205161870208511988780509635637180530466139316558837890625
  human  UC1 0031·0687·2480·2907·1365·0098:1925·2279·0814·1116·0715
  ucid   0000000000050PM6JHYSQVFAA0446J292X7AGVGHSP9PNBM29FJ1

2000-01-01T12:00:00 TT   (J2000.0, exact)
  ticks  8070205173569972963515184424835637180530466139316558837890625
  human  UC1 0031·0687·2481·1163·2191·0758:1924·0749·2247·0012·1174
  ucid   0000000000050PM6JSRZ1JEN8CJ8JG0H3SXHYWVS2CY7KBM29FJ1

2026-07-29T00:00:00 TT
  ticks  8070205189123984864657505252035637180530466139316558837890625
  human  UC1 0031·0687·2481·2999·3108·2437:1104·2790·0251·2597·0804
  digit5 UC1/5 00000.00000.00000.00000.00111.10222.34411.43444.44413.34222.
               13404.42130.02001.40342.11204.13400.00000.00000.00000.00000.
               00000.00000
  ucid   0000000000050PM6K45HH4YGQJ6SEDGDDZ1NKFHD32F2XBM29FJ1

Earth formation, SI_EPOCH − 4.54 Gyr (Julian years)
  ticks  5412720418856573655000343085635637180530466139316558837890625
  human  UC1 0020·2935·2420·2803·2533·2001:2269·2517·0923·1945·1875
  ucid   000000000003BS5WVY8XGGMN3M0D068RR37G6W0DEWKHSBM29FJ1

recombination, t = 380 kyr after the datum
  (point value; the catalog entry MUST be a Window, and any statement at this scale falls
   inside BIG_BANG_CLAIM ⇒ UCAL-W0006 per §10.6)
  ticks  222432546681680327568000000000000000000000000000000000000
  human  UC1 0000·0002·2153·0825·0246·0025:1908·2584·2019·0482·2740
  ucid   000000000000004H4KEWEGEB5M995XKBZHX3425VFFD900000000
```

Two structural checks visible above: every whole-second fixture ends in six all-zero groups in digit form (§2.4), and all modern dates share the prefix `0031·0687·248` — the `span` group (45 yr) is the field that turns over within a human lifetime.

# Appendix D — Locale table

```hjson
{
  profile: UC-1
  tiers: [
    { k:  5, exp: 85, id: deep }    { k:  4, exp: 80, id: drift }
    { k:  3, exp: 75, id: span }    { k:  2, exp: 70, id: sweep }
    { k:  1, exp: 65, id: arc  }    { k:  0, exp: 60, id: beat  }
    { k: -1, exp: 55, id: flicker } { k: -2, exp: 50, id: glint }
    { k: -3, exp: 45, id: spark }   { k:-12, exp:  0, id: tick  }
  ]
  locales: {
    en: { deep: [deep, deeps], drift: [drift, drifts], span: [span, spans]
          sweep: [sweep, sweeps], arc: [arc, arcs], beat: [beat, beats]
          flicker: [flicker, flickers], glint: [glint, glints]
          spark: [spark, sparks], tick: [tick, ticks] }
    ru: { deep: [глубь, глуби], drift: [дрейф, дрейфы], span: [срок, сроки]
          sweep: [обход, обходы], arc: [дуга, дуги], beat: [бой, бои]
          flicker: [мерцание, мерцания], glint: [блик, блики]
          spark: [искра, искры], tick: [тик, тики] }
  }
}
```

Canonical identity is `exp`; `id` is a stable key; locale strings are display and parse aliases (Rule N). Names were chosen as short, concrete motion words with no mythological, religious, national, or numeric-prefix content. Calendar unit names (day, year, cycle) are **not** in this table — they belong to each body's calendar and are declared with it.

# Appendix E — Diagnostic codes

| code | meaning |
|---|---|
| UCAL-E0001 | malformed timestamp |
| UCAL-E0002 | unknown profile tag |
| UCAL-E0003 | mixed text forms in one string (Rule D) |
| UCAL-E0004 | group value out of range (> 3124) |
| UCAL-E0005 | invalid base-5 digit |
| UCAL-E0006 | non-contiguous tier sequence |
| UCAL-E0007 | calendar rendering without a kind/id qualifier (§6.6) |
| UCAL-E0010 | locale table load failure |
| UCAL-E0011 | duplicate name in active locale table (Rule N) |
| UCAL-E0012 | unknown key in HJSON data file |
| UCAL-E0013 | profile lacks a `datum_provenance` record (Rule Q.4) |
| UCAL-E0014 | name not found in the active locale table (Rule N) — D-A17 |
| UCAL-E0015 | this build does not reproduce the declared constants (§3.3) — D-A18 |
| UCAL-E0020 | result precedes the datum (Rule Z) |
| UCAL-E0021 | result exceeds DOMAIN (Rules O, W) |
| UCAL-E0022 | window inversion, lo > hi (Rule U) |
| UCAL-E0023 | comparison indeterminate at stated precision (Rule T) |
| UCAL-E0024 | lossy rendering requested without a rounding mode (Rule R) |
| UCAL-E0025 | `BIG_BANG_CLAIM` or its half-width used as a computational operand (Rule Q.3) |
| UCAL-E0030 | binary form is not 64 bytes (Rule B) |
| UCAL-E0031 | instant outside UCID range (Rule I) |
| UCAL-E0032 | invalid Crockford base-32 |
| UCAL-E0040 | civil date outside renderable range |
| UCAL-E0041 | invalid civil date for the stated calendar |
| UCAL-E0042 | second = 60 outside a leap-second instant |
| UCAL-E0043 | foreign-unit input finer than the bridge constant permits (Rules A, R, Y) |
| UCAL-E0050 | profile mismatch (Rule P) |
| UCAL-E0060 | body parameter missing required provenance or as-measured value (Rules C, Y) |
| UCAL-E0061 | leap-rule derivation cannot meet the requested drift bound |
| UCAL-E0062 | calendar has no anchor; local fields cannot be produced (Rule J.3) |
| UCAL-E0063 | anchor phase definition not evaluable for this body's parameters (Rule J.4) |
| UCAL-E0064 | grouping cycle requested but none derivable from any satellite (§9.6) |
| UCAL-E0065 | legacy calendar supplied where a derived calendar is required (Rule K.6) |
| UCAL-E0070 | cosmology inversion failed to bracket |
| UCAL-E0071 | requested enclosure width unreachable at the permitted depth (Rule X) |
| UCAL-W0001 | precision loss in requested rendering (Rule R) |
| UCAL-W0002 | leap-second table may be stale; bounded error reported |
| UCAL-W0003 | body parameter evaluated outside its validity window (Rule C) |
| UCAL-W0004 | cosmology enclosure width exceeds one tick |
| UCAL-W0005 | value produced by a legacy (non-derived) calendar (§8.6) |
| UCAL-W0006 | quantity comparable to or smaller than `BIG_BANG_CLAIM`; the datum's physical identification is uncertain at this scale (§10.6) |

# Appendix F — Base-5 group codec

```
GROUP_BASE = 3125            // 5^5, one tier per step

encode(t, k_lo, k_hi):
    x = t / TIER[k_lo]                              // exact integer division
    out = []
    while x > 0 or len(out) < (k_hi - k_lo + 1):
        (q, r) = divmod(x, GROUP_BASE); out.push(r); x = q      // r in 0..=3124
    reverse(out)                                    // most significant tier first

decode(groups, k_lo):
    acc = 0
    for g in groups:
        assert g < GROUP_BASE                       // else UCAL-E0004
        acc = acc * GROUP_BASE + g
    acc * TIER[k_lo]

digit5(group) -> [char; 5]                          // five base-5 digits, zero-padded
```

Full-width encode is 45 `divmod` steps.

> **[D-A7 · CORRECTION]** A full-width value spans **45** five-digit groups, not 44: 220 base-5 digits is 44 groups, and the value may carry a 221st digit.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).
 Digit-by-digit division by 5 MUST NOT be used.


> **[D-A6 · EDITORIAL]** The Earth body parameters given here are **chosen to reproduce Appendix I**, not independently sourced. They are internally consistent and they are not a citation; a body parameter set intended for use must satisfy Rule C on its own provenance.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).

# Appendix G — Body and anchor schema

```hjson
// bodies/mars.hjson
{
  id: mars
  name: { en: Mars, ru: Марс }
  rotation_period: { value: "88642.6632 s", epoch: "J2000.0", rate: null,
                     valid: ["-1e6 yr", "+1e6 yr"], source: "IAU WGCCRE 2015, doi:…" }
  solar_day:       { value: "88775.244 s",  epoch: "J2000.0", … }
  orbital_period:  { value: "686.9726 d",   epoch: "J2000.0", … }   // tropical
  obliquity:       { value: "25.19 deg",    epoch: "J2000.0", … }
  formation:       { window: ["-4.6 Gyr", "-4.5 Gyr"], source: "…" }
  satellites: [
    { id: phobos, orbital_period: { value: "27553 s",  … }, retrograde: false }
    { id: deimos, orbital_period: { value: "109123 s", … }, retrograde: false }
  ]
}

// anchors/mars-d.hjson
{
  calendar: mars-d
  revision: 1
  phase:   { kind: mean_solar_midnight, meridian: airy-0 }
  tick:    "<exact tick value>"          // UC-P12 deliverable, empirically determined
  window:  ["<lo tick>", "<hi tick>"]    // REQUIRED, must contain tick
  method:  "…ephemeris or observation used, with stated uncertainty…"
  source:  "…citation…"
}
```

Every parameter MUST carry `epoch`, `valid`, `source`, and its verbatim as-measured value (Rules C, Y.1). Values parse into exact rational ticks; anything requiring finer than 10⁻³⁰ s is `UCAL-E0043`. Anchor files carry no default: an absent anchor is `UCAL-E0062`, never a fallback.

# Appendix H — Integer numerics (`ucal-core::num`)

**H.1 Widening multiply-divide.** `mul_div(a, n, d)` computes `a × n / d` with an intermediate twice the backend width (`U1024` by default) and a directed remainder, returning `(quotient, remainder)`. Callers choose the direction; nothing rounds implicitly.

**H.2 Integer square root with directed rounding.** `isqrt_floor(x)` and `isqrt_ceil(x)` by integer Newton iteration with the post-condition `r² ≤ x < (r+1)²` asserted. Rational square roots are enclosed as `[isqrt_floor(num·S²/den)/S, isqrt_ceil(num·S²/den)/S]` for a declared scale `S`, which is recorded in the result.

**H.3 Interval arithmetic.** `RatInterval` operations round outward: `lo` toward −∞, `hi` toward +∞. Multiplication takes the min and max of the four endpoint products. Division requires the divisor interval to exclude zero (`UCAL-E0070`).

**H.4 Certified quadrature.** For a monotone integrand on `[a, b]`

> **[D-A15 · EDITORIAL]** The monotone case **does not apply to the ΛCDM integrand**, so the fallback this section describes is the only path actually taken. With `f(u) = u/√g(u)`, the derivative's numerator changes sign at `u ≈ 0.6038` (`z ≈ 0.656`), so every query below that redshift — including the present epoch — straddles the turn. H.4's requirement that monotonicity be *asserted, not assumed* is what makes this discoverable rather than a latent unsoundness.
> Reasoning: [`SPEC-DELTAS.md`](SPEC-DELTAS.md).
, subdivide into `2^depth` panels; on each panel the endpoint values bound the integral from below and above; sum the lower bounds and the upper bounds separately with outward rounding. The result is a rigorous enclosure whose width shrinks monotonically with `depth`. Monotonicity of the ΛCDM integrand over the integration range MUST be asserted, not assumed; where it fails, the panel is bounded by the interval extension of the integrand.

**H.5 Continued fractions.** `cf_expand(r, max_depth)` and `convergents(cf)`, exact throughout, used by `derive_leap_rule` and `derive_cycles`. Both MUST return the full sequence, not only the selected convergent, so every derivation is auditable.

**H.6** No transcendental function is implemented or called anywhere in the workspace (N13).

# Appendix I — Derived calendars (normative test vectors)

Computed exactly from the parameters cited in Appendix G. Implementations MUST reproduce these sequences.

## I.1 Earth — intercalation

`orbital_period / solar_day = 365.242190` (mean tropical year in mean solar days). Fractional part 0.242190, continued fraction `[0; 4, 7, 1, 3, 24, 6, 2, 2]`.

| convergent | rule | value | drift | 1 day slips in |
|---|---|---|---|---|
| 1 | **1/4** | 0.250000000 | 7.810 d / 1000 yr | 128 yr |
| 2 | 7/29 | 0.241379310 | 0.811 | 1 234 yr |
| 3 | 8/33 | 0.242424242 | 0.234 | 4 269 yr |
| 4 | 31/128 | 0.242187500 | 0.0025 | 400 000 yr |
| 5 | 752/3105 | 0.242190016 | 0.00002 | 6.21×10⁷ yr |
| 6 | 4543/18758 | 0.242189999 | ~0 | 9.38×10⁸ yr |

Convergent 1 is the Julian rule. **97/400 (Gregorian) does not appear at any depth**: it is less accurate than convergent 3 while using a denominator twelve times larger, and convergent 4 is 124× more accurate. §21.3 requires a test asserting its absence.

## I.2 Earth — grouping cycle

`orbital_period / synodic_month = 12.368266761`, continued fraction `[12; 2, 1, 2, 1, 1, 17, 2, 1]`.

| convergent | cycle | value | error |
|---|---|---|---|
| 1 | 12 / 1 yr | 12.000000000 | 3.68×10⁻¹ |
| 2 | 25 / 2 yr | 12.500000000 | 1.32×10⁻¹ |
| 3 | 37 / 3 yr | 12.333333333 | 3.49×10⁻² |
| 4 | 99 / 8 yr | 12.375000000 | 6.73×10⁻³ |
| 5 | 136 / 11 yr | 12.363636364 | 4.63×10⁻³ |
| 6 | **235 / 19 yr** | 12.368421053 | 1.54×10⁻⁴ |
| 7 | 4131 / 334 yr | 12.368263473 | 3.29×10⁻⁶ |

Convergent 6 is the Metonic cycle, derived with no special-casing — the mechanism recovers a cycle known since antiquity from the tick, the datum, and the body's periods.

## I.3 Mars — intercalation

`orbital_period / solar_day = 668.592165627` sols. Fractional part 0.592165627, continued fraction `[0; 1, 1, 2, 4, 1, 2, 2, 1]`.

| convergent | rule | value | drift | 1 sol slips in |
|---|---|---|---|---|
| 1 | 1/1 | 1.000000000 | 407.83 sol / 1000 yr | 2 yr |
| 2 | 1/2 | 0.500000000 | 92.17 | 11 yr |
| 3 | 3/5 | 0.600000000 | 7.834 | 128 yr |
| 4 | 13/22 | 0.590909091 | 1.257 | 796 yr |
| 5 | **16/27** | 0.592592593 | 0.427 | 2 342 yr |
| 6 | 45/76 | 0.592105263 | 0.060 | 16 566 yr |
| 7 | 106/179 | 0.592178771 | 0.013 | 76 079 yr |

## I.4 Mars — grouping cycle

None. Phobos's synodic period is 0.4500 sol and Deimos's 5.3629 sols; under the default bounds (5–100 solar days) neither qualifies as a month-scale cycle for a 668-sol year. `cycles() == []`, so `DerivedFields` for `mars-d` contains no month or weekday field; requesting one is `UCAL-E0064`.

## I.5 Titan — intercalation

`orbital_period / solar_day = 673.983719443` Titan days (Saturn's tropical year over Titan's solar day). Fractional part 0.983719443, continued fraction `[0; 1, 60, 2, 2, 1, 2, 1, 11]`.

| convergent | rule | value | drift | 1 Titan day slips in |
|---|---|---|---|---|
| 1 | 1/1 | 1.000000000 | 16.281 Tday / 1000 yr | 61 yr |
| 2 | **60/61** | 0.983606557 | 0.113 | 8 859 yr |
| 3 | 121/123 | 0.983739837 | 0.020 | 49 032 yr |
| 4 | 302/307 | 0.983713355 | 0.006 | 164 270 yr |
| 5 | 423/430 | 0.983720930 | 0.001 | 672 208 yr |

Titan is tidally locked to Saturn, so its solar day and its orbit about Saturn coincide, and its year is Saturn's orbit about the Sun. The mechanism handles this with no special case, which is the point of Rule K.

## I.6 Anchors

Not included: anchor determination is empirical (Rule J, N15) and is a UC-P12 deliverable, gated by GE-3. Until an `Anchor` record exists, `earth-d`, `mars-d`, and `titan-d` are complete in units, intercalation, and cycles, and incomplete in phase — a state the API represents explicitly (`UCAL-E0062`) rather than defaulting away.

---

*End of RFC UCAL-1, final draft.*
