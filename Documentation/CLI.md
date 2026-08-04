# `ucal` — command reference

Every command, every option, and what each field of the output means.

The book (*Life, the Universe, and God*) is an argument and does not document
this surface; `spec/UCAL-1.1.md` §19 is normative and constrains only four
commands. This file is the manual, and the fields are its point — a reader
looking at `remainder_ticks` or `window_ticks` should not have to read `lib.rs`
to find out what they are.

**Scope note.** Command names, subcommands and global options are checked
against the source by `cargo run -p xtask -- check-docs`, which fails when this
file and the CLI disagree. The prose is not generated and can still go stale;
where it does, the source is right.

---

## Contents

- [Global options](#global-options)
- [Reading a timestamp](#reading-a-timestamp)
- [Exit codes](#exit-codes)
- Commands: [`now`](#ucal-now) · [`datum`](#ucal-datum) ·
  [`explain`](#ucal-explain) · [`from-civil`](#ucal-from-civil) ·
  [`to-civil`](#ucal-to-civil) · [`ladder`](#ucal-ladder) ·
  [`cal`](#ucal-cal) · [`show`](#ucal-show) · [`events`](#ucal-events) ·
  [`timeline`](#ucal-timeline) · [`ruler`](#ucal-ruler) ·
  [`cosmo`](#ucal-cosmo) · [`doctor`](#ucal-doctor)
- [Recurring fields](#recurring-fields)

---

## Global options

Accepted by every command (§19.1).

| option | default | what it does |
|---|---|---|
| `--profile <TAG>` | `UC-1` | The profile to compute in. Only `UC-1` exists; anything else is exit 5 (Rule P). |
| `--sep <CHAR>` | `·` | Group separator inside the base-5 text forms. Must not be a digit (§6.3). |
| `--locale <TAG>` | `en` | Locale for tier names. `en` or `ru`. Affects both display *and* parsing — under `--locale ru`, `--step пролёт` and `--step пр` resolve. |
| `--json` | off | Stable, versioned JSON instead of text. Never coloured. |
| `--color <WHEN>` | `auto` | `auto`, `always` or `never`. `auto` colours only into a terminal. |
| `--no-color` | off | Alias for `--color never`, and it wins over `--color`. |
| `--width <N>` | terminal, else 80 | Columns to lay out at. Never below 80. |
| `--tick-sep <CHAR>` | none | Separator between three-digit groups in decimal counts. Off by default so a copied tick count is still an integer. |
| `--decimals <N>` | each field's own | Fractional digits for every rendered rational. Without it each field keeps the precision it was written with, so the default output is unchanged. |
| `--round <MODE>` | each field's own | `trunc`, `ceil`, `half-even` or `half-up`, for every rendered value including `to-civil`'s sub-second field. |

### Colour, and what it is allowed to mean

Nothing you cannot also read. `strip_ansi(coloured) == plain`, byte for byte,
is a test — so anything colour shows you is available down a pipe, in a log, or
on a terminal with no colour at all.

| appearance | meaning |
|---|---|
| bold | a field name, or a document title |
| dimmed | a group separator, or the **leading zero run** of a fixed-width form — domain the value has not reached |
| alternating cyan | three-digit grouping in a decimal count of seven digits or more |
| yellow | a `UCAL-W####` warning |
| red | a `UCAL-E####` error |

`NO_COLOR` is honoured. Precedence: `--no-color` > `--color` > `NO_COLOR` > tty
detection.

---

## Reading a timestamp

Anywhere a command takes `<INSTANT>`, three spellings are accepted:

| form | example |
|---|---|
| a `UC1` text form | `UC1 0031·0687·2481·…` |
| a UCID | `0000000000050PM6K45R0F8GAP92CDK4XPSK7NFPQNP5WJZ9FJ1` |
| a decimal tick count | `8070205189123984864657505252035637180530466139316558837890625` |

A tier is written `T<k>`, `5^e`, or a locale name — `T4`, `5^80`, `drift`,
`дрейф`, `др`. All are accepted wherever any of them is (Rule N).

---

## Exit codes

§19.5. A failure is a status, not a message to parse.

| exit | meaning |
|---|---|
| 0 | success |
| 1 | usage error |
| 2 | parse error |
| 3 | domain error (Rules Z, O, W) |
| 4 | precision error (Rules T, R) |
| 5 | profile mismatch (Rule P) |
| 6 | data or config error, including missing provenance (Rule Q.4) |
| 7 | calendar derivation or anchor error (Rules K, J, C) |
| 8 | cosmology model or enclosure error (Rule X) |
| 9 | internal invariant violation, including metadata used as an operand (Rule Q.3) |

---

## `ucal now`

The current instant, from the system clock. Performs no network access (§8.4).

```
ucal now [--precision <TIER>] [--form human|digit5|named]
```

| option | default | notes |
|---|---|---|
| `--precision` | `T-12` | The tier to render to. `T-12` is the tick — exact. |
| `--form` | `human` | Which text form the `rendered` field uses. |

| field | meaning |
|---|---|
| `ticks` | The instant as an unsigned decimal count of Planck ticks since the datum. This is the value; everything else is a rendering of it. |
| `human` | The `UC1` text form at full precision: base-5 digits grouped in fives, with `:` marking the beat (T0) boundary. |
| `ucid` | The sortable 52-character identifier. **Contains no randomness** — it is a pure function of the instant, so two events at the same tick get the same UCID (Rule I). |
| `rendered` | The instant at `--precision`, in `--form`. Coarser precision **truncates**; it does not pad, so a shorter string means a coarser value. |
| `precision` | The tier `rendered` was cut to. A value stated to tier `e` denotes the interval `[v, v + 5^e − 1]` (Rule T). |
| `source.clock` | Where the reading came from. |
| `source.leap_table` | The bundled IERS table version used to convert it. |
| `source.network` | Always `none (§8.4)`. |

---

## `ucal datum`

What tick 0 is, what is claimed about it, and how it was fixed. The output
order is normative (§19.2).

```
ucal datum
```

| field | meaning |
|---|---|
| `datum` | The statement itself: tick 0 is a *stipulated reference point*, conventionally identified with the FLRW `t→0` limit. Not a measurement and not an observed event. |
| `frame` | The reference frame the count is in. |
| `tick_zero` | `0`, by construction. |
| `big_bang_claim.window` | The published identification of the origin, as a **signed** interval of ticks. Signed because the limit may lie before the datum, which is not representable as a tick. |
| `big_bang_claim.half_width_ticks` | Half the window, exactly. |
| `big_bang_claim.half_width_drifts` | The same width on the ladder: 141.53 drifts. |
| `big_bang_claim.citation` / `.locator` | Where the number comes from. |
| `big_bang_claim.status` | **`metadata only; no arithmetic operation may consume it`.** This is enforced by the type system, not by discipline: the value lives in a type with no operators, and three tests exist whose job is to fail to compile (Rule Q.3). |
| `datum_provenance.input` | The published age and uncertainty the datum was derived from, verbatim. |
| `datum_provenance.unit_defs[]` | The exact definitions used, so the chain can be re-executed. |
| `datum_provenance.chain[]` | Every arithmetic step from the published age to `ORIGIN_OFFSET`. Re-executable, and `xtask` re-executes it. |
| `rounding.to` / `.mode` | What the derived value was rounded to, and how. |
| `rounding.residual_ticks` | Exactly what the rounding discarded. Reported rather than absorbed. |
| `rounding.residual_rendered` | The same residual in seconds: `−0.017190364 s`. |
| `rounding.rationale` | Why a whole-beat datum was chosen. |
| `earth_dependency` | A plain statement of where Earth enters: the input arrives in Julian years and the bridge anchor is an Earth date. Both are metrology (Rule Y); neither enters a computation. |
| `implied_age` | A *consequence* of the declared datum, not a measurement. The measurement is `datum_provenance.input`. §19.2 forbids presenting this as an age of the universe, and a lint enforces the wording. |

---

## `ucal explain`

Everything about one instant.

```
ucal explain <INSTANT> [--claim]
```

| option | notes |
|---|---|
| `--claim` | Also print `BIG_BANG_CLAIM`. Metadata; never an operand. |

| field | meaning |
|---|---|
| `ticks` | The instant, exactly. |
| `precision` | `tick (exact)` when the input determined every digit, otherwise the tier it was stated to. |
| `human` / `digit5` | The two text forms. `digit5` is **fixed-width** so that lexicographic order equals chronological order (Rule S) — which is why it opens with 27 groups of zeros at the present epoch. |
| `ucid` | The sortable identifier, or `— (outside 2^256, UCAL-E0031)` for an instant beyond UCID's range. |
| `tiers.T<k> <name>` | The instant decomposed onto the ladder: how many of each tier, from T5 down to T0. Reassembles to `ticks` exactly. |
| `beats_since_datum.whole` | How many whole **beats** — universe seconds, `5^60` ticks — have elapsed. |
| `beats_since_datum.remainder_ticks` | The ticks left over after those whole beats. `whole × BEAT + remainder_ticks == ticks`. |
| `si_bridge.unit` | The declared foreign unit: the SI second. |
| `si_bridge.epoch` | The civil instant the bridge is anchored to. |
| `si_bridge.seconds_from_epoch` | The instant expressed through the bridge. Informative (Rule A.5) — this is the *only* place Earth enters, and it is division, so it is the only place a rounding mode is chosen. |
| `warning` | Present when the instant lies inside the claim's half-width (`UCAL-W0006`). |

---

## `ucal from-civil`

A civil date to absolute time. Exact, or an error — never rounded.

```
ucal from-civil <DATE> [--scale tt|tai|utc] [--calendar gregorian|julian]
```

`<DATE>` accepts `2026-07-29`, `2026-07-29T12:34:56.5`, `-0043-03-15`, and
`44 BC-03-15`.

| option | default |
|---|---|
| `--scale` | `tt` |
| `--calendar` | `gregorian` |

| field | meaning |
|---|---|
| `ticks` / `human` / `ucid` | The instant the label denotes. |
| `input.label` | The date as given. |
| `input.scale` | Which time scale it was read in. |
| `input.calendar` | Which calendar. Both shipped calendars are **legacy** (§8.6): declared tables, not derived. |
| `input.exactness` | Whether the conversion was exact. It always is, because a label finer than a tick is refused (`UCAL-E0043`) rather than rounded. |

---

## `ucal to-civil`

Absolute time to a civil label. **The only place this program rounds** (Rule R).

```
ucal to-civil <INSTANT> [--scale tt|tai|utc] [--digits N] [--calendar gregorian|julian]
```

| option | default | notes |
|---|---|---|
| `--digits` | `0` | Fractional-second digits of the civil label, up to 30. Distinct from the global `--decimals`, which governs rendered rationals. |

`--round` is global. A civil label's sub-second field and a rendered rational
are rounded by the same declared choice, and `half-even` remains the default for
both.


| field | meaning |
|---|---|
| `ticks` | The instant that was converted. |
| `qualified` | The rendered label with its calendar qualifier attached, so a bare date can never circulate without saying which calendar it is in. |
| `calendar_id` / `kind` | Which calendar, and whether it is `derived` or `legacy` (§19.4 requires `kind` on every rendering). |
| `fields.*` | `year`, `month`, `day`, `hour`, `minute`, `second`, `weekday`. |
| `rounding` | The mode actually applied. |
| `lossy` | Whether digits were discarded. `false` means the label denotes the exact tick. |
| `warning` | Any `UCAL-W####` raised by the conversion. |

---

## `ucal ladder`

The universal tier grid (§4.2): body-independent, and the canonical way to
state any duration.

```
ucal ladder [--named-only]
```

| option | notes |
|---|---|
| `--named-only` | Show only the ten named tiers. The rest stay addressable by index (D-20). |

| field | meaning |
|---|---|
| `locale` | The locale the `name` column is in. |
| `note` | What the ladder is, and that names are display-only. |
| `tiers.T<k>.exponent` | The tier's **canonical identity**. The name is an alias; nothing decides behaviour from one (Rule N). |
| `tiers.T<k>.name` | Singular and plural in the chosen locale, or `—` for an unnamed tier. |
| `tiers.T<k>.beats` | The tier in universe seconds. Exact by construction — every tier is a whole power of five of the beat. |
| `tiers.T<k>.seconds (bridge)` | The same span in SI seconds. **Informative** (Rule A.5), shown alongside and never instead. |
| `tiers.T<k>.ticks` | The tier as an exact integer count of ticks. |

The two seconds are incommensurable above T-6: one bridge second is
21.385061835 beats, because `BEAT` carries `5^60` while `SECOND` carries only
`5^30`. They share a common measure only at the tick.

---

## `ucal cal`

Calendars, derived and legacy, each labelled with which it is.

```
ucal cal list
ucal cal show <ID> <INSTANT>
ucal cal anchor <ID>
```

### `cal list`

| field | meaning |
|---|---|
| `calendars.<id>.kind` | `derived — Rule K` or `legacy — declared tables (§8.6)`. §19.4 makes this mandatory on every entry. |
| `calendars.<id>.body` | The body whose periods it is derived from. |
| `calendars.<id>.anchor_revision` | Which revision of the body's anchor was used. Anchors are versioned because they are observations (Rule J). |
| `calendars.<id>.leap_rule` | The intercalation rule, **derived** by continued fraction, with which convergent it is. Earth's is `31/128 (convergent 4)`. |
| `calendars.<id>.cycles` | The grouping cycle, or a statement that the body has none. Mars has no month: neither moon qualifies, and the mechanism returns nothing rather than inventing one. |
| `calendars.<id>.status` | Present instead of the above when a calendar is structurally complete but unusable — Titan has no published anchor to cite, so asking for local fields is `UCAL-E0062`. |

### `cal show`

| field | meaning |
|---|---|
| `calendar` / `kind` / `body` | Which calendar this is. |
| `anchor` | The anchor instant, its revision, and its uncertainty window. |
| `intercalation` | The derived leap rule and the continued-fraction expansion behind it. |
| `fields` | The instant rendered in this calendar's local fields. |
| `cycles` | The derived grouping cycles, if the body has any. |

### `cal anchor`

| field | meaning |
|---|---|
| `phase` | What the anchor fixes. A body's periods give you a calendar's *shape*; only an anchor gives it a *phase*. |
| `revision` | The anchor's version. |
| `tick` | The anchored instant. |
| `window` | Its uncertainty, as an interval. An anchor is an observation and carries one. |
| `determination` | How it was determined, cited. |

---

## `ucal show`

One instant in several local calendars. §19.4 calls this the primary
demonstration of Rules K and J.

```
ucal show <INSTANT> [--calendars <ID,ID,…>]
```

| option | default |
|---|---|
| `--calendars` | `earth-d,mars-d,earth-civil` |

| field | meaning |
|---|---|
| `ticks` / `human` | The one absolute instant all the renderings are of. |
| `calendars.<id>.rendered` | The local label. |
| `calendars.<id>.kind` | `derived (Rule K)` or `legacy (§8.6)`, on every row. |
| `calendars.<id>.anchor_revision` | Which anchor revision produced it. |
| `calendars.<id>.window_ticks` | The uncertainty the anchor contributes, in ticks. A local date is only as sharp as the anchor behind it. |
| `calendars.<id>.day_is_ambiguous` | Whether the instant falls close enough to a day boundary that the anchor's window straddles it. |
| `calendars.<id>.error` | Present instead of fields when a calendar cannot render — a missing anchor is `UCAL-E0062`, not a guess. |

---

## `ucal events`

Cited, interval-valued milestones (§17).

```
ucal events list
ucal events show <ID>
```

| field | meaning |
|---|---|
| `citation_set` | The version of the catalogue. Versioned by citation set, independently of the library, so a revised citation does not force a release (D-7). |
| `events.<id>.label` | The event's name. |
| `events.<id>.as_published` | The figure **as its source states it**, verbatim, in the source's own units. |
| `events.<id>.window_ticks` | That figure converted to an interval of ticks. **Every entry is an interval**, because not one of these is known to a point. |
| `events.<id>.citation` | The source. |
| `events.<id>.warning` | `UCAL-W0006` where the event's window overlaps the claim's half-width — i.e. where the dating is not separable from the uncertainty in the datum itself. |
| `stated_as` (`show`) | Whether the source gave a point, a range, or a bound. |
| `midpoint` (`show`) | The window's midpoint. A *rendering choice*, not a claim: the window is the datum. |

---

## `ucal timeline`

The whole catalogue against the tier ladder, on one screen.

```
ucal timeline [--tier <TIER>]
```

| option | default |
|---|---|
| `--tier` | `drift` |

| field | meaning |
|---|---|
| `tier` | The tier positions are stated in. |
| `events.<label>.at` | The event's position, rendered at that tier. |
| `events.<label>.T<k>s since the datum` | The same position as a plain count of that tier. |
| `events.<label>.as_published` | The source's own figure, alongside. |
| `events.<label>.warning` | As in `events list`. |

Positions are the windows' **midpoints floored to the stated tier**. The
midpoint is a rendering choice; the window is what is known.

---

## `ucal ruler`

Evenly spaced marks on the tier grid.

```
ucal ruler --from <INSTANT> --to <INSTANT> --step <TIER>
```

| field | meaning |
|---|---|
| `from` / `to` | The bounds, in ticks. |
| `step` | The tier between marks. |
| `whole_steps` | How many whole steps fit between the bounds. |
| `marks.<n>` | Each mark, rendered at the step tier. |

---

## `ucal cosmo`

Flat ΛCDM, by certified integer quadrature (§10). No floating point anywhere:
every answer is an **enclosure** — two numbers and a guarantee the true value
lies between them — rather than one number and a hope.

```
ucal cosmo model
ucal cosmo age --z <REDSHIFT> [--depth N] [--scale N]
ucal cosmo z --at <INSTANT> [--tolerance-years N] [--depth N] [--scale N]
```

### `cosmo model`

| field | meaning |
|---|---|
| `model` | The model identifier, carried on every result (Rule X). |
| `as_published.*` | `H0`, `Omega_m`, `Omega_Lambda`, `Omega_r`, each **verbatim as published**, uncertainty included. |
| `citation` | Planck 2018. |
| `hubble_time` | `1/H0` in ticks and in Gyr. |
| `monotonicity.turns_at_u` | Where the integrand stops being monotone — `u ≈ 0.604`. Published because Appendix H.4 requires monotonicity to be *asserted, not assumed*, and here it fails, which is why the panels use an interval extension. |
| `ge1` / `ge2` | The measured outcomes of two gated experiments, including the kill criteria that fired. |

### `cosmo age`

| option | default | notes |
|---|---|---|
| `--depth` | `12` | `2^depth` panels. Cost grows about 4× per step. |
| `--scale` | `12` | Decimal digits for the directed square roots. |

| field | meaning |
|---|---|
| `z` | The redshift asked for. |
| `enclosure.lo_ticks` / `.hi_ticks` | The age, as an interval of ticks. The answer is the *pair*. |
| `enclosure.lo_years` / `.hi_years` | The same in years. |
| `enclosure.at_drift` | The same on the ladder. |
| `widths.arithmetic_years` | How much of the width comes from the quadrature. |
| `widths.parameter_years` | How much comes from **Planck's own error bars**. |
| `quadrature.depth` / `.panels` / `.sqrt_scale_digits` | What was actually computed. |

The two widths are **never merged**, and the reason is visible in the numbers:
at `z = 1100` the arithmetic width is 251 years against a parameter width of
10 917 years. The quadrature is already forty times sharper than the
measurement it is integrating. Merging them would hide which one matters.

### `cosmo z`

| option | default | notes |
|---|---|---|
| `--tolerance-years` | `1` | Resolution required *in time*. |
| `--depth` | `8` | |

| field | meaning |
|---|---|
| `instant_ticks` / `years_after_datum` | The instant asked about. |
| `tolerance_years` | The resolution the bisection converged to. |
| `z` | The redshift, as an interval. Both ends come from separate bisections against opposite bounds of the age enclosure — one bisection would give a narrow, plausible bracket that encloses nothing. |

A tolerance finer than about a millisecond is refused with `UCAL-E0071`. The
limit is not the step budget: the bisection midpoints leave the 512-bit domain
at around step 125, with the bracket still some `7.8 × 10^26` ticks wide.

---

## `ucal doctor`

Profile, backend, ceiling, leap table, features, provenance (§19.3).

```
ucal doctor
```

| field | meaning |
|---|---|
| `profile` / `frame` | Which profile is in force. |
| `backend` | Which integer backend was compiled in: `u512` (stack, `Copy`) or `bigint` (heap). Both produce byte-identical results — that is what Rule W buys. |
| `domain_max_ticks` | The largest representable instant: `2^512 − 1`, all 155 digits. |
| `domain_bits` | `512`. Fixed so it never has to change; the canonical binary form is 64 bytes *because* of this. |
| `features[]` | Which optional crates are compiled in. |
| `datum_provenance.present` | Whether a provenance record exists. Its absence is `UCAL-E0013` — a profile without provenance is not usable (Rule Q.4). |
| `leap_seconds.table_version` | The bundled IERS bulletin. |
| `leap_seconds.entries` | How many leap seconds are in it. |
| `leap_seconds.complete_through` | The date the table is authoritative to. |
| `leap_seconds.pre_1972` | That the 1961–1972 rubber-second era is modelled exactly, and UTC before 1961 is `UCAL-E0041` rather than an extrapolation. |
| `leap_seconds.network` | `never` — the table is bundled and offline (§8.4). |
| `spec.rfc` | Which specification this build implements. |
| `spec.deltas[]` | Every recorded correction to the RFC, with its class: amendment, correction or editorial. |

---

## Recurring fields

Names that mean the same thing wherever they appear.

| field | meaning |
|---|---|
| `ticks` | An unsigned integer count of Planck ticks since the datum. Always exact. Emitted as a JSON *string*, because a 61-digit integer cannot survive a JSON number. |
| `window` / `window_ticks` | An interval, `[lo, hi]`. Wherever one appears, the quantity is not known to a point, and the interval is the answer rather than an apology for it. |
| `kind` | `derived` or `legacy`. Mandatory on every calendar rendering (§19.4) so a declared table can never be mistaken for a derived one. |
| `anchor_revision` | Which revision of a body's anchor produced a rendering. Anchors are observations and are versioned. |
| `precision` | The tier a value was stated to. Coarser than tick means the value denotes an interval (Rule T). |
| `widths.arithmetic_years` | Width contributed by *this program's* arithmetic. |
| `widths.parameter_years` | Width contributed by the *measurements* being computed with. Never merged with the above (failure mode F8). |
| `citation` | Where a number came from. Every measured quantity in this program carries one. |
| `notes[]` | Explanatory prose. Never load-bearing: nothing in the output depends on a note being read. |

### Asking for more digits

Rule R makes rendering the only place a value may be rounded, so that place
answers to the caller rather than to a constant. A tick's length in beats is
`1 / 5^60` — a finite expansion sixty places long — and the default six digits
print it as zero:

```
$ ucal ladder --named-only | grep T-12
T-12  0   tick / ticks   0.000000

$ ucal ladder --named-only --decimals 60 --json | jq -r '.tiers["T-12"].beats'
0.000000000000000000000000000000000000000001152921504606846976
```

At sixty digits that value is **exact**, and the `certification` map below drops
it while keeping `seconds (bridge)`, which never terminates at any digit count.

### `certification`

Every document that rounds anything carries a `certification` object mapping a
field's dotted path to what was done to it:

```json
"certification": {
  "tiers.T5.beats": "rounded, half-even, 6 digits",
  "tiers.T5.seconds (bridge)": "rounded, half-even, 6 digits"
}
```

**Only the exceptions are listed.** Exactness is the expectation, so a numeric
field *absent* from this map is being told its printed digits are the value —
and that is a claim rather than a convention: `tests/certification.rs` checks
that the map lists every non-exact quantity and nothing else, that anything
called exact reparses to the value it prints, and that no rendered decimal
reaches the output without going through the certified constructor at all.

`exact` is a claim about *this rendering*, not about the number in the abstract.
A tick in beats is `1 / 5^60` — a finite expansion sixty places long — so it is
exact at sixty digits and a rounding at six, where it prints as `0.000000`.

None of this is floating point. Rule E forbids a float token in any shipped
crate; a decimal is produced by one integer multiply-divide,
`mul_div_rounded(numerator, 10^digits, denominator, mode)`, with a decimal point
inserted into the result's digits.

### The `--json` contract

`--json` output is stable and versioned (§19.1). Every document carries
`"format": "ucal-json/1"`, so a consumer can tell when it changes.

Stability means **existing fields keep their names, shapes and meanings**. New
fields may be added without a version bump — `certification` was added in 0.4.0
this way — so a consumer should ignore keys it does not know rather than reject
them.

Numbers are emitted as **strings**, deliberately. A tick count exceeds every
JSON number implementation in practice, and a consumer that silently converted
one to a double would lose exactly the exactness this specification exists to
provide.

Text-mode layout — tables, wrapping, colour, group separators — never affects
JSON. A table is a rendering decision; the document underneath is the same
either way.
