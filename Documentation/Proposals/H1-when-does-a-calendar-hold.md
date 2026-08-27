# H1 — when does a calendar hold?

**Status: three findings, evidenced and unfixed. One of them is a normative
requirement the code produces and then discards.**

---

## Where this came from

A reader asked two questions about the grouping cycle:

> Satellites can appear and disintegrate during billions of ticks. Orbital
> parameters may change. That raises the question: if a tick is absolute and a
> cycle is tied to something that changes as a body evolves, how can we specify
> a cycle — but *when*?

The questions are about the calendar's *validity in time*, and this project
already carries the machinery to answer them: every parameter has an epoch, an
optional secular rate, and a validity window, and Rule C says what to do outside
one. What the questions found is that **the machinery is not connected to
anything a user can see.**

Nothing below is a design disagreement. All three are gaps between what the code
declares and what the code does.

---

## Finding 1 — `UCAL-W0003` is produced once, discarded once, and can never reach a user

The strongest of the three, and the one with a spec requirement behind it.
`crates/ucal-body/src/param.rs` opens by quoting the rule it implements:

> MUST warn (`UCAL-W0003`) and MUST NOT silently extrapolate.

The implementation is correct. `RatedParam::evaluate(at)` returns
`(Ratio, Option<Warning>)` and yields `Warning::W0003` whenever `at` falls
outside the parameter's declared window. A test asserts it. `evaluate_strict`
offers callers the option of taking it as an error instead.

**There is one production caller in the workspace**, and it is
`crates/ucal-body/src/calendar.rs:228`:

```rust
let (v, _) = self.body.solar_day().evaluate(t)?;
```

The warning is bound to `_` on the line that produces it.

### The evidence

Earth's parameters declare `valid_years: 10_000`. Asked for a date five times
outside that window:

```
$ ucal cal show earth-d <J2000 + 50 000 Julian years>
fields:
  year             50002
  day              27
  day_fraction     0.499261
  anchor_revision  1
```

Exit 0. No warning. `W0003` occurs zero times in any output this program can
emit — the string appears in the library, in its tests, in the book, and in the
diagnostics table, and never on a terminal.

### It is worse than one dropped value

There are **three** layers of correct machinery and the production path goes
round all of them.

`Body::days_per_year(at)` exists precisely for this, and its doc comment says
so:

> Returns any warning from either parameter, so an evaluation outside a validity
> window cannot become invisible by being combined with another (Rule C,
> `UCAL-W0003`).

**It has no production callers** — five call sites, every one of them a test.
`BodyCalendar::build` does not use it; it
reads `value_at_epoch()` from both parameters directly, bypassing evaluation and
warning together:

```rust
let leap_rule = derive_leap_rule(
    body.solar_day().value_at_epoch(),
    body.orbital_period().value_at_epoch(),
    bound,
    max_depth,
)?;
```

So: a function that warns, a function that combines warnings and explains why it
must, and a build path that calls neither.

### What this is an instance of

The same shape this project keeps finding in its own work — a claim with no
mechanism, a mechanism with no wire, a schedule with no date. **This is a
requirement with no wire**, and it is the first one found where the spec uses
the word MUST.

### The fix, and its cost

`DerivedFields` already propagates uncertainty; it would gain a warning. Every
command that renders local fields would render it. The `Doc` renderer has no
warning channel today, which is the actual work — `cal show`, `show`,
`cal derive --at` and the wall clock's dials all reach `fields()`.

**Stop if** surfacing it makes the common case noisy. It should not: every
shipped calendar's window is ±10 000 Julian years and the `earth-civil` range
is much smaller, so an ordinary date warns about nothing. If that turns out to
be wrong, the finding is that the declared windows are too narrow for the dates
people ask about, which is worth knowing and is a different repair.

---

## Finding 2 — the day drifts and the intercalation does not

Within one calendar, at one instant:

| quantity | evaluated at | with its rate? |
|---|---|---|
| `solar_day` | **`t`** | yes |
| `orbital_period` | J2000 only | no |
| `leap_rule` | J2000, at build | no |
| `cycles` | J2000, at build | no |

`fields()` divides elapsed ticks by a solar day evaluated at `t`, then splits
the resulting whole days into years using a leap rule frozen when the calendar
was built. Walk far enough forward and the day used to count days is not the day
the leap rule was derived from.

### The honest size of it, which is smaller than it looks

**No shipped parameter declares a rate.** `with_rate_per_julian_century` has
three call sites in the whole tree and all three are in `param.rs`'s own tests.
So every rate on every shipped body is `None`, every `evaluate` returns
`value_at_epoch()` unchanged, and **the inconsistency currently produces no
numeric difference at all.** It is structural, latent, and would become real the
moment a rate is added.

