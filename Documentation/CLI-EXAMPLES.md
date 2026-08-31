# Worked examples

**Generated — do not edit.** Every block below is the real output of the command above it, captured by running it. Regenerate with:

```
cargo build --release -p ucal --features full
cargo run -p xtask -- gen-examples
```

`cargo run -p xtask -- check-docs` fails if this file is not what a fresh run produces, so an example cannot describe output the program does not produce. Field-by-field documentation is in [`CLI.md`](CLI.md).

`ucal now` and `ucal tour` are absent on purpose: `now` reads the system clock, so its output differs on every run and could never match a committed file.

Colour is off and the width is 80 columns, so these match a redirected run rather than a terminal.

---

## `ucal datum`

What tick 0 is, and — more of the output than a reader expects — what is *not* being claimed about it.

```
Profile UC-1 — the datum
────────────────────────
datum             tick 0 is a stipulated reference point, conventionally
                  identified with the FLRW t→0 limit
frame             FLRW comoving (cosmological time, CMB rest frame)
tick_zero         0
frame_bridge_claim:
  bridge_scale      TT (§8.1)
  half_width_ticks  40351020014477982581316000000000000000000000000000000000
  bound             5 x 10^-6 of elapsed time: the rate difference between this
                    profile's declared frame and its bridge scale
  citation          Planck 2018 results I: Overview, and the cosmological
                    legacy of Planck, A&A 641, A1 (2020) -- solar dipole 369.82
                    +/- 0.11 km/s
  cancels_in        any difference of two instants carried through the same
                    bridge, which is every interval this program computes. It
                    bears only on reading an absolute tick count as elapsed
                    cosmological time
  status            metadata only; no arithmetic operation may consume it, for
                    the same reason big_bang_claim may not (Rule Q.3)
big_bang_claim:
  window             [-117069761411410698720000000000000000000000000000000000000
                     00, 1170697614114106987200000000000000000000000000000000000
                     0000] ticks
  half_width_ticks   11706976141141069872000000000000000000000000000000000000000
  half_width_drifts  141.53
  citation           Planck 2018 results VI: Cosmological parameters, A&A 641,
                     A6 (2020)
  locator            doi:10.1051/0004-6361/201833910
  status             metadata only; no arithmetic operation may consume it
                     (Rule Q.3)
datum_provenance:
  input     13.787 Gyr ± 0.020 Gyr (age_of_universe)
  citation  Planck 2018 results VI: Cosmological parameters, A&A 641, A6 (2020)
  unit_defs:
    Gyr = 10^9 x 31 557 600 s (Julian years, exact by definition)
  chain:
    AGE_s = 13 787 000 000 x 31 557 600 = 435 084 631 200 000 000 s (exact)
    AGE_ticks = AGE_s x SECOND = 80702040028955965162632000000000000000000000000
    00000000000000 (exact)
    beats = round_half_even(AGE_ticks / BEAT) = 9 304 311 741 502 590 385
    ORIGIN_OFFSET = beats x BEAT = 807020400289559651594434308563563718053046613
    9316558837890625
rounding:
  to                 BEAT
  mode               half_even
  residual_ticks     -318856914364362819469533860683441162109375
  residual_rendered  -0.017190364 s
  rationale          a whole-beat datum makes all sub-beat digits of the bridge
                     epoch zero (§2.4)
earth_dependency  The input arrives in Julian years and the bridge anchor is an
                  Earth calendar date. Both are metrology (Rule Y). Neither
                  appears in any computation: ORIGIN_OFFSET is a declared
                  integer of ticks.
implied_age:
  note  a consequence of the declared datum, not a measurement (Rule Q.1). The
        measurement is the `input` above.
certification:
  rounded, half-even, 2 digits  half_width_drifts
  every other number above is exact: the digits shown are the value

Changing the datum, BIG_BANG_CLAIM or datum_provenance produces a new profile;
Rule P then keeps values from the two from mixing.
```

## `ucal explain 8070205189123984864657505252035637180530466139316558837890625`

One instant in every form the program has. The 61-digit integer is the value; everything under it is a rendering of that same number.

```
ucal explain
────────────
ticks      8070205189123984864657505252035637180530466139316558837890625
precision  tick (exact)
human      UC1 0031·0687·2481·2999·3108·2437:1104·2790·0251·2597·0804·1100·0000·
           0000·0000·0000·0000·0000
digit5     UC1/5 00000.00000.00000.00000.00000.00000.00000.00000.00000.00000.
           00000.00000.00000.00000.00000.00000.00000.00000.00000.00000.00000.
           00000.00000.00000.00000.00000.00000.00111.10222.34411.43444.44413.
           34222.13404.42130.02001.40342.11204.13400.00000.00000.00000.00000.
           00000.00000
ucid       0000000000050PM6K45HH4YGQJ6SEDGDDZ1NKFHD32F2XBM29FJ1
tiers:
  T5 deep   31
  T4 drift  687
  T3 span   2481
  T2 sweep  2999
  T1 arc    3108
  T0 beat   2437
beats_since_datum:
  whole            9304313109130808687
  remainder_ticks  306669363733110525505617260932922363281250
  note             the beat is the universe second (§0.5), 5^60 ticks; this
                   count carries no Earth content
```

## `ucal explain 8070205189123984864657505252035637180530466139316558837890625 --why`

The same document with each field annotated by the rule or section that requires it. Almost nothing here is a convenience.

```
ucal explain
────────────
ticks      8070205189123984864657505252035637180530466139316558837890625
precision  tick (exact)
human      UC1 0031·0687·2481·2999·3108·2437:1104·2790·0251·2597·0804·1100·0000·
           0000·0000·0000·0000·0000
digit5     UC1/5 00000.00000.00000.00000.00000.00000.00000.00000.00000.00000.
           00000.00000.00000.00000.00000.00000.00000.00000.00000.00000.00000.
           00000.00000.00000.00000.00000.00000.00111.10222.34411.43444.44413.
           34222.13404.42130.02001.40342.11204.13400.00000.00000.00000.00000.
           00000.00000
ucid       0000000000050PM6K45HH4YGQJ6SEDGDDZ1NKFHD32F2XBM29FJ1
tiers:
  T5 deep   31
  T4 drift  687
  T3 span   2481
  T2 sweep  2999
  T1 arc    3108
  T0 beat   2437
beats_since_datum:
  whole            9304313109130808687
  remainder_ticks  306669363733110525505617260932922363281250
  note             the beat is the universe second (§0.5), 5^60 ticks; this
                   count carries no Earth content
why:
  ticks  Rule Z: the value itself, an unsigned integer count from the datum.
         Everything else on this page is a rendering of this number.
  precision  Rule T: a form printed to a coarser tier denotes an interval, not
             a point with trailing zeros. This says which one you are holding.
  human  §6: the text form anchored at T0, for reading aloud.
  digit5  §6 and Rule S: fixed-width, so lexicographic order equals
          chronological order. That is why it opens with 27 groups of zeros.
  ucid  §6.5: a sortable identifier for an instant, or a statement that this
        one is outside the 2^256 UCID range.
  tiers  §4.2: the instant decomposed onto the universal ladder. It reassembles
         to `ticks` exactly, because every tier is a power of five.
  beats_since_datum  §0.5: the beat is the universe second, 5^60 ticks. This
                     count carries no Earth content, which is the point of
                     having it.
  si_bridge  Rule A.5 and D-A16: an SI second is an Earth unit, so the
             conversion is shown on request with --bridge and never unasked.

Each line names the rule or section that requires the field above it. Almost
none is a convenience: this command's output is dense because the model is, not
because more seemed better.
```

## `ucal between 0 8070205189123984864657505252035637180530466139316558837890625 --at beat`

A duration on the tier ladder. `--at` adds the whole count and remainder at one named tier.

