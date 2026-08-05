# The rules

RFC UCAL-1 names twenty-four rules by single letters and cites them throughout.
The implementation cites them 538 times. This file is what those citations
resolve to.

Each entry gives what the rule requires, the failure mode it guards, and where
it is enforced — because a rule enforced by the type system and a rule enforced
by a comment are not the same rule.

Authority: [`RFC-UCAL-1.md`](RFC-UCAL-1.md) §11 for the index, §0.4 for the
failure modes. Where the corrected specification amends a rule, the amending
delta is named and [`SPEC-DELTAS.md`](SPEC-DELTAS.md) carries the reasoning.

**Enforcement is ranked.** *Type system* means it cannot be written. *Lint* means
it cannot survive review. *Test* means it cannot regress unnoticed. *Convention*
means it is only as strong as the next person's care — and only three rules
rest there.

---

## These rules are a framework, not dogma

**Any rule here may be overridden.** They exist to make the reasoning behind the
design available, not to bind the project to decisions taken before the code was
written. Several have already been amended — Rule K by D-A5, Rule C by D-A11 and
D-A13, Rule X by D-A14 — and each amendment made the design better, not weaker.

What matters is not that a rule is never broken. It is that breaking one is a
**decision** rather than a drift.

So the only discipline attached to an override is: **record it.** A new `D-A`
entry in [`SPEC-DELTAS.md`](SPEC-DELTAS.md) saying what the rule required, what
is done instead, and why. That is the same logic the `float-free` lint already
uses — the exemption is permitted, and the report lists every one it honoured,
so an escape hatch cannot quietly become a retreat.

An unrecorded override is not a rule violation; it is a lost explanation. The
expensive part of this project was never the code.

One practical note: where a rule is enforced by the **type system**, overriding
it means changing the type, and the compile-fail tests will tell you exactly
which guarantee you are spending. That is a feature. The cost should be visible
at the moment it is paid.

---

## The datum, and what is not claimed about it

### Rule Q — the datum is stipulated  §1.3

Tick 0 is a **stipulated reference point**, conventionally identified with the
FLRW `t→0` limit. It is not a measurement, not an observed event, and not the
creation of anything.

Four parts:

- **Q.1** — no description may present tick 0 as measured or observed.
- **Q.2** — `BIG_BANG_CLAIM` is declared separately from the datum.
- **Q.3** — the claim is **non-consumable**: no arithmetic operation may take
  it as an operand.
- **Q.4** — `datum_provenance` is machine-readable data, not prose.

*Guards* **F11** (exactness of the arithmetic mistaken for accuracy about the
origin) and **F12** (Earth-flavoured provenance hidden in unauditable prose)
and **F13** (prose drift reintroducing overclaims).

*Enforced by* **type system** — `SignedWindow` is inert, with no operators, and
`compile_fail/signed_window_arithmetic.rs`, `signed_window_as_operand.rs` and
`signed_window_into_delta.rs` pin that it cannot reach `Instant` or `Delta`.
Also **lint** — `datum-no-overclaim` scans documentation prose. Also **test** —
the provenance chain re-executes to the declared `ORIGIN_OFFSET` and residual.

This is the rule the project is *about*. Everything else is arithmetic.

### Rule F — the frame is declared  §1.1

The reference frame is stated, not assumed: FLRW comoving, cosmological time,
CMB rest frame.

*Enforced by* **convention** — the profile carries `FRAME` and every `datum`
rendering prints it. Nothing prevents a second profile from declaring a
different frame; that is the point of declaring it.

---

## The tick, and the units that are not it

### Rule A — atomicity  §1.4

The **tick** is primitive. Bridge constants are exact integers. No other unit
is fundamental — in particular, neither the SI second nor the beat.

*Guards* **F3** (foreign units injecting rounding into the core).

*Enforced by* **lint** — `core-names-no-foreign-unit` fails if `ucal-core` names
a foreign unit outside the `Bridge` declaration. Also **type system** — there is
no constructor taking seconds.

