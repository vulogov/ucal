# Spec deltas against RFC UCAL-1 (final draft, 2026-07-29)

Amendments to the authoritative RFC, established during UC-P0 verification.
Each entry records what the RFC says, what exact integer arithmetic shows, and
what this implementation does. The RFC text itself is unchanged; this file is
the delta of record, and every item here is covered by a test.

Status key: **CORRECTION** — RFC is wrong, implementation follows the corrected
value. **AMENDMENT** — normative change adopted by decision. **EDITORIAL** — no
behavioural effect.

---

## D-A1 — WITHDRAWN. The 44 BC fixture is correct.

An earlier revision of this file claimed RFC Appendix C's `-0043-03-15T00:00:00
TT` fixture was one day late. **That claim was wrong and is withdrawn.** The
RFC's value is correct:

```
ticks  8070203977843789392286957152835637180530466139316558837890625
ucid   0000000000050PM5STSSZT2034C3TGX8CMS2Z79C0SGBSBM29FJ1
```

The error was in the checking oracle, not the RFC. It computed the day count
with Hinnant's era algorithm, which reads

```
era = (y2 >= 0 ? y2 : y2 - 399) / 400
```

The `- 399` adjustment exists to make *truncating* division (C, Rust) behave
like flooring division. Applying it on top of a language whose division already
floors (Python `//`) double-counts: for `y2 = -43` it yields `era = -2` and
hence `yoe = 757`, outside the algorithm's valid `[0, 399]`, and the result is
one day off. The bug is invisible for non-negative years, which is why seven of
eight fixtures agreed and only the sole BC fixture did not.

Correct: `days_from_civil(-43, 3, 15) = -15632`, which cross-checks against
`-43 × 365.25 + 74 = -15632`.

**Retained as a regression.** Negative astronomical years are the only place
this class of bug appears, so the harness keeps the 44 BC fixture and adds
`-0001-01-01`, `-0100-01-01` and `-4712-01-01` (the Julian Day epoch year) as
further negative-era vectors. Appendix C's tick values are **8 of 8 correct**.

A proleptic **Julian** `44 BC-03-15` vector (= proleptic Gregorian
`-0043-03-13`, two days earlier) remains worth adding at UC-P8 to exercise
§8.5's Julian input path, since that is the calendar the historical Ides of
March is a date in. That is a coverage note, not a correction.

---

## D-A2 — `ORIGIN_OFFSET` has 61 trailing base-5 zeros, not 62  (EDITORIAL)

Appendix A annotates `ORIGIN_OFFSET` as "203 bits; 88 base-5 digits; 62
trailing base-5 zeros". Bit length and digit count are correct. The valuation
is 61: `ORIGIN_OFFSET = beats × 5^60` with `beats = 9 304 311 741 502 590 385`,
and `beats / 5 = 1 860 862 348 300 518 077` is not divisible by 5, so `beats`
contributes exactly one factor of 5.

No behavioural effect — §2.4 requires ≥ 60 (zero in all tiers below T0), which
61 satisfies. Recorded because §21.3 forbids hand-transcribed constants the P0
harness does not reproduce, and this annotation is one.

---

## D-A3 — Appendix B seconds column is imprecise  (EDITORIAL)

The informative seconds column disagrees with exact evaluation of `5^e /
SECOND` in the 5th significant figure for the upper tiers:

| tier | Appendix B | exact |
|---|---|---|
| T5 deep | 1.3934×10¹⁶ | 1.3936×10¹⁶ |
| T4 drift | 4.4590×10¹² | 4.4595×10¹² |
| T3 span | 1.4269×10⁹ | 1.4271×10⁹ |
| T2 sweep | 4.5660×10⁵ | 4.5666×10⁵ |
| T−6 | 5.0212×10⁻²³ | 5.0210×10⁻²³ |

The pattern is consistent with chain-computing each row from its neighbour by
×3125 with rounding retained at each step. Appendix B's *human* column
(441.607 Myr, 141.314 kyr, …) is correct, as are T1, T0 and T−1…T−3.

**Closed at UC-P10.** Per §13.5 the tier table, the locale table and the
documentation table now come from one generated source of truth: `docs/TIERS.md`
is produced by `cargo run -p xtask -- gen-docs` from `ucal_core::tier` and
`ucal_core::locale`, and `check-docs` fails if the committed file is stale.

Every value is rendered from the exact rational `5^e / SECOND` in **one step**
under half-even rounding, rather than chained from the neighbouring row — which
is the mechanism that produced the published column's drift. The generated table
carries 1.3936×10¹⁶ for the deep, and a test asserts explicitly that it is *not*
Appendix B's 1.3934×10¹⁶.

The drift test bites: re-injecting Appendix B's value into the committed file
fails `generated_docs_are_current` with the offending line number.

---

## D-A4 — Appendix C human forms are truncated at T−5  (CORRECTION)

