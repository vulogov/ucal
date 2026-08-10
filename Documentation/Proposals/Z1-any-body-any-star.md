# Z1 — any body, any star

**Status: research. Every claim below was run against `target/release/ucal`
rather than reasoned about, and the commands are included so they can be run
again.**

---

## The question

> A user must be able to add and use whatever celestial body, with whatever
> orbital and stellar parameters.

## The short answer: most of it already works, and nothing in it is
solar-system-shaped

1.4.0's body-file loader takes HJSON, and the first thing to establish is what
it will already accept. A planet around another star, with a `primary` naming
that star:

```hjson
id: kepler-22b
primary: kepler-22
rotation_period:  { value: 1.5,       unit: d, citation: hypothetical, valid_years: 100 }
solar_day:        { value: 1.5024,    unit: d, citation: hypothetical, valid_years: 100 }
orbital_period:   { value: 289.8623,  unit: d, citation: Borucki et al. 2012, ApJ 745, 120,
                    locator: https://doi.org/10.1088/0004-637X/745/2/120, valid_years: 100 }
```

```
$ ucal cal derive kepler-22b.hjson
days_per_year  192.932841
leap_rule:
  rule                 125/134
  convergent           4
  whole_days_per_year  192
```

That is a working calendar for a planet 620 light years away, derived by the
same code path as Earth's, with no entry in any table in this repository.

**There is no star model, and that is why it works.** `primary` is a label. A
body's year is whatever period the file states, and Rule K.2's rule for a
satellite — that its year is its primary's orbit, not its own — is a rule about
which number the author writes down, not a lookup. Stellar mass, luminosity,
spectral type and distance appear nowhere in the derivation, because none of
them is a period, and Rule K derives a calendar from periods.

So the answer to "whatever orbital/star parameters" is that the format needs
fewer of them than expected, not more. What follows is what it is actually
missing, in order of how much it costs.

---

## Finding 1 — a file cannot say "derive this", and for most bodies that is the parameter that matters

This is the sharp one, and it is not a limitation of HJSON.

Six of the twelve derived calendars that ship — Titan, Io, Europa, Ganymede,
Callisto, Enceladus — do not state a solar day. They *derive* it:

```
    solar_day = 1 / (1/P_rotation - 1/P_year)
```

because tidal lock fixes the body's face towards its primary and not towards
the Sun, and **no source publishes a solar day for a tidally locked body**. The
compiled-in data carries the result exactly, as a ratio.

A file cannot. §15.1's format has one way to state a parameter — a decimal
string — so an author must round the derivation, and rounding it changes the
calendar. Measured on Europa, whose file ships as this repository's documented
example:

| decimals | value | derived rule |
|---|---|---|
| 2 | `3.55` | `47/105` |
| 3 | `3.554` | `2/27` |
| 4 | `3.5541` | `5/126` |
| 5 | `3.55409` | `5/116` |
| 6 | `3.554094` | `1/24` |
| 12 | `3.554094092244` | `1/24` |

Six decimals happen to settle Europa. That is a fact about Europa — its rule is
the second convergent, reached before the far terms of the continued fraction
start moving — and not a rule anyone can apply to a body they have not tried.

**The cost of this gap has already been paid once, in this repository.** The
documented example stated `3.552106`, cited the NASA fact sheet for it, and was
wrong: that source does not publish the figure, and the value was off in the
third decimal. It derived `202/279` against the real body's `1/24` — a different
calendar, in the file that documents how to write one. It survived a full
release cycle, because the only check on it was that it parsed. It became
visible the moment Y3 shipped `europa-d` and an independent computation of the
same quantity existed to compare against.

That is the whole argument for this feature. The format's inability to express a
derivation does not merely inconvenience an author; it converts a derivable
exact quantity into a hand-typed constant, which is the failure mode Rule C
exists to prevent, and it is silent.

### Z1.1 — a derived form

```hjson
solar_day: {
  derived: synodic          // 1 / (1/rotation - 1/orbital_period)
  citation: derived from the rotation and orbital period cited in this file
  valid_years: 10000
}
```

One named derivation to start with, because one is what the shipped data uses
six times, and a vocabulary of derivations is a thing to add when a second is
needed rather than in advance. `RatedParam::derived` already exists and already
carries the formula string, the reason and the underlying citations into the
provenance output — the library side is done, and this is the file syntax for
reaching it.

**Stop if** the derivation cannot be expressed without the file also declaring
which parameters feed it, at which point the format is a small expression
language and the honest answer is that derived bodies are code.

---

## Finding 2 — the unit vocabulary is `s`, `d`, `yr`, and sources publish hours

```
$ ucal cal derive uranus.hjson       # rotation_period stated in hours
UCAL-E0060: unit must be `s` (SI second), `d` (86 400 s) or `yr` (Julian year)
```

`MeasuredUnit` has three variants. The NASA planetary fact sheets — the source
this project cites more than any other — publish rotation period and length of
day **in hours**. `data::jupiter` handles this by converting in a comment:

```rust
// 9.9250 h x 3600 = 35 730 s, exact.
param(35_730, 0, MeasuredUnit::SiSecond, NASA_FACT_SHEET, 1_000),
```

An author of a file has to do that conversion by hand, and Rule C requires the
*published value verbatim*. A file stating `35730 s` for a source that printed
`9.9250 h` has already departed from verbatim, and one stating `0.4134375 d` has
rounded — which is Finding 1 again.

### Z1.2 — add `h` and `min`

Both are exact multiples of the SI second (3600 and 60), so this costs one enum
variant, two match arms and no precision. It is additive to a
`#[non_exhaustive]` enum, so it is not breaking.

**Stop if** a unit is wanted that is not an exact multiple of the second, which
would put a rounding inside the unit conversion and is a different decision.