Worth stating plainly, because it is easy to misread: the beat (`5^60` ticks,
§0.5's *universe second*) is **not** a second in disguise. One SI second is
21.385061835 beats. They share a common measure only at the tick, because
`SECOND` carries `5^30` and `BEAT` carries `5^60` — which is exactly why the
tick is primitive rather than either.

### Rule Y — metrology  §1.5

Foreign units enter at **three declared points only**. Declared constants are
in ticks.

*Guards* **F3**, and **F9** with Rule K.

*Enforced by* **lint** (`core-names-no-foreign-unit`) and by the crate graph:
`ucal-core` does not depend on `ucal-civil`.

### Rule Z — zero and the unsigned domain  §1.2

Absolute time is an **unsigned** integer count. The domain begins at tick 0, and
**no earlier instant is representable**.

That is a statement about the representable range, not about what exists. The
RFC is explicit on the point: `BIG_BANG_CLAIM` is a *signed* window precisely
because "the limit may lie before the datum, which is not representable as a
tick (N12)". The datum is the best available starting point for time as this
system can date it, and everything it dates is dated from there. What may lie
earlier is a question it declines rather than answers.

*Enforced by* **type system** — `Instant::since` returns
`Result<Delta>` and errors rather than going negative; there is no `Sub` impl
yielding a signed value. `Ratio` applies the same discipline to rationals:
`sub` refuses a negative result and directs the caller to `abs_diff`.

### Rule M — monotone total order  §1.2

Order on instants is total and monotone in the tick count.

*Enforced by* **type system** — `Ord` is hand-written over the tick count.
(Hand-written rather than derived because `#[derive]` would demand `P: Ord` on
the profile parameter.)

---

## Profiles, backends, arithmetic

### Rule P — profile binding  §2.5

Profiles are named, versioned, **type-bound**, and tagged in every serialised
form. A timestamp from one profile is not a timestamp in another.

*Guards* **F1** (timestamps shifting when the age constant is revised).

*Enforced by* **type system** — `Instant<P>` is parameterised by profile, and
`compile_fail/cross_profile_arithmetic.rs` and `cross_profile_comparison.rs`
pin that mixing them does not compile.

### Rule W — one domain across backends  §3

Both integer backends accept and reject **exactly** the same values.

*Enforced by* **test** — the whole suite runs against `u512` and `bigint`, and
`bigint` enforces the `[0, 2^512)` ceiling despite being unbounded.

### Rule O — overflow is a typed error  §3

Overflow returns a diagnostic. It never wraps and never saturates.

*Guards* **F7**.

*Enforced by* **lint** — `no-wrapping-arithmetic` bans `wrapping_*` and
`saturating_*` on time types — and by `overflow-checks = true` in the release
profile, so the guarantee holds in optimised builds too.

### Rule E — integrality  §3

**No floating-point value anywhere in the workspace.** Not in a signature, a
field, a constant, an intermediate, or a rendering path. No transcendental
function is evaluated.

*Guards* **F8** (float error and parameter uncertainty conflated).

*Enforced by* **lint** — `float-free` scans every shipped crate. A float
reference implementation is permitted **only** as a test oracle and **only**
when marked; the lint reports every exemption it honours, so an escape hatch
cannot become a silent retreat.

### Rule R — rounding only on rendering  §3

Values round when displayed, never when constructed, and always under a mode
the caller names.

*Enforced by* **type system** — `Value::Quantity` carries the exact rational
and renders at the last possible moment, so the digit count and the mode belong
to the caller (`--decimals`, `--round`) rather than to a constant at the call
site. `Value::Bridge` does the same for foreign units: showing one is a request,
not a default. Also **lint** — `rounding-is-declared` fails on any
`to_decimal_string` or `snap` in a shipped library crate that does not carry a
marker saying why its mode is forced. Also **test** — a property fails if any
call site in `crates/ucal` formats a decimal itself instead of going through
`Value::quantity`, and `no_earth_units.rs` fails if a non-Earth command prints
a foreign unit unasked.

`Ratio::to_decimal_string` remains the single rounding path. `Ratio::snap` is
still the one place a *computation* discards information; it is directed,
documented, and used only to keep a certified sum's denominator bounded, and
`the_quadrature_snaps_outward` checks that its two uses widen rather than
narrow.

*History, because the enforcement line is the point of this file.* Until 0.5.0
this rule read **convention plus review**, and it was one of only two that did.
Every alignment defect found by a reader across 0.3.0 and 0.4.0 landed on it:
twelve render sites with an undeclared mode, a year that never said which year,
a tick's length printing as `0.000000`, and cosmological widths given in Earth
years and nothing else. No type-enforced rule leaked once. That is the argument
of §29 arriving from the inside, and it is why the line above now names
mechanisms.

*Not covered.* Four shipped sites round under a fixed mode and are exempt with
a stated reason, listed by `xtask -- lint` on every run: a calendar label's
day fraction, which must truncate or it names the following day; the two
quadrature snaps; and the audit's own prose figures.

---

## The grid

### Rule G — the tier grid  §4

Tiers are the powers `5^(5k)`, a universal ladder, five base-5 digits each.

*Enforced by* **test** and by construction — the tier table is computed, not
transcribed.

### Rule N — names are display-only  §4

A tier's canonical identity is its **exponent**. Names (`beat`, `drift`,
`deep`, and their `ru` equivalents) are display aliases. Nothing decides
behaviour from a name.