Appendix C presents its human forms as tick-exact, quoting five sub-beat groups
(T−1…T−5). That is tick-exact only for an instant whose base-5 valuation
reaches 35. §2.4 guarantees only **30** for a whole SI second, so T−6 is
generally non-zero and the printed form is a T−5 *window*, not the tick it is
labelled as. This is failure mode F2.

Measured valuations and the dropped T−6 group:

| fixture | v5 | lowest non-zero tier | T−6 group | Appendix C's form |
|---|---|---|---|---|
| absolute zero | — | — | — | exact |
| SI_EPOCH | 61 | T0 | — | exact (whole beat) |
| −0043-03-15 | 32 | T−6 | 0925 | **lossy** |
| Apollo 11 | 31 | T−6 | 0265 | **lossy** |
| Unix epoch | 32 | T−6 | 2550 | **lossy** |
| GPS epoch | 33 | T−6 | 1000 | **lossy** |
| J2000.0 | 32 | T−6 | 0800 | **lossy** |
| 2026-07-29 | 32 | T−6 | 1100 | **lossy** |
| Earth formation | 39 | T−5 | — | exact |
| recombination | 36 | T−5 | — | exact |

Note the two non-civil fixtures: Earth formation and recombination are whole
Julian-year multiples, whose valuations (39 and 36) happen to exceed 35, so
Appendix C's five groups *are* tick-exact for them. The defect is confined to
the whole-SI-second civil fixtures, and the invariant the harness asserts is
the general one — a fixture's form is lossy at T−5 exactly when its valuation
is below 35.

**Implementation (refined by D-A8):** tick-exact rendering descends to **T−12**,
not to the lowest non-zero tier. An earlier statement of this delta said "six
sub-beat groups"; that was imprecise. Under §6.1 the last group's tier *is* the
stated precision, so stopping at T−6 states a T−6 window rather than a tick. The
six trailing zero groups are guaranteed by §2.4 and must be written. The vector file
records, per fixture, the tick-exact form, the T−5 truncation (which reproduces
Appendix C's quote exactly, for all ten fixtures), and the closed window
`[v, v + 5^35 − 1]` the truncation denotes — so the truncation stays
representable and testable but is never labelled as a tick.

The RFC's `digit5` line for 2026-07-29 carries 22 groups against 18 for
T5…T−12. The range is **T9…T−12**, i.e. the value padded up four tiers; the
harness reproduces the RFC's string exactly at that range. The generated codec
emits whatever tier range it is asked for and the vector file states the range
in the field name.

---

## D-A5 — Grouping cycles are declared per body, not admitted by a global bound  (AMENDMENT)

**Adopted by decision.** Amends Rule K.3, §9.6, D-11 and Appendix I.4.

### The defect

§9.6 admits a satellite as a grouping cycle if its synodic period lies within
`cycle_bounds`, default 5–100 solar days (D-11). Evaluating the RFC's own
formula on the RFC's own Appendix G parameters:

```
Mars solar day = 88775.244 s        Deimos P_orb = 109123 s
synodic = 1 / |1/P_orb − 1/P_solar_day| = 476092.8405 s = 5.362901 sols
```

which is the 5.3629 sols Appendix I.4 itself prints — and it is inside
[5, 100]. So the specified algorithm **admits** Deimos, while §9.6 states Mars
"derives nothing", Appendix I.4 states `cycles() == []`, and §21.3 assertion 7
requires a test that Mars yields no cycle. Under the algorithm as written,
`mars-d` acquires a Deimos cycle of 124.669870235 cycles per Mars year.

> **Correction, added at UC-P11.** The trigger above is itself an artefact of a
> second defect, recorded as D-A12: §9.6's synodic formula is written against the
> primary's *solar day* rather than its *year*, and so computes the wrong
> quantity. Under the corrected formula Deimos's synodic period is **1.2315
> sols**, not 5.3629, and it falls outside [5, 100] comfortably — so the
> specified algorithm never admitted it. Appendix I.4 reached the right
> conclusion by the wrong route.
>
> The amendment below still stands, on its own merits: the bound is calibrated on
> Earth's Moon, "month-like" is not derivable, and Rule J's pattern is the right
> one. But it is no longer motivated by a live admission, and this record should
> not be read as claiming one.

### Why the bound is the wrong instrument

The bracket 5–100 solar days is calibrated on Earth's Moon (synodic 29.53
solar days, comfortably interior). It is an Earth-derived constant embedded in
the one mechanism Rule K exists to keep Earth-free, and it is therefore an
instance of failure mode F9 — "Earth becomes the template rather than an
instance". Tuning the bound to exclude Deimos would fit the constant to a
predetermined answer about a non-Earth body, compounding the defect. Whether a
satellite is "month-like" is not derivable, because *month-like* is an Earth
predicate.

### The amendment