```
ucal between
────────────
from          0
to            8070205189123984864657505252035637180530466139316558837890625
direction     `to` is later than `from`
ticks         8070205189123984864657505252035637180530466139316558837890625
natural_tier  T5 deep
on_the_ladder:
  tier         whole
  ───────────  ────────────────────────────────
  T5 deep      31
  T4 drift     687
  T3 span      2481
  T2 sweep     2999
  T1 arc       3108
  T0 beat      2437
  T-1 flicker  1104
  T-2 glint    2790
  T-3 spark    251
  T-12 tick    23621918377466499805450439453125
at:
  unit             T0
  tier             T0 beat
  whole            9304313109130808687
  remainder_ticks  306669363733110525505617260932922363281250
```

## `ucal --bridge between 0 8070205189123984864657505252035637180530466139316558837890625`

The same difference with the SI conversion asked for. Without `--bridge` no Earth unit appears, which is the project's whole argument in one flag.

```
ucal between
────────────
from          0
to            8070205189123984864657505252035637180530466139316558837890625
direction     `to` is later than `from`
ticks         8070205189123984864657505252035637180530466139316558837890625
natural_tier  T5 deep
on_the_ladder:
  tier         whole
  ───────────  ────────────────────────────────
  T5 deep      31
  T4 drift     687
  T3 span      2481
  T2 sweep     2999
  T1 arc       3108
  T0 beat      2437
  T-1 flicker  1104
  T-2 glint    2790
  T-3 spark    251
  T-12 tick    23621918377466499805450439453125
si_bridge:
  unit     second
  seconds  435084695152502399.982810
certification:
  rounded, half-even, 6 digits  seconds
  every other number above is exact: the digits shown are the value
```

## `ucal ladder --named-only`

The ten named rungs of the 45-tier grid. A tier's identity is its exponent; the names are display-only.

```
ucal ladder — locale en
───────────────────────
locale  en
note    the universal ladder (§4.2): body-independent, and the canonical way to
        state any duration. Names are display-only (Rule N); the canonical
        identity of a tier is its exponent.
tiers:
  tier  exponent  name                beats
  ────  ────────  ──────────────────  ─────────────────────────
  T5    85        deep / deeps        298023223876953125.000000
        ticks  258493941422821148397315216271863391739316284656524658203125
  T4    80        drift / drifts      95367431640625.000000
        ticks  82718061255302767487140869206996285356581211090087890625
  T3    75        span / spans        30517578125.000000
        ticks  26469779601696885595885078146238811314105987548828125
  T2    70        sweep / sweeps      9765625.000000
        ticks  8470329472543003390683225006796419620513916015625
  T1    65        arc / arcs          3125.000000
        ticks  2710505431213761085018632002174854278564453125
  T0    60        beat / beats        1.000000
        ticks  867361737988403547205962240695953369140625
  T-1   55        flicker / flickers  0.000320
        ticks  277555756156289135105907917022705078125
  T-2   50        glint / glints      0.000000
        ticks  88817841970012523233890533447265625
  T-3   45        spark / sparks      0.000000
        ticks  28421709430404007434844970703125
  T-12  0         tick / ticks        0.000000
        ticks  1
certification:
  rounded, half-even, 6 digits  beats
  every other number above is exact: the digits shown are the value

The beat is the universe second (§0.5): 5^60 ticks, a pure power of the tick
with no Earth content. The bridge second is a declared foreign unit (Rule A.3)
and is shown only alongside.

The two seconds are incommensurable above T-6: one bridge second is
21.385061835 beats, not a whole number, because BEAT carries 5^60 while SECOND
carries only 5^30. They share a common measure only at the tick, which is why
Rule A.1 makes the tick primitive.
```

## `ucal cal list`

Seven derived calendars, two with anchors. A body without an anchor is the ordinary case, not a failure.

```
ucal cal list
─────────────
calendars:
  calendar      kind                             body       anchor_revision
  ────────────  ───────────────────────────────  ─────────  ───────────────
  earth-d       derived — Rule K                 earth      1
                leap_rule  31/128 (convergent 4)
                cycles  from moon
  mars-d        derived — Rule K                 mars       1
                leap_rule  45/76 (convergent 6)
                cycles  none — the calendar names no grouping satellite
  titan-d       derived — Rule K                 titan      —
                leap_rule  88/117 (convergent 3)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  luna-d        derived — Rule K                 luna       —
                leap_rule  31/84 (convergent 5)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  mercury-d     derived — Rule K                 mercury    —
                leap_rule  1/2 (convergent 1)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  venus-d       derived — Rule K                 venus      —
                leap_rule  135/146 (convergent 5)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  jupiter-d     derived — Rule K                 jupiter    —
                leap_rule  68/81 (convergent 4)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  io-d          derived — Rule K                 io         —
                leap_rule  58/59 (convergent 2)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  europa-d      derived — Rule K                 europa     —
                leap_rule  1/24 (convergent 2)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  ganymede-d    derived — Rule K                 ganymede   —
                leap_rule  149/261 (convergent 4)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  callisto-d    derived — Rule K                 callisto   —
                leap_rule  17/28 (convergent 6)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  enceladus-d   derived — Rule K                 enceladus  —
                leap_rule  28/151 (convergent 5)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  uranus-d      derived — Rule K                 uranus     —
                leap_rule  42/85 (convergent 2)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  neptune-d     derived — Rule K                 neptune    —
                leap_rule  7/179 (convergent 4)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  pluto-d       derived — Rule K                 pluto      —
                leap_rule  53/149 (convergent 5)
                status  no anchor: complete in units, intercalation and cycles,
                        incomplete in phase. Asking for local fields is
                        UCAL-E0062 (Rule J.3).
  earth-civil   legacy — declared tables (§8.6)  —          —
                leap_rule  97/400 (NOT a convergent — declared, not derived)
                arbitrary  4
  earth-julian  legacy — declared tables (§8.6)  —          —
                leap_rule  1/4 (a convergent)
                arbitrary  4

A derived calendar is a consequence of a body's periods (Rule K). A legacy one
is a declared table preserved for interoperation (§8.6) and is outside that
mechanism.
```

## `ucal cal show earth-d 8070205189123984864657505252035637180530466139316558837890625`

One derived calendar in full: its intercalation, its cycles, and the cited anchor that gives it a phase.

```
ucal cal show earth-d
─────────────────────
calendar  earth-d
kind      derived — Rule K
body      earth
ladder_placement:
  unit       rung      above_rung
  ─────────  ────────  ──────────
  solar_day  T1 arc    591.3
  year       T2 sweep  69.1
  cycle      T2 sweep  5.6
anchor:
  phase         mean solar midnight
  revision      1
  method        mean solar midnight at the prime meridian on 2000-01-01, i.e.
                00:00:00 UT1, converted through TT = UT1 + Delta-T with
                Delta-T(2000.0) = 63.8285 s
  uncertainty   dominated by the resolution of the published Delta-T series,
                which is quoted to 0.0001 s near 2000.0; the window is widened
                to 1 ms to cover the series' own stated scatter
  window_ticks  37097168799722000000000000000000000000000
  citation      IERS Conventions (2010) and the IERS Earth Orientation Centre's
                published Delta-T series; iers.org no longer answers — the
                original document is available in the Internet Archive
intercalation:
  whole_days_per_year  365
  rule                 31/128
  bound                1 local day per 10000 local years
  walked:
    1: 1/4 — 1 day slips in 128 local years
    2: 7/29 — 1 day slips in 1234 local years
    3: 8/33 — 1 day slips in 4269 local years
    4: 31/128 — 1 day slips in 400000 local years   <- chosen
    5: 752/3105 — 1 day slips in 62100000 local years
    6: 4543/18758 — 1 day slips in 937900000 local years
    7: 9838/40621 — 1 day slips in 4062100000 local years
    8: 24219/100000 — 1 day slips in never (exact) local years
fields:
  year             27
  day              210
  day_fraction     0.999261
  anchor_revision  1
  window_ticks     37097168799722000000000000000000000000000
cycles:
  satellite        moon
  cycles_per_year  12.368266523
  convergents:
    12/1
    25/2
    37/3
    99/8
    136/11
    235/19
    4131/334
    8497/687
certification:
  rounded, half-even, 1 digits  above_rung
  rounded, trunc, 6 digits      day_fraction
  rounded, half-even, 9 digits  cycles_per_year
  every other number above is exact: the digits shown are the value
```