*Enforced by* **type system** — `Tier` carries an exponent; lookup by name goes
through `resolve_tier_name` and returns a `Tier`. Also **test** — §13.5 requires
the tier table, the locale table and `docs/TIERS.md` to come from one source,
and `xtask -- check-docs` fails when they diverge.

---

## Uncertainty

### Rule T — truncation is uncertainty  §5

A value stated to a coarser tier **is an interval**, `[v, v + 5^e − 1]`. No
parse path may return tick precision from truncated input.

*Guards* **F2** (precision invented by zero-filling).

*Enforced by* **test** — `rule_t_no_invented_precision.rs` sweeps the parse
paths exhaustively — and by **type system**, since `parse` returns
`(Instant, Precision)` and `window_at` turns the pair into the interval it
denotes.

*Amended by* **D-A8**: what a printed form means, and how each form is anchored.

### Rule U — interval arithmetic  §5

Operations on windows produce windows. A midpoint is a rendering choice, not a
measurement.

*Enforced by* **type system** — `Window` has no operation that silently returns
a point; `midpoint` requires an explicit `Rounding`.

---

## Notation and identity

### Rule D — two text forms, one value  §6

The human form and the `digit5` form denote the same value. Round-tripping is
exact.

*Enforced by* **test** — golden fixtures in both forms.