That is itself the second half of the finding: the project models secular drift,
its own test cites Earth's rotation lengthening by **1.8 ms per century**, and
no shipped body uses it. Every derived calendar is computed as though its
parameters were constant for all time — which is exactly the assumption Rule C's
validity windows exist to refuse.

### Two ways out, and they are not equivalent

**(a) Evaluate everything at `t`.** The leap rule and the cycle become functions
of the instant. Honest, and expensive in a way that matters: the leap rule
*defines the year boundaries*, so a rule that changes with `t` makes
`year(t)` non-monotonic near the change and there is no obvious right answer for
a date on the seam. `BodyCalendar` would stop being a value and become a
function.

**(b) Freeze everything at the epoch and say so.** The leap rule, the cycle
*and* the day all come from `value_at_epoch()`, and the calendar declares that
it is the calendar of a body whose parameters are those at J2000. Cheaper,
consistent, and it makes the validity window mean exactly what Finding 1 needs
it to mean: outside the window, this calendar is not claimed to hold.

**(b) is the recommendation.** A calendar is a counting convention, and a
counting convention whose rule changes underneath the count is not one. Drift
belongs in the *warning* — "you are outside the window where these parameters
were measured" — not in a silently varying year length.

**Stop if** (b) turns out to break `earth-civil` agreement, which is checked
against `hifitime` over a long range. It should not, since freezing is what the
code effectively does today for two of the three quantities.

---

## Finding 3 — obliquity is declared ten times and read by nothing

`Body::with_obliquity` is called for ten shipped bodies. `obliquity()` is read
in two tests and nowhere else. The type's own doc comment records the intent:

> Obliquity is carried because it is what makes a body have seasons, and a
> future seasonal overlay would need it.

This is the direction the reader's question pointed at: a subdivision of a
body's year that needs **no satellite**, and so exists for the fourteen shipped
calendars that have no cycle at all.

### Why a "universe cycle" is the wrong shape, and seasons are not

A subdivision identical for every body already exists — it is the tier ladder.
`T2` sweep is 5.3 days at Alpha Centauri and here. A second universal ladder
would compete with the first, which is [`W4`](W4-two-ladders.md)'s territory.

And tying a universal subdivision to *orbital parameters* does not escape the
problem: any fraction-of-the-orbit rule is Earth's twelve smuggled in wearing a
physics costume, which is what D-A5 refuses when it says **"month-like" is an
Earth predicate.**

Seasons are different in kind. Solstices and equinoxes are not a convention
imposed on a body; they are where its spin axis points relative to its orbit —
four points that exist for any body with a non-zero tilt, derived rather than
declared. That is the same standard Rule K holds intercalation to.

### The obstacle, stated before anyone starts

**The stored data is not sufficient.** `AngleParam` holds one number, the
obliquity in degrees. That gives the *amplitude* of a seasonal cycle and not its
*phase*: placing an equinox needs the orientation of the spin axis, and the
cited source — the IAU WGCCRE report — publishes α₀ and δ₀ for exactly that and
this project stores neither.

So a seasonal overlay needs a new cited parameter per body, and the quantity it
needs is a **phase**. Rule J.3 makes phase empirical: determined and cited, never
derived. [`D5`](D5-titan-anchor.md) recorded what establishing one honestly cost
for Titan, and the answer there was no anchor.

**Stop if** the pole orientation cannot be quoted verbatim for the bodies that
have an obliquity. Then the honest outcome is that obliquity stays carried and
unused, with this paragraph as the reason — which is better than the present
state, where it is carried and unused with no reason recorded anywhere but a
doc comment.

---

## A fourth thing, which the questions raise and none of the above fixes

**The satellite set has no time coordinate.** Phobos becomes a ring in roughly
fifty million years; the Moon recedes 3.8 cm a year. D-A5 makes the grouping
satellite the *calendar's* declaration, and `calendar::registered` states it as
`Some("moon")` unconditionally. There is no way to say **"grouped by Luna, while
Luna is there."**

A body's `formation` window is the nearest existing idea — `Body::with_formation`
takes one — and `formation()` is read by nothing at all, so the same gap exists
one level up: this project can already say when a body began and never consults
it.

Recorded rather than proposed. It is a data-model question, and answering it
inside a `1.x` cycle is not obviously possible.

---

## What this page does not decide

Whether any of it is 1.9.0's work. The cycle's scope is C1–C3 and the F-series,
all of which have landed, and adding three more items to a cycle that already
widened once is a decision for the author rather than a conclusion of the
measurement.

Finding 1 is the one that would be hard to justify carrying: the spec says MUST,
the code produces the warning, and one underscore keeps it from anybody.