## `ucal show 8070205189123984864657505252035637180530466139316558837890625`

One instant in several calendars at once, each labelled derived or legacy.

```
ucal show
─────────
ticks  8070205189123984864657505252035637180530466139316558837890625
human  UC1 0031·0687·2481·2999·3108·2437:1104·2790·0251·2597·0804·1100·0000·
       0000·0000·0000·0000·0000
calendars:
  calendar     rendered                             kind
  ───────────  ───────────────────────────────────  ────────────────
  earth-d      earth-d/1: 0027-210.9992 c328        derived (Rule K)
               anchor_revision  1
               window_ticks  37097168799722000000000000000000000000000
               day_is_ambiguous  false
  mars-d       mars-d/1: 0082-083.4420              derived (Rule K)
               anchor_revision  1
               window_ticks  37097168799722000000000000000000000000000000
               day_is_ambiguous  false
  earth-civil  earth-civil: 2026-07-29T00:00:00 TT  legacy (§8.6)
               warning  UCAL-W0005: value produced by a legacy (non-derived)
                        calendar
               note  declared tables; not a Rule K derivation

One instant, several local calendars. Each derived rendering carries its anchor
revision (Rule J.5) and the width of the window that revision implies (Rule
J.2); each legacy one is labelled as declared table data (§8.6).
```

## `ucal add 8070205189123984864657505252035637180530466139316558837890625 1 --step mars-d-year`

The operation this program did not have: it could read time and measure it, and not move through it. Moving below the datum is an error rather than a negative instant, because absolute time is unsigned (Rule B).

```
ucal add
────────
ticks         8070205190224925567986409801898677180530466139316558837890625
human         UC1 0031·0687·2482·0004·3034·0649:1620·0455·2719·2556·1950·2134·
              3000·0000·0000·0000·0000·0000
ucid          0000000000050PM6K4X2TJ6SC83N4QSJZQ5BG2NG77AJ1VM29FJ1
from          8070205189123984864657505252035637180530466139316558837890625
moved_by      1 x one year of `mars`
offset_ticks  1100940703328904549863040000000000000000000000000000

Exact: a whole number of a unit that is itself a whole number of ticks, so
nothing is rounded. Moving below the datum is `UCAL-E0020` rather than a
negative instant, because absolute time is unsigned (Rule B); moving past the
ceiling is `UCAL-E0021`. Neither wraps and neither saturates (Rule O).
```

## `ucal add 8070205189123984864657505252035637180530466139316558837890625 1 --in mars-d`

The same instant, one Martian year later as the calendar counts them — same day of the year, same position within it. `--step mars-d-year` above adds the mean orbital period instead, which is a duration and lands on a different local date: a local year is not a constant span, because the leap rule makes the lengths differ by one.

```
ucal add
────────
ticks     8070205190225597130539563071720833180530466139316558837890625
human     UC1 0031·0687·2482·0005·0156·3033:1336·1423·1507·3059·0601·0562·1575·
          0000·0000·0000·0000·0000
ucid      0000000000050PM6K4X398QQWA8RMJM2XM2JBM466ZF1WZ429FJ1
from      8070205189123984864657505252035637180530466139316558837890625
calendar  mars-d
moved_by  1 x local year of `mars-d` = 669 local days
local:
  from             0082-083
  to               0083-083
  day_fraction     0.442043
  days_moved       669
  direction        forwards
  anchor_revision  1
certification:
  rounded, trunc, 6 digits  day_fraction
  every other number above is exact: the digits shown are the value

The day of the year and the position within it are carried across unchanged,
not recomputed — so nothing here is rounded and the answer is an instant rather
than a window. `--step <id>-year` adds the body's mean orbital period instead,
which is a duration: it is exact too, and it lands on a different local date.
```

## `ucal between 0 8070205189123984864657505252035637180530466139316558837890625 --at mars-d`

How many Martian solar days since the datum, whole and remainder. `--at` has always meant express this span in this unit; since 1.11.0 the unit can be a calendar's own and not only a tier.

```
ucal between
────────────
from          0
to            8070205189123984864657505252035637180530466139316558837890625
direction     `to` is later than `from`
ticks         8070205189123984864657505252035637180530466139316558837890625
natural_tier  T5 deep
on_the_ladder:
  tier         whole
  ───────────  ────────────────────────────────
  T5 deep      31
  T4 drift     687
  T3 span      2481
  T2 sweep     2999
  T1 arc       3108
  T0 beat      2437
  T-1 flicker  1104
  T-2 glint    2790
  T-3 spark    251
  T-12 tick    23621918377466499805450439453125
at:
  unit             one solar day of `mars`
  whole            4900968733496
  remainder_ticks  445666073352520285973180530466139316558837890625
```

## `ucal cal from mars-d 82-83`

The inverse of `cal show`, and the thing the fifteen derived calendars did not have while Earth's legacy ones had it from 0.1.0. The answer is a window and would be wrong not to be: a local day is a span, and the anchor's uncertainty propagates into it (Rule J.2).

```
ucal cal from
─────────────
calendar         mars-d
local            82-83
window:
  lo           8070205189123256952903759537926801463523316507316558837890625
  hi           8070205189124903645106880591489885463523316507316558837890625
  width_ticks  1646692203121053563084000000000000000000000000000
lo_human         UC1 0031·0687·2481·2999·2840·0712:0747·1720·1796·0973·0964·
                 0679·1235·1299·2374·0000·0000·0000
anchor_revision  1

**A local date is an interval, not an instant**, and for two reasons that would
each be enough on their own. A local day is a span — this window is that whole
day unless a fraction was given. And the anchor carries uncertainty, which
propagates (Rule J.2), so it widens the answer at both ends. The endpoints are
taken outward to tick boundaries, never inward: narrowing would be narrowing by
assumption.
```

## `ucal cal validate --all`

The precision probe over every calendar this project ships. Fifteen calendars rest on nineteen distinct published figures and fourteen of those decide their leap rule outright — which is Rule K working, not a defect. The sharp part is the sharing: a satellite's year is its primary's orbit, so Jupiter's 4332.589 d decides five leap rules at once.