*Amended by* **D-A8** (anchoring: human at T0, `digit5` at T32) and **D-A4**
(Appendix C's human forms are truncated at T−5).

### Rule S — sort order  §6

Lexicographic sort agrees with chronological order on the **binary form and the
UCID only** — never on text.

*Enforced by* **test**, and by the fixed-width binary encoding.

### Rule B — fixed 64-byte canonical binary  §7

One encoding, 64 bytes, byte-identical across backends.

*Guards* **F5** (a backend change silently changing the wire format).

*Enforced by* **test** — both backends emit identical bytes for every fixture.
The `u512` and `bigint` features are mutually exclusive at compile time, with a
`compile_error!` explaining that a build with both would have an ambiguous
canonical encoding.

*Amended by* **D-A7**: full-width encode is 45 `divmod` steps, not 44.

### Rule I — UCID range and non-uniqueness  §7

52 Crockford characters. A UCID identifies an instant, not an event: two events
at the same tick share one.

*Enforced by* **test** — round-trip and range.

---

## The SI bridge

### Rule L — leap seconds at the boundary only  §8

Absolute-time arithmetic never sees a leap second. The pivot is TT.

*Guards* **F4**.

*Enforced by* **test** — differential against `hifitime` at every leap-second
boundary.

Note the era this makes visible rather than hides: 1961–1972 UTC ran at a
variable rate with fractional offsets. It is exactly representable here, because
every rate coefficient divides by 27 and cancels 86400's factor of 3³. UTC
before 1961 is refused, because there is nothing to convert exactly.

---

## Calendars

### Rule K — one derivation mechanism  §9

There is **one** calendar-derivation path. Earth is an instance of it, not the
template for it. Legacy calendars are quarantined and labelled.

*Guards* **F9** (Earth becoming the template).

*Enforced by* **crate graph** — `ucal-body` must not depend on `ucal-civil`,
checked by the `dependency-direction` lint — and by **test**: `earth-d` and
`mars-d` are constructed by the identical generic path from data alone. Also
**type system** — `compile_fail/legacy_is_not_derived.rs` and
`legacy_fields_cannot_render.rs`.

*Amended by* **D-A5**: grouping cycles are declared per body via a named
grouping satellite, not admitted by a global bound. The bound the RFC specified
was calibrated on Earth's Moon, which made it an Earth constant inside the one
mechanism this rule exists to keep Earth-free.

### Rule J — the anchor  §9

Phase is not derivable from the tick, the datum and the body's periods. The
anchor is therefore **declared, cited, interval-valued and body-defined**, and
its absence is `UCAL-E0062` — not a guess, and not a fallback to another body.

*Guards* **F10** (a body calendar silently phasing off Earth).

*Enforced by* **type system** — no constructor accepts a calendar without an
anchor.

Titan has no anchor and will not be given one by invention: no published
convention exists to cite. `titan-d` is therefore complete in units,
intercalation and cycles, and incomplete in phase. That state is represented
explicitly rather than defaulted away.

### Rule C — body parameter provenance  §9

Every body parameter carries epoch, rate, validity window, and its as-measured
value — **in ticks**.

*Guards* **F6** (a derived calendar drifting because parameters were treated as
constants).

*Enforced by* **test** — `UCAL-W0003` fires on out-of-window evaluation.

*Amended by* **D-A11** (obliquity cannot be a `RatedParam`) and **D-A13** (a
drift bound is a rate, not a duration, and is stated in the body's own local
days and years).

---

## Cosmology

### Rule X — certified enclosures  §10

A cosmological result is an **enclosure**, never a point estimate. Model,
parameter set and citation accompany every result. **Arithmetic width and
parameter width are reported separately and never merged.**

*Guards* **F8**.

*Enforced by* **type system** — `CosmoResult` has two distinct width fields, and
summing them requires calling `total_width()` by name.

The separation is not pedantry. At `z = 1100` and the default depth the
arithmetic width is 251 years and the parameter width is 10 917 years. A single
merged tolerance would have hidden the fact that more computation buys nothing —
which is precisely how GE-1's kill criterion came to be answerable.

*Amended by* **D-A14** (§10.3's integral is improper and cannot be quadratured
as written) and **D-A15** (Appendix H.4's monotone case does not apply).

---

## Where the rules are weakest

Honesty about enforcement is worth more than a table of green ticks.

- **Rule F** (frame) and **Rule R** (rounding on rendering) rest on convention
  and review. Both are followed throughout; neither is mechanically enforced.
- **Rule A**'s lint covers *identifiers* in `ucal-core`. It cannot stop a
  foreign unit entering as a bare integer with a misleading name in another
  crate.
- **Rule Q.1**'s prose lint matches known phrasings. It catches the overclaims
  the RFC named; it cannot catch one nobody has thought of yet.

These are the places where the next contributor's care is load-bearing.