Apply the pattern Rule J already establishes for phase. Phase is not derivable
from the tick, the datum and the body's periods, so Rule J makes the anchor a
declared, cited, interval-valued constant per body and makes its absence
`UCAL-E0062` rather than a default (Rule J.3: "not a guess and not a fallback
to another body"). Grouping admission has the same character.

1. **Rule K.3 as amended.** `Cycles` are optional grouping periods derived from
   a satellite **named by the calendar**. A calendar MAY declare at most one
   `grouping_satellite`, which MUST cite the ground for the choice. The
   *structure* of the cycle — the commensurability convergents — remains
   derived by continued-fraction expansion of `orbital_period /
   synodic_period` and MUST NOT be declared. No calendar may declare a
   grouping table.
2. A calendar declaring no `grouping_satellite` has `cycles() == []`. This is
   not a fallback and not an error at construction; requesting a cycle field
   from such a calendar is `UCAL-E0064`.
3. `Satellite` gains no privileged status: the declaration names an existing
   entry in the body's `satellites` list, and naming an absent satellite is
   `UCAL-E0064`.
4. **D-11 as amended.** `cycle_bounds` ceases to be an admission gate. It is
   retained as an OPTIONAL sanity filter, default absent, and when present a
   declared satellite falling outside it produces `UCAL-W0003`-style warning
   rather than silent rejection.

This preserves the RFC's house style — declared, cited input; derived structure
— exactly as Rule C declares body parameters and derives units from them.

### Consequences

- **`earth-d`** declares the Moon and still derives the Metonic cycle 235/19
  with no special-casing. Appendix I.2 is untouched.
- **`mars-d`** has `cycles() == []` because it declares no grouping satellite,
  not because 5.3629 fell on the convenient side of a constant chosen for the
  Moon. Appendix I.4's conclusion stands; its justification is replaced.
  I.4 retains the Deimos figures as *what a Deimos cycle would yield had
  `mars-d` declared one*, computed by the same generic path:

  | convergent | cycles / Mars year | error |
  |---|---|---|
  | 1 | 124/1 | 6.699×10⁻¹ |
  | 2 | 125/1 | 3.301×10⁻¹ |
  | 3 | 374/3 | 3.204×10⁻³ |
  | 4 | 12841/103 | 3.268×10⁻⁵ |
  | 5 | 26056/209 | 1.378×10⁻⁵ |
  | 6 | 38897/312 | 1.560×10⁻⁶ |

- **§21.3 assertion 7** survives in substance: Earth's grouping sequence
  contains 235/19; Mars yields no cycle. The Mars half is now a statement about
  Mars's declared data rather than about a global default.
- **Appendix G** gains `grouping_satellite` in the calendar (not body) schema,
  with `citation` REQUIRED.

---

## D-A6 — Earth body parameters are chosen to reproduce Appendix I  (EDITORIAL)

Appendix I states its tables are "computed exactly from the parameters cited in
Appendix G", but Appendix G exhibits only Mars, and no standard lunar period
reproduces I.2's ratio 12.368266761 exactly:

| sidereal month | derived synodic | year / synodic |
|---|---|---|
| 27.321661 d | 29.530680860 d | 12.368227869 |
| 27.321582 d | 29.530588569 d | 12.368266523 |
| — | — | **12.368266761** (I.2) |

Appendix I's printed ratios are treated as the normative pinned vectors, and
each body's declared parameters are chosen consistent with them to the printed
precision, with `as_measured` and `citation` per Rule C. Where a body's
parameters and Appendix I's ratio disagree beyond printed precision, the
divergence is recorded in the body file rather than reconciled silently.

---

## Verified without amendment

Reproduced bit-exactly by independent exact-integer computation:

- Every Appendix A constant: `BEAT`, `SECOND`, `ORIGIN_OFFSET`,
  `BIG_BANG_CLAIM` half-width, `DOMAIN_MAX`.
- The whole §2.2 `datum_provenance` chain: `AGE_s` = 435 084 631 200 000 000,
  `AGE_ticks`, `beats` = 9 304 311 741 502 590 385 under half-even rounding,
  and the residual −318 856 914 364 362 819 469 533 860 683 441 162 109 375
  ticks = −0.017190364 s.
- `BIG_BANG_CLAIM` is *exactly* 0.020 Gyr of Julian years; 0.1451 % of
  `ORIGIN_OFFSET`; 141.53 drifts.
- All §2.4 alignment invariants: v5(`SECOND`) = 30, v5(`NANOSECOND`) = 21,
  v5(`ORIGIN_OFFSET`) = 61, and the valuation floors hold across whole-second
  and whole-nanosecond offsets.
- Informative quantities: tick length 5.3912470000001396×10⁻⁴⁴ s, implied age
  435 084 631 200 000 000.000000 s = 13.787000000000 Gyr, 1 s = 21.385061835
  beats, declared/measured tick deviation +2.583×10⁻¹⁴, UCID ceiling
  1.978172×10²⁶ yr, domain ceiling 2.290567×10¹⁰³ yr, present epoch 31.220
  deeps.
- Appendix B's human column at every named tier.
- Appendix C: **all 8** tick fixtures, all 10 UCIDs, all human forms at the
  precision printed, all 10 T−5 sub-beat quotes, Earth formation,
  recombination, and the 22-group `digit5` line at range T9…T−12.
- Appendix I.1 Earth intercalation: cf `[0; 4, 7, 1, 3, 24, 6, 2, 2]`,
  convergents 1/4 · 7/29 · 8/33 · 31/128 · 752/3105 · 4543/18758, every drift
  and slip figure, and both accuracy claims about 97/400 — 8/33 is more
  accurate with a denominator 12.1× smaller, 31/128 is 124.0× more accurate.
  **97/400 is absent at every depth.**
- Appendix I.2: Metonic 235/19 present as a convergent.
- Appendix I.3 Mars and I.5 Titan intercalation: cf and convergents exact.
- Appendix I.4: Phobos synodic 0.4500 sols, Deimos 5.3629 sols.


---

## D-A7 — Full-width encode is 45 `divmod` steps, not 44  (CORRECTION)

Appendix F states "Full-width encode is 44 `divmod` steps" and §13.1 repeats it
as "44 steps at full width, not 221". The correct count is **45**.

`2^512 − 1` has 221 base-5 digits. At five digits per tier that is 44.2 groups,
which rounds up to 45 — and the 45th group is load-bearing rather than a padding
artefact:

```
domain_max / 5^220 mod 3125 = 2      (T32's group, non-zero)
tier grid T-12 ..= T32               = 45 tiers
```

The "not 221" contrast the RFC draws is the real point and is correct: the
Appendix F loop takes one step per *tier*, not one per base-5 digit, so it is 45
steps rather than 221. Only the number is wrong.

`TIER_COUNT` is 45 throughout the implementation, and
`full_width_encode_takes_45_steps_not_44` pins it.

---

## D-A8 — What a printed form means, and how each form is anchored  (AMENDMENT)

Resolves an inconsistency between §6.1 and Appendix C, and settles a question
§6 leaves open.

### The inconsistency

§6.1 states that **the last group's tier is the stated precision**. Combined with
Rule T, a rendering therefore denotes the closed interval `[v, v + 5^e − 1]`.

Appendix C reads the other way. It prints SI_EPOCH as six groups ending at T0 and
annotates "sub: all tiers below T0 are zero", treating the omission of the
sub-beat part as an assertion of *exactness*.

Both cannot hold. **§6.1's reading is adopted**, because the alternative
institutionalises failure mode F2: if trailing omission meant "exact", then
reading a T−5 form would zero-fill it to a tick, which is precisely "precision
invented by zero-filling a truncated timestamp". Rule T and F2 agree.

Consequences:

- Tick-exact text runs down to T−12. For a whole SI second the last six groups
  are zero by §2.4 and must still be written.
- Appendix C's printed human forms denote windows, not ticks — the observation
  already recorded as D-A4.

### Anchoring

Neither form states which tier it begins at, so each needs an anchor. §6 supplies
one only for the human form; the digit form's is derived here from D-9 and
Rule S.

| form | anchor | expressible precision |
|---|---|---|
| human (`UC1`) | §6.4's `:` fixes the group before it as T0; with no `:`, the last group is T0 | **T0 and finer only** |
| digit (`UC1/5`) | always begins at T32, the highest tier the domain holds; the group count fixes the precision | the whole grid |
| named (§6.5) | self-describing — each term carries its tier | the whole grid |

The human form's limitation is a real constraint, not an implementation choice.
Its only anchor is the sub-beat separator, so a rendering that stopped above T0
would be read back as though its last group *were* T0, silently changing the
value by whole tiers. Rather than invent syntax the RFC does not define, the
implementation reports `UCAL-E0006` and directs the caller to the digit or named
form. `coarse_precision_needs_the_digit_or_named_form` pins this.

The digit form's top anchor is what makes D-9 ("canonical text form for parse and
sort is the base-5 digit form") and Rule S ("lexicographic order equals
chronological order ... unless zero-padded to a fixed tier width") true at the
same time: a fixed top plus a fixed precision gives a fixed width, and
`padded_digit_form_sorts_chronologically` verifies the resulting order matches the
numeric one over a 130-value sample spanning the domain.

Appendix C's `digit5` line for 2026-07-29 carries 22 groups, which is the range
T9…T−12 — neither the canonical fixed width nor a top-anchored form. It is
reproduced exactly by the range-explicit encoder in the vector file, and is not
what the canonical renderer emits.


---

## D-A9 — §6.6 needs a calendar-id grammar  (AMENDMENT)

§6.6 requires every local calendar rendering to carry its id and kind, and gives
examples — `earth-d/1:`, `earth-civil:`, `mars-d/1:` — but never defines what an
id may look like. Without a grammar the notation is ambiguous, because the
**body** of a rendering may itself contain colons:

```
earth-civil: 2026-07-29T00:00:00Z
```

Splitting at the first colon yields a "calendar id" of `2026-07-29T00`, which
parses as a legacy calendar and silently succeeds. The failure is quiet, which is
the worst kind.

**Adopted grammar** — the narrowest that admits every id the RFC uses:

```
id  =  lowercase-letter , { lowercase-letter | digit | "-" }
```

Requiring a leading *letter* is what disambiguates a qualifier from a date, since
every date form in this specification begins with a digit. `is_valid_calendar_id`
implements it and `calendar_id_grammar` pins both the accepted and the rejected
cases; a malformed or absent qualifier is `UCAL-E0007`, as §6.6 requires.

Two further consequences follow from Rule J.5 and are enforced at the same point:

- a **derived** rendering must state its anchor revision (`earth-d/1`, not
  `earth-d`), because renderings carry the revision so values from different
  determinations are never silently compared;
- a **legacy** rendering must not, because a legacy calendar has no anchor — it
  is a declared table, not a determination.


---

## D-A10 — Appendix A's "implied age" is the unrounded input, not the quotient  (EDITORIAL)

Appendix A's informative block reads:

```
implied age = ORIGIN_OFFSET / SECOND = 435 084 631 200 000 000.0 s
            = 13.787 000 000 000 Gyr of Julian years
```

That number is `AGE_s`, the value *before* rounding. `ORIGIN_OFFSET` is the datum
rounded down to a whole beat, so the quotient is smaller by exactly the residual
the same appendix documents:

```
ORIGIN_OFFSET / SECOND = 435 084 631 199 999 999.982 810 s
             AGE_s     = 435 084 631 200 000 000        s
             residual  =              -0.017 190 364    s
```

The two lines are consistent with each other only if the quotient is read to
eighteen significant figures. No behavioural effect — the residual is declared
immediately above it, and both values are informative under Rule A.5.

Worth recording for two reasons. First, §21.3 requires CI to fail on any
hand-transcribed constant the P0 harness does not reproduce, and this is one.
Second, the discrepancy was invisible until the rendering became exact: the P0
harness originally printed the quotient through an `f64`, which cannot represent
`4.35×10^17` to the unit and so displayed the rounder value. Rule E's prohibition
on floats is what surfaced it.

`ucal datum` prints the exact quotient, labelled as a consequence of the declared
datum rather than as a measurement (§19.2, Rule Q.1), with the residual alongside.


---

## D-A11 — Obliquity cannot be a `RatedParam`  (CORRECTION)

§9.2 types `Body::obliquity` as `Option<RatedParam>`. Rule C requires a
`RatedParam` to be "stored for computation as an exact rational of **ticks**",
and an obliquity is an angle, not a duration. The two requirements cannot both
hold for the same type.

**Implementation:** obliquity gets a distinct type, `AngleParam`, holding exact
degrees with the published value verbatim and a citation — the same Rule Y.1
obligations, minus the tick storage that does not apply.

Nothing in Rule K consumes it. Intercalation comes from
`orbital_period / solar_day` and grouping from a satellite's synodic period, so
the change is confined to the type. Obliquity is carried because it is what gives
a body seasons, and a future seasonal overlay would need it.

---

## D-A12 — §9.6's synodic formula contradicts Appendix I.2  (CORRECTION)

§9.6 defines a satellite's synodic period as

```
synodic = 1 / |1/P_orb − 1/P_solar_day|
```

measured against the primary's **solar day**. Appendix I.2 divides the year by
the **synodic month** to reach its ratio of 12.368266761. These are different
quantities, and the difference is not small:

| | §9.6 formula (vs solar day) | Appendix I.2 (vs year) |
|---|---|---|
| Earth + Moon | 1.038 d | 29.530589 d |
| resulting ratio | 351.87 | **12.368267** |

§9.6's formula gives the interval between successive *moonrises* — the "lunar
day" — not the Moon's phase cycle. Appendix I.2's ratio is only reproducible from
the year-relative form.

**The year-relative form is adopted**, for three reasons:

1. It is the standard definition of a synodic period.
2. It is the only one that reproduces Appendix I.2, which §21.3-7 pins.
3. A grouping cycle *is* a phase cycle. It is what every lunar calendar counts,
   and what makes a "month" mean anything.

### Consequence for Appendix I.4

Appendix I.4's printed figures — Phobos 0.4500 sols, Deimos 5.3629 sols — are
computed with §9.6's formula and are therefore the wrong quantity. Under the
corrected one:

| satellite | §9.6 (printed) | corrected |
|---|---|---|
| Phobos | 0.4500 sols | **0.3105 sols** |
| Deimos | 5.3629 sols | **1.2315 sols** |

Neither is remotely month-like, and neither falls inside D-11's original
[5, 100] bound. **Appendix I.4's conclusion — that Mars yields no grouping cycle
— is therefore correct**, and correct for the original bounds; only its working
was wrong. See the correction note in D-A5, whose stated trigger this
invalidates.


---

## D-A13 — A drift bound is a rate, not a duration  (CORRECTION)

§9.5 types `derive_leap_rule`'s third parameter as `max_drift: &Delta`. A `Delta`
is an unsigned count of ticks — a duration. A drift bound is a *rate*: D-12
states the default as "1 day / 10 000 yr", which is drift per unit time and
cannot be a tick count.

**Implementation:** a `DriftBound { days, per_years }`, expressed in the body's
**own** local days and local years.

That choice is the same lesson D-A5 records. A bound stated in SI seconds would
be an Earth-derived constant sitting inside the one mechanism Rule K exists to
keep Earth-free. Stated in local units, "one day per ten thousand years" means
the same *thing* on Mars as on Earth without meaning the same *duration* — and
the test `the_bound_is_body_relative_not_earth_calibrated` shows the identical
bound selecting different rules on the two bodies, which is the point.

### What the default bound derives

| body | walked | chosen at D-12's default | one day slips in |
|---|---|---|---|
| Earth | 1/4 · 7/29 · 8/33 · **31/128** | 31/128 | 400 000 yr |
| Mars | 1/1 · 1/2 · 3/5 · 13/22 · 16/27 · **45/76** | 45/76 | 16 566 yr |

Earth's answer is worth noting: **31/128** is the convergent Appendix I.1 itself
singles out as 124 times more accurate than the Gregorian 97/400, with a
denominator three times smaller. The mechanism reaches it without being told to,
and never produces 97/400 at any depth.

---

## D-A14 — §10.3's integral cannot be quadratured as written  (CORRECTION)

§10.3 states the age at redshift `z` as

```text
t(z) = (1/H0) ∫_z^∞ dz' / [ (1+z') E(z') ]
```

and §10.6 requires "certified interval quadrature". The two are incompatible:
the integral is **improper**. Its upper limit is infinite, and no subdivision of
`[z, ∞)` into finitely many panels bounds it. An implementation that truncates
the upper limit at some large `z_max` and calls the result certified has quietly
replaced a proof with a guess about the tail.

**Implementation:** substitute `u = 1/(1+z)`, giving

```text
t(z) = (1/H0) ∫_0^{u₀} u du / √(Ω_r + Ω_m u + Ω_Λ u⁴),   u₀ = 1/(1+z)
```

`z → ∞` becomes `u → 0`, the range is compact, and the integrand is bounded and
smooth on the whole of it — the `Ω_r > 0` term keeps the denominator away from
zero, so even the endpoint needs no limiting argument. The enclosure is then
rigorous rather than rigorous-modulo-a-tail.

The general point is worth stating: **a specification that demands exactness and
then writes an improper integral has not finished writing the integral.** The
substitution is not an implementation detail; it is the step that makes §10.6's
requirement satisfiable at all.

---

## D-A15 — Appendix H.4's monotone case does not apply to ΛCDM  (EDITORIAL)

H.4 permits panel bounds from the endpoints "for a monotone integrand" and
requires monotonicity to be "asserted, not assumed; where it fails, the panel is
bounded by the interval extension".

It fails. With `f(u) = u/√g(u)` and `g(u) = Ω_r + Ω_m u + Ω_Λ u⁴`, the numerator
of `d(f²)/du` is `u(2Ω_r + Ω_m u − 2Ω_Λ u⁴)`, which changes sign at

```text
u ≈ 0.6038   (z ≈ 0.656)
```

for Planck 2018 parameters. So `f` rises and then falls, and every query below
`z ≈ 0.66` — including the present epoch, the most commonly asked one — has an
integration range that straddles the turn.

**Implementation:** the interval extension is used on *every* panel, with no
monotone fast path. Since `g` is increasing on `[0,1]` for non-negative
densities, the extension is

```text
f([a,b]) ⊆ [ a/√g(b) , b/√g(a) ]
```

which needs no case analysis and is valid whether or not the panel contains the
turn. `LambdaCdm::monotonicity_turns_at` returns the crossing as an exact
rational enclosure, so H.4's "asserted, not assumed" is discharged by a test
rather than by a comment.

No amendment: H.4 anticipated this case and prescribed the remedy. The entry is
here because the RFC's phrasing invites an implementer to look for the monotone
path first, and there isn't one.

---

## §21 gated experiments — outcomes

| id | question | outcome |
|---|---|---|
| GE-1 | is certified quadrature fast enough? | **kill criterion fired**; default depth 12, depth an explicit argument |
| GE-2 | what scale gives a ≤ 1-tick enclosure at `z = 1100`? | **unreachable by ~55 orders**; width published, `UCAL-W0004` always set |
| GE-3 | can body anchors be fixed without inventing a convention? | Earth ±1 ms, Mars ±1 s, **Titan: none** — no convention exists to adopt |
| GE-4 | will the fixed-width integer library churn? | fired on arrival (bnum 0.14 API change); absorbed entirely in `TickInt` |
| GE-5 | is `no_std` + no-alloc viable? | **yes** for `ucal-core` with text forms off; `alloc` needed above it |
| GE-6 | CMB-anchored datum for a hypothetical `UC-2`? | **kill criterion fired**; the derived enclosure is ~10× the published uncertainty. D-21 stands |

### GE-1, measured

Release build, `z = 1100`, `aarch64-apple-darwin`:

| depth | panels | wall | arithmetic width |
|---:|---:|---:|---:|
| 4 | 16 | 1.3 ms | 64 251 yr |
| 8 | 256 | 37 ms | 4 011 yr |
| 12 | 4 096 | 476 ms | 251 yr |
| 14 | 16 384 | 2.01 s | 63 yr |
| 16 | 65 536 | 8.66 s | 16 yr |

Cost grows about 4× per depth step — twice the panels, and larger rationals in
each. Depth 24 is therefore on the order of hours, not the two seconds GE-1 set
as its threshold. **The kill criterion fires**: the default is depth 12, and
`--depth` is the high-precision mode GE-1 asked to be exposed.

The kill costs nothing scientifically, and Rule X is what makes that visible. At
depth 12 the arithmetic width at `z = 1100` is 251 years; the **parameter**
width — Planck's own error bars, propagated — is 10 917 years. The quadrature is
already forty times sharper than the measurement it integrates. Depth 16 would
buy a factor of sixteen on the term contributing two per cent of the total, for
eighteen times the wall clock. A single merged tolerance, which F8 is precisely
the habit of reporting, would have hidden that entirely.

### GE-2, measured

No scale reaches one tick, and the scale is not the obstacle. At the default
depth the enclosure at `z = 1100` is 360 432…371 600 yr — about 11 000 years, or
`1.6 × 10^55` ticks. Closing that on arithmetic alone would need subdivision
depth near 180. The real constraint is upstream: a cosmological age is derived
from inputs measured to four significant figures, and asking it to land on a
Planck tick is asking for fifty digits nobody measured.

`UCAL-W0004` is therefore set on every cosmological result, and the width is
reported rather than rounded away — Rule T applied to the one quantity §21 hoped
might escape it.

### GE-5, measured

`ucal-core` builds `no_std` with no allocator at all, for
`wasm32-unknown-unknown`, with warnings denied. What goes with the allocator is
exactly what GE-5 predicted: **radix formatting**. `parse`, `render`, the
continued-fraction expansions and `to_dec_string` are `alloc`-gated; the tick
type, the whole of the checked arithmetic, `Ratio`, `RatInterval`, the tier grid,
the binary codec and every Rule O guarantee remain.

Above `ucal-core`, every crate needs `alloc` — `ucal-civil`, `ucal-body`,
`ucal-events` and `ucal-cosmo` all return `String` or `Vec` in their public
surface, and all four build `no_std` + `alloc` for wasm cleanly.

One consequence worth stating plainly: the `bigint` backend implies `alloc`,
because a heap-backed integer cannot be built without one. **The no-alloc
configuration is a `u512` configuration by construction.**

### GE-6, measured

GE-6 asked whether deriving the datum offset from `z = 1089.9 ± 0.4` through
`ucal-cosmo` would give "a shorter, fully tick-native chain", with the kill
criterion: *if the resulting enclosure is wider than the current published age
uncertainty, the route adds auditability without adding rigour; leave D-21
standing.*

| route | age | width |
|---|---|---|
| published (`BIG_BANG_CLAIM`) | 13.787 Gyr ± 0.020 | 0.040 Gyr |
| derived, `t_of_z(0)` at depth 12 | 13.590…13.987 Gyr | **0.397 Gyr** |

Ten times wider, and depth will not fix it: 18.9 Myr of that width is
quadrature and 378 Myr is the parameters. Integrating to infinite depth would
still leave an enclosure nine times the published one.

The reason is structural rather than numerical. The published age is a *fit* —
Planck's pipeline constrains `H0`, `Ω_m` and the age jointly against the whole
data set, so the age carries less uncertainty than propagating the marginal
parameter ranges through the integral suggests. Re-deriving it from the
marginals discards the correlations that made it tight, and interval arithmetic
is rigorous precisely because it refuses to assume any.

**The kill criterion fires. D-21 stands**, and the datum keeps its single cited
scalar. The derivation remains available — `ucal cosmo age --z 0` prints it —
as a cross-check that the declared datum lies inside what the model implies,
which it does.

---

## D-A16 — §4.3's SI equivalent is printed on request, not always  (AMENDMENT)

**§4.3 says:** "`ucal explain` always prints the SI equivalent alongside."

**Amended to:** a foreign-unit conversion is printed when the caller asks for
one, with `--bridge`. It is not printed unasked, in `explain` or anywhere else.

### Why

§4.3's sentence is a concession, and the paragraph around it says so: "Nothing
on the ladder is near a second or an hour. That is the accepted cost of leaving
the Earth paradigm (D-2 rationale)." The concession was made so a reader would
have some purchase on an unfamiliar ladder.

Implementing 0.4.0 showed what the concession had grown into. `ucal cosmo age`
reported its three enclosure widths in Julian years **and nothing else** — a
Julian year being 365.25 of Earth's rotations, used as the sole measure of an
epoch some 13.4 billion years before Earth existed. §4.3 permits the bridge
*alongside*; that was the bridge *instead*, in the one program written to object
to precisely that substitution.

The narrow fix was to add tick and drift columns beside the years, and that was
done. But the general form of the defect is that a foreign unit appeared without
being asked for, and could therefore become the only unit again anywhere a
future field forgot its body-independent twin.

### What changes

`--bridge` is the request. Without it, output uses ticks and the tier ladder,
which are body-independent by construction. With it, the SI second, the Julian
year and Gyr appear as before.

Two contexts keep foreign units unconditionally, and the list is short on
purpose:

- **`ucal to-civil` and `ucal from-civil`.** A civil label *is* an Earth label
  and rendering one is the entire request. Gating it would gate the command.
- **`ucal datum`'s provenance chain and rounding residual.** §19.2 requires
  them, and they record where an Earth-sourced measurement entered — the point
  being that it entered there and nowhere else (Rule Y). Hiding the audit trail
  to avoid naming a second would defeat what the trail is for.

### Enforcement

`Value::Bridge` makes "this is a foreign unit" a property of the value rather
than a bool threaded through five signatures, so omitting it is the default
rather than something a call site must remember.
`crates/ucal/tests/no_earth_units.rs` asserts that no non-Earth command prints
a foreign unit by default, that `--bridge` brings them back, and that the Earth
commands above keep theirs.

---

## D-A17 — `UCAL-E0014`, a name that is not found

**Status: AMENDMENT.** Appendix E gains one code.

### What the RFC says

Rule N: *"A name collision within an active table is `UCAL-E0011`."* Appendix E
lists `UCAL-E0011` as *duplicate name in active locale table (Rule N)*.

### What was wrong

Rule N and Appendix E define a code for a **collision** — two entries claiming
one name — and no code for a **miss**, which is the far commoner event: a person
types a tier name that does not exist.

This implementation returned `E0011` for both. The diagnostic read

```
UCAL-E0014: name not found in the active locale table (unknown tier name; …)
```

before the amendment as

```
UCAL-E0011: duplicate name in the active locale table (unknown tier name; …)
```

— a code whose canonical meaning is the opposite of the context beside it. A
reader looking up `E0011` in Appendix E would have found a description of a
condition that had not occurred, and a conforming implementation reproducing
this behaviour would have propagated the error rather than the diagnosis.

### The amendment

| code | meaning |
|---|---|
| `UCAL-E0014` | name not found in the active locale table (Rule N) |

`UCAL-E0011` keeps Rule N's meaning exactly: a collision, and only a collision.
`UCAL-E0014` takes exit code 6 — data/config error — as the rest of the
`E001x` family does.

### Why it is an amendment and not an erratum

Nothing in the RFC was self-contradictory: it simply did not name this
condition. An implementation had to either invent a code or misuse one, and
misusing one is worse, because a misused code looks correct in every place a
reader might check it.

### Enforcement

`Code` is `#[non_exhaustive]`, so the addition breaks no exhaustive match. The
CLI's hostile-input corpus asserts that `ucal between 0 100 --at nope` exits with
a §19.5 code and a message; `crates/ucal-core/src/locale.rs` is the only site
that raises it.

---

## D-A18 — `UCAL-E0015`, a build that does not reproduce its own constants

**Status: AMENDMENT.** Appendix E gains one code.

### What was missing

§3.3 requires a conforming implementation's declared constants to be
reproducible, and `ucal verify` re-derives them to check that this binary does.
Appendix E named no code for the answer being *no*.

### Why that mattered more than it looks

The first implementation of the check reported the disagreement in a note and
**exited 0**. A verification command whose failure a caller cannot detect is
worse than no command, because it is read as a passing check — and the release
workflow that packages prebuilt binaries relies on exactly this exit status to
refuse to ship a binary that fails it.

The second attempt fixed the exit code by borrowing `UCAL-E0025`, which carries
§19.5's exit 9 and means *"BIG_BANG_CLAIM used as a computational operand"* —
a code chosen for its number, describing something that had not happened. That
is the defect [D-A17](#d-a17--ucal-e0014-a-name-that-is-not-found) was written
to remove one cycle earlier, reintroduced within the hour.

### The amendment

| code | meaning | exit |
|---|---|---|
| `UCAL-E0015` | this build does not reproduce the declared constants (§3.3) | 9 |

Raised by `ucal verify` and by nothing else. Exit 9 is §19.5's *internal
invariant violation*, which is what a binary disagreeing with its own declared
constants is: not a user error, and not a difference of opinion, since every
quantity involved is an exact integer.

### Enforcement

`Code` is `#[non_exhaustive]` and the variant is **appended**, so no
discriminant shifts and no exhaustive match breaks — `cargo semver-checks`
verifies this. The release workflow runs `ucal verify` on every artefact before
packaging it, so the code has a caller that acts on it rather than only a
definition.