```
ucal cal validate --all
───────────────────────
calendars  15
figures:
  parameters_probed   24
  parameters_derived  6
  distinct_figures    19
  distinct_sensitive  14
  distinct_stable     5
intercalation:
  calendar     rule     solar_day
  ───────────  ───────  ────────────────────────────────
  earth-d      31/128   86400 s decides it
               orbital_period  31556925.216 s has a digit to spare
  mars-d       45/76    88775.244 s has a digit to spare
               orbital_period  686.9726 d (86400 s) decides it
  titan-d      88/117   derived — no last digit to move
               orbital_period  10759.2058 d (86400 s) decides it
  luna-d       31/84    29.53 d (86400 s) decides it
               orbital_period  365.256 d (86400 s) decides it
  mercury-d    1/2      15201360 s has a digit to spare
               orbital_period  87.969 d (86400 s) has a digit to spare
  venus-d      135/146  10087200 s has a digit to spare
               orbital_period  224.701 d (86400 s) decides it
  jupiter-d    68/81    35733.24 s decides it
               orbital_period  4332.589 d (86400 s) decides it
  io-d         58/59    derived — no last digit to move
               orbital_period  4332.589 d (86400 s) decides it
  europa-d     1/24     derived — no last digit to move
               orbital_period  4332.589 d (86400 s) decides it
  ganymede-d   149/261  derived — no last digit to move
               orbital_period  4332.589 d (86400 s) decides it
  callisto-d   17/28    derived — no last digit to move
               orbital_period  4332.589 d (86400 s) decides it
  enceladus-d  28/151   derived — no last digit to move
               orbital_period  10759.2058 d (86400 s) decides it
  uranus-d     42/85    17.24 h (3600 s) decides it
               orbital_period  30685.4 d (86400 s) decides it
  neptune-d    7/179    16.11 h (3600 s) decides it
               orbital_period  60189.0 d (86400 s) decides it
  pluto-d      53/149   153.2820 h (3600 s) decides it
               orbital_period  90560 d (86400 s) decides it
cycles:
  calendar     grouped_by
  ───────────  ──────────
  earth-d      moon
               terms_surviving  7
  mars-d       —
               terms_surviving  no grouping satellite declared, though the body
                                has one
  titan-d      —
               terms_surviving  no satellite at all
  luna-d       —
               terms_surviving  no satellite at all
  mercury-d    —
               terms_surviving  no satellite at all
  venus-d      —
               terms_surviving  no satellite at all
  jupiter-d    —
               terms_surviving  no satellite at all
  io-d         —
               terms_surviving  no satellite at all
  europa-d     —
               terms_surviving  no satellite at all
  ganymede-d   —
               terms_surviving  no satellite at all
  callisto-d   —
               terms_surviving  no satellite at all
  enceladus-d  —
               terms_surviving  no satellite at all
  uranus-d     —
               terms_surviving  no satellite at all
  neptune-d    —
               terms_surviving  no satellite at all
  pluto-d      —
               terms_surviving  no satellite at all
carried_by_more_than_one:
  figure
  ──────────────────────
  4332.589 d (86400 s)
                          calendars resting on it  5 calendars: jupiter-d,
                                                   io-d, europa-d, ganymede-d,
                                                   callisto-d
  10759.2058 d (86400 s)
                          calendars resting on it  2 calendars: titan-d,
                                                   enceladus-d

**Sensitive is a measurement, not a verdict.** A leap rule is a convergent of a
continued fraction, and continued fractions are violently sensitive to their
inputs — so most published figures deciding their own rule is Rule K working.
What it means for an author is that quoting a source to one digit fewer
declares a different calendar.

**The probe is not comparable between bodies.** One unit in the last place is a
second for Earth's 86400 s and a millisecond for Mars's 88775.244 s, so
`sensitive` means `at the precision this source published`, not `to the same
tolerance`. Earth's solar day is exact by definition and has no last digit to
be wrong in; it is reported sensitive because a second either way does change
the rule.

**A satellite's year is its primary's orbit.** So the figures above are shared,
and a revision to one moves every calendar under it — which is a thing to know
before revising one.

**Fourteen of the fifteen have no cycle**, because they declare no grouping
satellite. §15.3 forbids a fallback structure, so that is the answer and not a
gap — and it is listed per calendar rather than summarised, because a section
reporting one of fifteen without saying so is the shape V1 Finding 1 caught
fourteen times in this tree.

**`cycles` counts terms, not rules.** A leap rule is chosen by a drift bound,
so `which rule` is a decision that can survive a nudge; nothing selects a
cycle, and the deepest convergent is the ratio itself — which any nudge
changes. What carries information is how far the continued fraction agrees,
because terms that agree are candidate cycle rules that agree.
```

## `ucal cal validate Documentation/examples/europa.hjson`

Two questions with separate answers: does the file load, and does a calendar follow from it? The `precision:` rows measure the caveat this project has carried since 0.2.0 — a rounded parameter is a different calendar — by moving each published figure's last digit and re-deriving.

```
ucal cal validate
─────────────────
source  Documentation/examples/europa.hjson
kind    body file (§15.1)
checks:
  loads            ok — strict HJSON, every key known (§15.1), and every
                   parameter carries a value, a unit, an epoch, a validity
                   window and a citation (Rule C)
  id               `europa` — which derives `europa-d`, and a calendar of that
                   id already ships. This file is valid; a command naming
                   `europa-d` will get the compiled-in one
  primary          `jupiter`
  rotation_period  measured: 3.551181 d (86400 s) — NASA Planetary Fact Sheets
                   (Williams, D. R.), Jovian satellite fact sheet; the original
                   document is available in the Internet Archive
  solar_day        derived (Z1.1), so no figure of its own is stated for it —
                   derived as 1 / (1/P_rotation - 1/P_orbital_period) from the
                   two published figures cited in this file; no source
                   publishes a solar day for a tidally locked moon
  orbital_period   measured: 4332.589 d (86400 s) — NASA Planetary Fact Sheets
                   (Williams, D. R.), Jupiter fact sheet; the original document
                   is available in the Internet Archive
  intercalation    1/24 at convergent 2, 1219 whole days per year
  obliquity        none declared for this body
  cycles           none — no grouping satellite, so this calendar has years and
                   days and no cycle. §15.3 forbids a fallback structure, which
                   makes that the answer and not a gap
  precision:
    orbital_period  sensitive — one unit in the last published place derives a
                    different rule. 4332.589 d (86400 s) gives 1/24; 4332.590 d
                    (86400 s) → 5/119; 4332.588 d (86400 s) → 7/169. This is
                    the standing caveat measured, not a verdict: a figure that
                    is exact by definition has no last digit to be wrong in,
                    and a figure that was rounded to reach this precision
                    declares a calendar the unrounded one would not

A file that loads is not a file that is right. Every check above is on
*internal* consistency — that the parameters are present, cited and mutually
coherent. Whether the published figures are the ones this body actually has is
a question about the sources, and nothing in this program can answer it.
```

## `ucal events show recombination`

A cited milestone. Events are intervals, and the citation travels with the value.

```
ucal events show recombination
──────────────────────────────
label         recombination
description   electrons and protons combine and the universe becomes
              transparent. A process, not an instant: it spans roughly z = 1400
              to z = 1000. Planck 2018 quotes last scattering at z_* = 1089.92,
              t_* = 372.6 kyr; the classic textbook figure of 380 kyr names the
              same era less precisely
as_published  240 to 430 kyr (z = 1400 to z = 1000)
stated_as     after the datum
window:
  lo           140483713693692838464000000000000000000000000000000000000
  hi           251699987034533002248000000000000000000000000000000000000
  width_ticks  111216273340840163784000000000000000000000000000000000000
midpoint:
  ticks     196091850364112920356000000000000000000000000000000000000
  at_drift  2 drift
  note      a midpoint is a rendering choice, not a measurement (Rule U): the
            window is the value
citation      Planck 2018 results VI: Cosmological parameters, A&A 641, A6
              (2020)
warning       UCAL-W0006: quantity comparable to or smaller than BIG_BANG_CLAIM

This event lies inside BIG_BANG_CLAIM's half-width. The datum's own physical
identification is uncertain by more than the interval being quoted — but the
arithmetic above is exact, and the claim is never an operand (Rule Q.3).
```

## `ucal timeline`

The catalogue against the tier ladder — the whole of time on one screen.

