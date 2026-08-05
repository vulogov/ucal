# What 1.0 will promise

Written before 1.0 rather than after, so that the promise is a decision rather
than a description of whatever happened to be true on the day.

**Status: this document describes an intention. The crates are `0.x` and promise
nothing yet.** Every `0.x` release is permitted to break anything, and three of
the five so far did.

---

## The floor: semver

Within `1.x`, no release removes a public item, changes a public signature, or
narrows what compiles. That is the ordinary contract and it is the least
interesting part of this file.

## Above the floor

Semver has no vocabulary for most of what this project actually claims. Five
promises, each with the mechanism that keeps it — because a promise with no
mechanism is a convention, and 0.5.0 was spent removing the last two of those.

### 1. `BIG_BANG_CLAIM` never acquires an arithmetic operator

The published identification of the origin is metadata. It is fully readable,
fully cited, carries an exact magnitude, and **cannot enter a computation**
(Rule Q.3). No `1.x` release adds an operator, a conversion, or an accessor that
would let it.

*Enforced by* three compile-fail cases —
`signed_window_as_operand`, `signed_window_into_delta`,
`signed_window_arithmetic` — which must fail to build. If one starts compiling,
the promise has been broken by the type system rather than by a person.

### 2. A rounding never becomes undeclared again

Values round when displayed, never when constructed, and always under a mode the
caller can name (Rule R). `--decimals` and `--round` reach every rendered
rational, and every rendering that cannot take them says why.

*Enforced by* the `rounding-is-declared` lint over the library crates, a
property test that no call site in `crates/ucal` formats a decimal without going
through `Value::quantity`, and the certification map that lists every rounding
in the output.

*History.* This was a convention until 0.4.0, and every alignment defect a
reader found in two cycles landed on it. That is why it is on this list.

### 3. No Earth unit appears outside an Earth context unless asked for

A Julian year is 365.25 of Earth's rotations; an SI second is an Earth unit.
Neither is used to describe something that is not of Earth without `--bridge`.
`to-civil` and `from-civil` are exempt because they *are* Earth calendar
commands, and `datum`'s provenance chain is exempt because §19.2 requires it and
it records where Earth entered.

*Enforced by* `no_earth_units.rs`, and structurally by `Value::Bridge`, which
makes "this is a foreign unit" a property of the value rather than something a
call site must remember.

### 4. `ucal-json/1` fields never change name, shape or meaning

New fields may appear — a consumer must ignore what it does not recognise. No
existing field is removed, renamed, or changed from a string to an object.

*Enforced by* `fixtures/json-surface.txt`, a committed baseline of every field
path and the JSON kind it serialises to, checked by `json_surface.rs`. A removed
path or a changed kind fails; an added path does not.

**Its limit, stated because a green run would otherwise imply more.** A field
that keeps its name and its kind and begins reporting something *else* passes
this check, and is exactly the breakage this promise is about. Nothing
mechanical reaches meaning. What the baseline gives is that the *shape* cannot
drift unnoticed, which is the part that can be automated.

### 5. A certified enclosure never narrows silently

An enclosure claims the true value provably lies inside it. Within `1.x`, an
enclosure may widen — a correction always may — and may narrow only when the
model or the depth changed and the release notes say so.

*Enforced by* the outward-rounding tests within a version: every step of the
quadrature widens, the two accumulator snaps are directed apart, and quantising
to ticks floors the lower bound and ceils the upper.

**Its limit.** Those checks run against one version. Nothing compares an
enclosure across releases, so this promise is the weakest on the list — it rests
on the release-notes discipline that has caught a `Breaking` omission twice, not
on a mechanism. It is listed anyway, because a promise stated with its weakness
is worth more than one silently omitted.

---

## What 1.0 does *not* promise

### The specification is not frozen

UCAL-1 was superseded, not finished. `spec/SPEC-DELTAS.md` carries sixteen
recorded amendments and D-A16 was written in 0.4.0. **1.0 freezes the API; the
specification keeps its amendment procedure.** A delta that changes behaviour
still produces a breaking change and still needs a major bump — the two are
independent.

### The text output is not an interface

`--json` is the contract. The human rendering — tables, colour, wrapping, column
order, the certification block — may change in any release. A script that parses
the text output is relying on something this file does not protect, and `--json`
exists so that it does not have to.

### The MSRV, until S3 says otherwise

`rust-version = "1.85"` today, with no stated policy for moving it. A 1.0 that
silently raised its MSRV in a patch release would have broken something semver
does not model. **This is an open item in 0.6.0.**

### Feature combinations, until S4 enumerates them

`u512` and `bigint` are mutually exclusive and say so with a `compile_error!`.
The rest interact in ways only the test matrix knows, and it tests two
combinations. **Open in 0.6.0.**

### Performance

The certified quadrature is slow on purpose — GE-1 measured depth-24 at hours
rather than seconds and the kill criterion fired. No release promises a runtime,
and a future release may be slower if being slower is more correct.

### The documents

The book, the Typst papers, and the proposals are not versioned with the crates
and carry no compatibility promise.

---

## What would make 1.0 dishonest

Recorded here because the pressure to ship a version number is real and arrives
without an argument attached.

**Shipping before contact.** The API has never been used by anyone but its
author. Every breaking change so far cost nothing, which means none of them
tested whether the surface is *right* — only that it could be changed. A promise
made in that state is a promise about a guess, and
[`ROAD-TO-1.0.md`](Proposals/ROAD-TO-1.0.md) makes 0.7.0 the gate for exactly
this reason.

**Freezing a surface nobody has enumerated.** At the opening of 0.6.0, thirty-
nine public types carried public fields or variants and no `#[non_exhaustive]`.
Each is a decision 1.0 makes permanent in one direction or the other, and
shipping without deciding *is* deciding.

**Promising what is merely true today.** Every item above names a mechanism or
admits it has none. A list of things that happen to hold is not a promise; it is
a snapshot that a reader will mistake for one.