---

## Finding 3 — a tidally locked body has no solar day, and the program says something else

Give the loader a synchronous body — the most likely habitable-zone case around
an M dwarf, and the state of every large moon in the outer solar system:

```
$ ucal cal derive proxima-b.hjson
UCAL-E0061: no convergent within the permitted depth meets the requested drift
bound; either widen the bound or raise the depth
```

Neither of those will help, because the problem is not depth. If rotation equals
the orbital period then

```
    1/solar_day = 1/P_rotation - 1/P_year = 0
```

and the solar day is unbounded. **The body has no day.** The star sits at a
fixed point in its sky forever; there is no noon, no midnight, and no sequence
of days for a calendar to count. That is a real and interesting Rule K result —
the derivation is telling the truth about the body and the message is describing
a numerical difficulty.

### Z1.3 — say it

Detect the degenerate case where rotation and year coincide within the stated
precision of the inputs, and return a diagnostic that says the body is tidally
locked and therefore has no solar day, so it has no day-based calendar. It
should name what *can* still be derived — the year is a period and remains one.

This is the same shape as `cal derive`'s existing answer for a body with no
satellite: *"no grouping satellite, so its calendar has no month. That is the
output, not a gap."* §15.3 forbids supplying a fallback structure, and a
fallback day would be a worse invention than a fallback month.

**Stop if** the degeneracy cannot be detected without a tolerance, and the
tolerance turns out to be arbitrary. A body one part in 10⁹ from synchronous has
a solar day of 10⁹ rotations, which is finite, absurd, and correct — and picking
where absurd becomes zero is a judgement the program should not make silently.
If that is where this lands, the finding is recorded and the message is fixed to
stop giving advice that cannot work.

---

## Finding 4 — the body-file loader raises `UCAL-E0010`, which means something else

```rust
Code::E0010 => "locale table load failure"
```

`body_file::load` raises it for a malformed body file. A body file is not a
locale table, and the user is told so:

```
$ ucal cal derive malformed.hjson
UCAL-E0010: locale table load failure (the body file is not well-formed HJSON)
```

This is the exact defect [D-A18] and [D-A19] fixed twice before — a code
borrowed for its exit status, carrying a name that describes a different
failure. It was introduced in 1.4.0 by the loader that also, correctly, gave
`UCAL-E0012` its first raiser.

### Z1.4 — its own code, and a delta

A new `Code` variant for a malformed data file, appended to the end of the enum
— which is where `E0014` had to be moved after being inserted mid-enum shifted
every discriminant below it, caught by `cargo semver-checks` — and a spec delta
recording the correction, as D-A18 and D-A19 did.

**Stop if** nothing: this is a defect with a known fix and no design question.

---

## The rest, in descending order of consequence

**No anchor file.** §15.1 names body files *and* anchor files. This is
[Y2](../Release_Notes/1.5.0.md), already in the current cycle's scope, and it is
the difference between a file producing an intercalation rule and a file
producing a date.

**`cal derive` is one-shot.** A file-derived calendar cannot be used by
`ucal cal show`, `ucal show` or `ucal between`, because the registry is
`calendar::registered()` and is compiled in. A `--body <file>` accepted by the
commands that take a calendar id would close it. This is what "use whatever
celestial body" asks for most directly, and it is a larger change than the four
above because it touches every command's calendar lookup.

**No retrograde rotation.** `Measured` carries an unsigned mantissa, so Venus's
published `-5832.6 h` is stored as a magnitude with the sign in a comment. A
file cannot express it either. This is item 3 of [`ROAD-TO-2.0.md`](ROAD-TO-2.0.md)
and rides along with that bundle.

**No obliquity.** `data::jupiter` carries an `AngleParam`; the file format has no
key for one. Nothing in the calendar derivation reads it today, which is why
this is last: adding a key that feeds nothing would be furniture.

**No `ucal cal validate`.** Nothing answers *are these parameters sane* before a
file is committed. Finding 1 is the argument that this matters — a wrong figure
with a citation attached parses perfectly — and also the argument that it is
hard: what caught the Europa error was an independent computation of the same
quantity, not a validation rule. What a validator could do is narrower and still
worth having: flag a stated solar day that differs from the synodic derivation
of the file's own rotation and year by more than the stated precision, which
would have caught it.

---

## Order, and what to do first

The four numbered items are all small, all additive, and all binary-only —
`ucal-body` is untouched, so §15.1 stays `UNIMPLEMENTED` for the library and
[D-A20] continues to carry the reason.

1. **Z1.4** (the wrong error code) — a defect, no design question.
2. **Z1.1** (the derived form) — the one that changes what can be authored, and
   the one with a measured cost for not having it.
3. **Z1.2** (hours) — small, and it removes a hand conversion from every file
   that cites a fact sheet.
4. **Z1.3** (the locked-body answer) — worth doing if the tolerance question in
   its kill criterion has an answer, and worth recording if it does not.

`--body <file>` for the other commands is the largest and should follow Y2,
because a body file and an anchor file together are what make a file-derived
calendar answer the questions those commands ask.

## What this does not fix

A loader makes it possible for someone else to derive a calendar; nothing here
makes anyone want to. That was true when [`X1`](X1-authoring-local-calendars.md)
said it and it is still true. The difference this page makes is that the reason
to do the work no longer depends on an audience: Finding 1 is a defect that
already occurred, in this repository, in the file that documents the format, and
it would have occurred to anyone else who authored a body that is tidally
locked.

[D-A18]: ../../spec/SPEC-DELTAS.md
[D-A19]: ../../spec/SPEC-DELTAS.md
[D-A20]: ../../spec/SPEC-DELTAS.md