```
ucal timeline — at tier T4
──────────────────────────
tier  T4
events:
  event
  ──────────────────────────────
  inflationary epoch
                                  at  0 drift
                                  tiers_since_datum  0
                                  as_published  10^-36 to 10^-32 s
                                  warning  UCAL-W0006 — inside the claim
                                           half-width
  big bang nucleosynthesis
                                  at  0 drift
                                  tiers_since_datum  0
                                  as_published  about 10 s to 20 min
                                  warning  UCAL-W0006 — inside the claim
                                           half-width
  matter-radiation equality
                                  at  0 drift
                                  tiers_since_datum  0
                                  as_published  z_eq = 3387 +/- 21
                                  warning  UCAL-W0006 — inside the claim
                                           half-width
  recombination
                                  at  2 drift
                                  tiers_since_datum  2
                                  as_published  240 to 430 kyr (z = 1400 to z =
                                                1000)
                                  warning  UCAL-W0006 — inside the claim
                                           half-width
  first stars
                                  at  1769 drift
                                  tiers_since_datum  1769
                                  as_published  100 to 400 Myr
  reionization
                                  at  1 deep, 943 drift
                                  tiers_since_datum  4068
                                  as_published  150 Myr to 1 Gyr
  first galaxies
                                  at  1 deep, 1828 drift
                                  tiers_since_datum  4953
                                  as_published  400 Myr to 1 Gyr
  dark energy domination
                                  at  22 deep, 2014 drift
                                  tiers_since_datum  70764
                                  as_published  z = 0.3 to 0.6
  Solar System formation
                                  at  20 deep, 2730 drift
                                  tiers_since_datum  65230
                                  as_published  4567 to 4571 Ma ago
  Earth accretion
                                  at  20 deep, 2971 drift
                                  tiers_since_datum  65471
                                  as_published  4.50 to 4.57 Ga
  last universal common ancestor
                                  at  22 deep, 2983 drift
                                  tiers_since_datum  71733
                                  as_published  3500 to 3800 Ma ago
  the Great Oxidation Event
                                  at  25 deep, 2914 drift
                                  tiers_since_datum  81039
                                  as_published  2.22 to 2.45 Ga
  base of the Cambrian
                                  at  29 deep, 3124 drift
                                  tiers_since_datum  93749
                                  as_published  538.8 ± 0.1 Ma ago
  the Permian-Triassic boundary
                                  at  30 deep, 2030 drift
                                  tiers_since_datum  95780
                                  as_published  251.902 +/- 0.024 Ma
  Cretaceous-Palaeogene boundary
                                  at  31 deep, 220 drift
                                  tiers_since_datum  97095
                                  as_published  66.0 to 66.1 Ma ago
  hominin-chimpanzee divergence
                                  at  31 deep, 641 drift
                                  tiers_since_datum  97516
                                  as_published  6 to 7 Ma ago
  earliest Homo sapiens fossils
                                  at  31 deep, 685 drift
                                  tiers_since_datum  97560
                                  as_published  315 +/- 34 ka
  the base of the Holocene
                                  at  31 deep, 687 drift
                                  tiers_since_datum  97562
                                  as_published  11 700 yr b2k
  the bridge epoch
                                  at  31 deep, 687 drift
                                  tiers_since_datum  97562
                                  as_published  0000-01-01T00:00:00 TT
  the end of star formation
                                  at  398 T6, 1702 deep, 574 drift
                                  tiers_since_datum  3892038074
                                  as_published  10^14 to 10^15 yr
  proton decay, if it happens
                                  at  1244 T13, 2813 T12, 3120 T11, 1052 T10,
                                      2336 T9, 2898 T8, 2312 T7, 1445 T6, 583
                                      deep, 690 drift
                                  tiers_since_datum  353821996987590080283870685
                                                     56510065
                                  as_published  10^34 to 10^40 yr (lower bound
                                                only)
  the last black holes evaporate
                                  at  1 T31, 1739 T30, 407 T29, 1119 T28, 2157
                                      T27, 2798 T26, 2739 T25, 728 T24, 760
                                      T23, 1260 T22, 2356 T21, 1020 T20, 1641
                                      T19, 2345 T18, 137 T17, 1901 T16, 2209
                                      T15, 1207 T14, 0 drift
                                  tiers_since_datum  357359859597606383480325912
                                                     094839567619719168000000000
                                                     000000000000000000000000000
                                                     00000000000000
                                  as_published  about 10^100 yr

Positions are the windows' midpoints floored to the stated tier. The midpoint
is a rendering choice; the window is the value (Rule U).
```

## `ucal cosmo age --z 1100 --depth 8`

A certified enclosure by integer quadrature. Depth 8 keeps this example fast; `CLI.md` has the cost table.

```
ucal cosmo age
──────────────
z           1100.0000
model       flat-LambdaCDM/planck2018
enclosure:
  lo_ticks  209889172060393451412424770268882531914815504124647413069
  hi_ticks  218641720554215180748181288675295490457543553467574678882
  at_drift  2 drift
widths:
  arithmetic_ticks   2348105362960176843944945987747501400223840119453206045
  arithmetic_drifts  0.028387
  parameter_ticks    6404443130861552491811572418665457142504209223474059768
  parameter_drifts   0.077425
  note               Rule X: quadrature error and parameter uncertainty are
                     reported separately and never merged (F8). The second is
                     what the measurement does not know; the first is what this
                     program does not know. Each is given in ticks and in
                     drifts, both body-independent; `--bridge` adds the
                     foreign-unit conversion, which is not performed unasked.
quadrature:
  depth              8
  panels             256
  sqrt_scale_digits  12
parameters  flat-LambdaCDM/planck2018 [H0 = 67.66 +/- 0.42 km/s/Mpc; Omega_m =
            0.3111 +/- 0.0056; Omega_Lambda = 0.6889 +/- 0.0056; Omega_r =
            9.14e-5 (derived from Omega_r h^2 = 4.18e-5);] Planck 2018 results
            VI: Cosmological parameters, A&A 641, A6 (2020),
            TT,TE,EE+lowE+lensing+BAO
citation    Planck 2018 results VI: Cosmological parameters, A&A 641, A6
            (2020), TT,TE,EE+lowE+lensing+BAO
warning     UCAL-W0004: cosmology enclosure width exceeds one tick
warning     UCAL-W0006: quantity comparable to or smaller than BIG_BANG_CLAIM
certification:
  rounded, half-even, 6 digits  arithmetic_drifts, parameter_drifts
  every other number above is exact: the digits shown are the value

The enclosure is certified: the true age under this model provably lies inside
it. It is not a measurement of the universe — it is what this parameter set
implies, with the parameter set's own uncertainty carried through (Rule X).
```

## `ucal lighttime 1 --unit ly`

A light-year is defined as a Julian year times c, so its light-travel time is that year exactly — 31 557 600 s with no remainder. A light-year is a time unit wearing a distance's clothes. One astronomical unit of light-time, 499.004783836 s, is also the largest a barycentric correction can be, for any target and any date.

```
ucal lighttime
──────────────
distance          1 ly
exact             true
ticks             585348807057053493600000000000000000000000000000000
seconds           31557600.000000000
as_ratio_seconds  31557600/1

A light-year is *defined* as a Julian year times `c`, so its light-travel time
is a Julian year — 31 557 600 s exactly, with no remainder. A light-year is a
time unit wearing a distance's clothes, and the conversion is the identity.

`c` is exact by definition (the metre is defined from it) and the astronomical
unit has been exact since IAU 2012 Resolution B2. **One astronomical unit of
light-time, 499.004783836… s, is also the largest a barycentric correction can
be** — for any target and any date. The correction's *value* needs an
ephemeris; the bound needs nothing, and answers whether a measurement is
sensitive to it at all.
```

## `ucal lighttime 1 --unit pc`

And the unit that cannot convert exactly. A parsec is defined as 648000/pi astronomical units — an exact definition of an irrational number — so the answer is a bracket and no decimal for it is the value.

```
ucal lighttime
──────────────
distance  1 pc
exact     false
seconds:
  lo  102927125.054339001
  hi  102927125.054339001
certification:
  rounded, trunc, 9 digits  lo, hi
  every other number above is exact: the digits shown are the value

A parsec is defined as `648000/π` astronomical units — an exact definition of
an irrational number — so the answer is a bracket and no decimal for it is the
value. Two of the three units here convert exactly and this one cannot, for a
reason about the definition rather than about this program.

`c` is exact by definition (the metre is defined from it) and the astronomical
unit has been exact since IAU 2012 Resolution B2. **One astronomical unit of
light-time, 499.004783836… s, is also the largest a barycentric correction can
be** — for any target and any date. The correction's *value* needs an
ephemeris; the bound needs nothing, and answers whether a measurement is
sensitive to it at all.
```

## `ucal from-jd 2451545.0 --scale tcb`

Barycentric Coordinate Time, whose offset from TDB is a *defining* constant rather than a measurement — so that step is exact and the answer carries only TDB's own bound, not a tick more. TCB runs ahead of TDB by 0.489 s per Julian year, which reaches a minute inside two centuries.

```
ucal from-jd
────────────
ticks  8070205173569972754741832104732271481136685314812878837890625
human  UC1 0031·0687·2481·1163·2191·0517:2863·3020·1358·2092·1852·2443·2769·
       0031·2038·1293·0000·0000
ucid   0000000000050PM6JSRZ1J9ZF64AZ6N34WEJS3ZXZJGDSSX6VFJ1
input:
  jd     2451545.0
  jd     2451545/1
  scale  tcb
window:
  lo           8070205173569972754741832104732271481136685314812878837890625
  hi           8070205173569972754804897291691798881136685314812878837890625
  width_ticks  63065186959527400000000000000000000000000

`ticks` above is the low end. TDB differs from TT by a periodic series whose
evaluation needs floating point, which Rule E forbids in a shipped crate — so
the answer carries the series' bound of ±1.7 ms instead of a centre that would
look exact. Rule U: the window is the value.
```

## `ucal dilate --rs-over-r 0.0000042467 --show 18`

The Sun's surface, bracketed to eighteen places by integer square roots. `f64` computing the same redshift keeps about eight of its sixteen digits, because `1/sqrt(1-x) - 1` in the weak field subtracts two numbers that agree to six places. The interval is proved to contain the value rather than converged to it.

```
ucal dilate
───────────
rs_over_r  42467/10000000000
observer   static at r — gravitational dilation only, √(1 − r_s/r)
digits     40
proper_per_coordinate:
  lo  0.999997876647745687
  hi  0.999997876647745688
coordinate_per_proper:
  lo  1.000002123356762946
  hi  1.000002123356762947
redshift_z:
  lo  0.000002123356762946
  hi  0.000002123356762947
certification:
  rounded, trunc, 18 digits  lo
  rounded, ceil, 18 digits   hi
  every other number above is exact: the digits shown are the value

Certified, not iterated: the two ends use `isqrt_floor` and `isqrt_ceil`, so
the interval is **proved** to contain the value rather than converged to it.
The same standard `cosmo` holds its quadrature to.

Exactness earns its keep at the two ends, not in the middle. `z = 1/√(1−x) − 1`
in `f64` keeps about 8 of its 16 digits at the Sun's surface and about 1 just
outside a horizon, both to cancellation; a neutron star at r_s/r = 0.35 is
where a double does best. The solar and white-dwarf redshifts are measured
quantities, and they sit in the band where the float has already lost half its
digits.

This is the ratio between two clocks and not a claim that either is the one
`ucal` keeps. Tick 0 is the FLRW t→0 limit, so absolute time here is a
cosmological coordinate; giving UC-1 a stated frame is a 2.0 question, because
one unsigned integer per instant asserts there is one time.
```

## `ucal ephem at Documentation/examples/ephemeris.hjson --cycle 5000`

A prediction five thousand cycles past a fit that covers five hundred. The window is the answer — it has grown from 7.5 s at the epoch to 164 s here — and `UCAL-W0003` says the fit does not reach this far. The figures in that file are illustrative and cite nothing, which it states.

```
ucal ephem at
─────────────
id  example
prediction:
  cycle             5000
  centre_ticks      807020520386767765210871024949775188053046613931655883789062
                    4
  lo                807020520386767460398120255424865432871303364528914287047265
                    0
  hi                807020520386768070023621794474684943234789863334397480530860
                    0
  half_width_ticks  3048127507695249097551817432494027415967417974
  sigmas            1

The window is what the answer is. `half_width_ticks` is `k·√(σ_T₀² +
(E·σ_P)²)`, so a prediction far from the epoch is wider than one near it —
which is the quantity that decides whether an observation is worth scheduling,
and the one most tooling drops.

UCAL-W0003: body parameter evaluated outside its validity window. Rule C makes
a parameter valid over a stated interval and forbids silent extrapolation, so
this instant lies outside the window at least one of this body's figures was
published for. The fields above are computed from the values at the epoch — see
`ucal cal show <id>` for the windows themselves

Cycle 5000 is outside the range this fit covers (-500 .. 500). Rule C requires
the warning and forbids extrapolating silently; it does not forbid
extrapolating, because that is what a reader is going to do and the useful
thing is to say how far out they have gone.
```

## `ucal verify`

The binary re-deriving the constants it ships with, and saying plainly that agreeing with itself is not verification.

```
ucal verify
───────────
profile                       UC1
backend                       u512
agrees                        true
constants:
  BEAT:
    agrees   true
    value    867361737988403547205962240695953369140625
    derived  867361737988403547205962240695953369140625
    from     5^60, by repeated multiplication
  SECOND:
    agrees   true
    value    18548584399861000000000000000000000000000000
    derived  18548584399861000000000000000000000000000000
    from     18548584399861 x 10^30 (D-3)
  ORIGIN_OFFSET:
    agrees   true
    value    8070204002895596515944343085635637180530466139316558837890625
    derived  8070204002895596515944343085635637180530466139316558837890625
    from     round_half_even(AGE_ticks / BEAT) x BEAT, from the provenance input
invariants:
  origin_offset_is_whole_beats  true
  bridge_divisibility_is_exact  true
  tier_grid_is_five_powers      true
compare_with                  fixtures/vectors.json in the source repository,
                              whose digest is signed; spec/CONFORMANCE.md
                              describes the file and the key
what_this_does_not_establish  This is a self-check. Every number above was
                              computed by one implementation from one
                              specification, so agreement means this build's
                              arithmetic works and reproduces the published
                              values — not that the specification is right. An
                              independent implementation reproducing these
                              constants is the check that would mean that, and
                              it has never been done. See
                              Documentation/CONTACT.md.
```

## `ucal doctor`

Which profile, which backend, which features — the first thing to paste into a bug report.

```
ucal doctor
───────────
profile           UC1
frame             FLRW comoving (cosmological time, CMB rest frame)
backend           u512 (bnum, stack, const-constructible; Instant is Copy)
domain_max_ticks  13407807929942597099574024998205846127479365820592393377723561
                  44372176403007354697680187429816690342769003185818648605085375
                  3882811946569946433649006084095
domain_bits       512
features:
  u512
  std
  civil
  body
  events
  cosmo
  tui
clock:
  granularity        1 ns — this program reads the clock to nine decimal places
  granularity_ticks  18548584399861000000000000000000000
  finest_tier        T-2 — the finest rung a nanosecond can fill
  rendering_floor    T-12 — where `ucal now` renders by default, 10 rungs below
                     what the clock can fill. A rung is 5^5, so those digits
                     are the conversion's and not the instrument's
  accuracy           not measurable here. §8.4 makes operation offline, so
                     there is no reference to compare against, and a rate error
                     estimated from a short baseline reports quantisation as
                     drift. This is resolution
  in_a_difference    a constant offset cancels between two readings and a rate
                     error does not; quantisation bounds each reading and so
                     bounds their difference twice over. The frame term `ucal
                     datum` declares cancels too
datum_provenance:
  present  true
  note     present; absence would be UCAL-E0013 (Rule Q.4)
leap_seconds:
  table_version     IERS Bulletin C 70 (no leap second to 2026-06-30)
  entries           27
  complete_through  2026-06-30
  pre_1972          the 1961-1972 rubber-second era is modelled exactly; UTC
                    before 1961-01-01 is UCAL-E0041
  network           never; the table is bundled and offline (§8.4)
spec:
  rfc  UCAL-1 final draft, 2026-07-29
  deltas:
    D-A2: ORIGIN_OFFSET has 61 trailing base-5 zeros, not 62 (editorial)
    D-A3: Appendix B's seconds column is imprecise; the table is generated
    (editorial)
    D-A4: Appendix C's human forms are truncated at T-5, not tick-exact
    (correction)
    D-A5: grouping cycles are declared per body, not admitted by a global bound
    (amendment)
    D-A6: Earth body parameters are chosen to reproduce Appendix I (editorial)
    D-A7: full-width encode is 45 divmod steps, not 44 (correction)
    D-A8: precision is the last group's tier; forms are anchored per-form
    (amendment)
    D-A9: §6.6 needs a calendar-id grammar to disambiguate qualifier from body
    (amendment)
    D-A10: Appendix A's implied age is the unrounded input, not the quotient
    (editorial)
    D-A11: obliquity is an angle and cannot be a RatedParam under Rule C
    (correction)
    D-A12: §9.6's synodic formula contradicts Appendix I.2; the year-relative
    form is correct (correction)
    D-A13: a drift bound is a rate in local units, not a Delta (correction)
    D-A14: §10.3's integral cannot be quadratured as written (correction)
    D-A15: Appendix H.4's monotone case does not apply to LambdaCDM (editorial)
    D-A16: §4.3's SI equivalent is printed on request, not always (amendment)

No network access is performed by any command (§8.4).
```

## `ucal to-civil 8070205189123984864657505252035637180530466139316558837890625 --digits 3`

An Earth calendar label. This is an Earth command, so it carries Earth units unasked.

```
ucal to-civil
─────────────
ticks        8070205189123984864657505252035637180530466139316558837890625
qualified    earth-civil: 2026-07-29T00:00:00.000 TT
calendar_id  earth-civil
kind         legacy — declared table data, outside Rule K (§8.6)
fields:
  year     2026
  month    7
  day      29
  hour     0
  minute   0
  second   0
  weekday  Wednesday
rounding     halfeven
lossy        false
warning      UCAL-W0005: value produced by a legacy (non-derived) calendar
```

## `ucal from-jd 2451545.0 --scale tt`

J2000.0, the epoch every parameter in this project is stated at, and the first time this program could convert it. `--scale` is required and has no default: a converter that defaults is silently wrong by 69 seconds whenever it guesses.

```
ucal from-jd
────────────
ticks  8070205173569972963515184424835637180530466139316558837890625
human  UC1 0031·0687·2481·1163·2191·0758:1924·0749·2247·0012·1174·0800·0000·
       0000·0000·0000·0000·0000
ucid   0000000000050PM6JSRZ1JEN8CJ8JG0H3SXHYWVS2CY7KBM29FJ1
input:
  jd     2451545.0
  jd     2451545/1
  scale  tt

Exact: a Julian day is a whole number of ticks, so nothing here is rounded.
`TT` is the pivot and `TT = TAI + 32.184 s` exactly. The scale is required and
has no default, because a converter that defaults is silently wrong by 69
seconds whenever it guesses.
```

## `ucal from-jd 2460000.5 --scale tdb`

The same conversion in a scale that is not exact. TDB differs from TT by a periodic series whose evaluation needs floating point, which Rule E forbids here — so the answer is a window carrying the series' 1.7 ms bound, rather than a centre that would look exact.

```
ucal from-jd
────────────
ticks  8070205187120737749440984658555873480530466139316558837890625
human  UC1 0031·0687·2481·2763·1541·0134:0558·2432·0156·0826·0630·0023·0035·
       0000·0000·0000·0000·0000
ucid   0000000000050PM6K2TPVFCTBADGEDG60D4QW3N7DZB7MXT29FJ1
input:
  jd     2460000.5
  jd     4920001/2
  scale  tdb
window:
  lo           8070205187120737749440984658555873480530466139316558837890625
  hi           8070205187120737749504049845515400880530466139316558837890625
  width_ticks  63065186959527400000000000000000000000000

`ticks` above is the low end. TDB differs from TT by a periodic series whose
evaluation needs floating point, which Rule E forbids in a shipped crate — so
the answer carries the series' bound of ±1.7 ms instead of a centre that would
look exact. Rule U: the window is the value.
```

## `ucal from-civil 2026-08-07`

And back. Exact or an error, never rounded.

```
ucal from-civil
───────────────
ticks  8070205189138408243886837165635637180530466139316558837890625
human  UC1 0031·0687·2481·3001·2180·0211:1363·0021·1485·0329·1114·1250·0000·
       0000·0000·0000·0000·0000
ucid   0000000000050PM6K45VCYNVTGKQMQHCXN9QJWQQ9WS4SBM29FJ1
input:
  label      2026-08-07
  scale      tt
  calendar   gregorian
  exactness  exact; construction never rounds (Rule R)
```

## `ucal wallclock --once --at 8070205189123984864657505252035637180530466139316558837890625 --theme startrek --height 26`

LCARS. The elbow, the rail of tier readouts, the beat in block digits, and the flicker as a bar because a number would be wrong before it was drawn.

```
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄   UNIVERSE CALENDAR · UC1
████████████████
████████████████   ABSOLUTE TIME · PLANCK TICKS · BASE FIVE
███████▀▀▀▀▀▀▀▀▀
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  █████ █   █ █████ █████
        T3 SPAN       █ █   █     █     █
           2481   █████ █████  ████    █
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  █         █     █   █
       T2 SWEEP   █████     █ █████   █
           2999
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄   T1 ARC   3108  ONE STOP EVERY 2 MIN 26 S
         T1 ARC
           3108   ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄  T-1 FLICKER · 66 000 PER SECOND · A POSITION, NOT A NUMBER
        T0 BEAT
           2437
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
    T-1 FLICKER   UC1 0031·0687·2481·2999·3108·2437
           1104
▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄




                  ▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄
                   Q TO DISENGAGE
```

## `ucal wallclock --once --at 8070205189123984864657505252035637180530466139316558837890625 --theme starwars --height 24`

A targeting computer: canopy brackets, a reticle round the beat, the flicker riding the crosshair, and every other hand compressed into one strip. An instrument, where LCARS is a console.

```
┌──────                                                                  ──────┐
  TARGETING · UC1                                                 LOCK T1 3108
  ABSOLUTE TIME · PLANCK TICKS · BASE FIVE
 ┌───                                                                      ───┐
 │                          █████ █   █ █████ █████                           │
 │                              █ █   █     █     █                           │
 │                          █████ █████  ████    █                            │
 │                          █         █     █   █                             │
 │                          █████     █ █████   █                             │
 └───                                                                      ───┘
 ━━━━━━━━━━━━━━━━━━━━━━━━━━━────────────┼──────────────────────────────────────
  T-1 FLICKER ON THE AXIS · 66 000 PER SECOND
  T3 SPAN 2481   T2 SWEEP 2999   T1 ARC 3108   T0 BEAT 2437   T-1 FLICKER 1104

  UC1 0031·0687·2481·2999·3108·2437

  [Q] DISENGAGE






└──────                                                                  ──────┘
```

## `ucal wallclock --once --at 8070205189123984864657505252035637180530466139316558837890625 --gagarin --locale ru --height 24`

A Vostok instrument panel: an enamelled plate with bezelled gauges set into it, engraved labels, and a red lamp. Everything Cyrillic here comes from --locale ru, including the chrome: through 1.8.0 the plates were hardcoded Russian and this theme was the one place that could override the flag.

```
 ВРЕМЯ ВСЕЛЕННОЙ · UC1
 АБСОЛЮТНОЕ ВРЕМЯ · ПЛАНКОВСКИЕ ТИКИ · ОСНОВАНИЕ ПЯТЬ
 ───────────────────────────────────────────────────
 ┌────────────┐  ┌────────────┐  ┌────────────┐
 │    2481    │  │    2999    │  │    3108    │
 └────────────┘  └────────────┘  └────────────┘
   T3 ПРОЛЁТ       T2 ОБХОД         T1 ДУГА

 ┌──────────────────────────────┐
 │ █████ █   █ █████ █████     │
 │     █ █   █     █     █     │
 │ █████ █████  ████    █      │
 │ █         █     █   █       │
 │ █████     █ █████   █       │
 └──────────────────────────────┘
        T0 БОЙ · ОСНОВНОЙ ОТСЧЁТ

 ● ▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▆▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
   T-1 · 66 000 В СЕКУНДУ · ПОЛОЖЕНИЕ, НЕ ЧИСЛО

 UC1 0031·0687·2481·2999·3108·2437

 ● ГОТОВ     [Q] ВЫХОД

```

## `ucal wallclock --once --at 8070205189123984864657505252035637180530466139316558837890625 --armstrong --height 24`

An Apollo DSKY. VERB 16 NOUN 65 is a real pair — monitor, decimal, time — and the lamps are drawn unlit except COMP ACTY, because the rest report conditions this program does not have.

```
┌────────────┐  PROG  01    UNIVERSE CALENDAR · UC1
│ COMP ACTY  │  VERB  16    NOUN  65   MONITOR DECIMAL · TIME
│UPLINK ACTY │  ──────────────────────────────────────────
│   NO ATT   │
│    STBY    │  R1   +02999   T2 SWEEP
│  KEY REL   │  R2   +03108   T1 ARC
│  OPR ERR   │
│  TRACKER   │  R3   T0 BEAT · 21 PER SECOND
└────────────┘  █████ █   █ █████ █████
                    █ █   █     █     █
                █████ █████  ████    █
                █         █     █   █
                █████     █ █████   █

                ▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▮▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯▯
                T-1 FLICKER · TOO FAST FOR A REGISTER

                UC1 0031·0687·2481·2999·3108·2437

                [Q] KEY REL




```

## `ucal wallclock --once --at 8070205189123984864657505252035637180530466139316558837890625 --theme orbit --height 22`

Hands, on dials, in braille. Every tier has 3125 stops because every rung is 5^5 of the one below, so each tier is a circle and they nest. Drawn with integer CORDIC: there is no float in this program and a clock face is a bad place to make the first exception.

```
 UCAL — universe calendar, on dials
 every tier has 3125 stops, because every rung is 5^5 of the one below
⠀⠀⠀⠀⢀⡠⠤⠴⠤⠤⣀⠀⠀⠀⠀ ⠀⠀⠀⠀⢀⡠⢤⠴⠤⠤⣀⠀⠀⠀⠀ ⠀⠀⠀⠀⢀⡠⠤⢴⠤⠤⣀⠀⠀⠀⠀ ⠀⠀⠀⠀⢀⡠⠤⠴⠤⠤⣀⠀⠀⠀⠀ ⠀⠀⠀⠀⢀⡠⠤⠴⠤⠤⣀⠀⠀⠀⠀
⠀⠀⢠⠞⠁⠀⠀⠀⠀⠀⠀⠙⢦⠀⠀ ⠀⠀⢠⠞⠁⠀⠸⡀⠀⠀⠀⠙⢦⠀⠀ ⠀⠀⢠⠞⠁⠀⠀⢸⠀⠀⠀⠙⢦⠀⠀ ⠀⠀⢠⠞⠁⠀⠀⠀⠀⠀⠀⠙⢦⠀⠀ ⠀⠀⢠⠞⠁⠀⠀⠀⠀⠀⠀⠙⢦⠀⠀
⠀⢠⣃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢣⠀ ⠀⢠⠃⠀⠀⠀⠀⢇⠀⠀⠀⠀⠀⢣⠀ ⠀⢠⠃⠀⠀⠀⠀⢸⠀⠀⠀⠀⠀⢣⠀ ⠀⢠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢣⠀ ⠀⢠⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢣⠀
⠀⢸⠀⠉⠑⠒⠢⠤⠀⠀⠀⠀⠀⢸⠀ ⠀⢸⠀⠀⠀⠀⠀⠸⠀⠀⠀⠀⠀⢸⠀ ⠀⢸⠀⠀⠀⠀⠀⠸⠀⠀⠀⠀⠀⢸⠀ ⠀⢸⠉⠒⠒⠢⠤⠤⠀⠀⠀⠀⠀⢸⠀ ⠀⢸⠀⠀⠀⠀⠀⠠⣀⠀⠀⠀⠀⢸⠀
⠀⠸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠀ ⠀⠸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠀ ⠀⠸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠀ ⠀⠸⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡸⠀ ⠀⠸⡀⠀⠀⠀⠀⠀⠀⠓⠤⡀⠀⡸⠀
⠀⠀⠱⣄⠀⠀⠀⠀⠀⠀⠀⢀⡴⠁⠀ ⠀⠀⠱⣄⠀⠀⠀⠀⠀⠀⠀⢀⡴⠁⠀ ⠀⠀⠱⣄⠀⠀⠀⠀⠀⠀⠀⢀⡴⠁⠀ ⠀⠀⠱⣄⠀⠀⠀⠀⠀⠀⠀⢀⡴⠁⠀ ⠀⠀⠱⣄⠀⠀⠀⠀⠀⠀⠀⢉⡶⠁⠀
⠀⠀⠀⠈⠑⠢⠤⠤⠤⠤⠒⠉⠀⠀⠀ ⠀⠀⠀⠈⠑⠢⠤⠤⠤⠤⠒⠉⠀⠀⠀ ⠀⠀⠀⠈⠑⠢⠤⠤⠤⠤⠒⠉⠀⠀⠀ ⠀⠀⠀⠈⠑⠢⠤⠤⠤⠤⠒⠉⠀⠀⠀ ⠀⠀⠀⠈⠑⠢⠤⠤⠤⠤⠒⠉⠀⠀⠀
     2481            2999            3108            2437            1104
    T3 span        T2 sweep         T1 arc          T0 beat       T-1 flicker
 ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
 the finest hand has no dial: 66 000 stops a second is not a hand

 UC1 0031·0687·2481·2999·3108·2437

 q to quit





```

## `ucal wallclock --once --at 8070205189123984864657505252035637180530466139316558837890625 --clock-local mars-d --height 20`

The plain theme with a second dial. A wall clock's second face has always shown another place, and Mars is one.

```
UCAL — universe calendar
█████ █   █ █████ █████
    █ █   █     █     █
█████ █████  ████    █
█         █     █   █
█████     █ █████   █

T3 span       2481
T2 sweep      2999
T1 arc        3108
T0 beat       2437
T-1 flicker   1104

MARS-D        year 82  day 83   counted from the anchor — year 1 began there
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░ 44% through the local day
anchor revision 1 — an anchor is an observation and is versioned (Rule J)


UC1 0031·0687·2481·2999·3108·2437
q to quit
```

## `ucal explain abc`

A rejection: an Appendix E code, an exit status, and — since 1.2.0 — what a good input would have looked like.

```
UCAL-E0001: malformed timestamp (expected a decimal tick count like 8070205189123984864657505252035637180530466139316558837890625, a UC1 text form like `UC1 0031·0687·...`, a 52-character UCID, or `-` to read instants from stdin on the commands that take a single one. `ucal now` prints one of each; `ucal tour` shows what to do with them)
```
